//! Frequency Test within a block.
//!
//! This tests for the same property as [crate::frequency_test], but within M-bit blocks.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::test_runner::TestRunner;
use crate::{Error, TestResult, BYTE_SIZE, Test};
use rayon::prelude::*;

/// The argument for the Frequency test within a block: the block length.
#[repr(C, u8)]
pub enum FrequencyBlockTestArg {
    /// The block length is measured in bytes - this allows for faster performance.
    Bytewise(usize) = 0,
    /// Bitwise block length
    Bitwise(usize) = 1,
    /// A suitable block length will be chosen automatically
    ChooseAutomatically = 2,
}

impl FrequencyBlockTestArg {
    /// Creates a new block length argument for the test. The passed block length is in bits.
    /// If block_length is a multiple of 8 (bits in a byte), [Self::Bytewise] is automatically
    /// chosen. Else, [Self::Bitwise] is chosen.
    pub fn new(block_length: usize) -> Self {
        if block_length % BYTE_SIZE == 0 {
            Self::Bytewise(block_length / BYTE_SIZE)
        } else {
            Self::Bitwise(block_length)
        }
    }
}

/// Frequency test within a block - No. 2
///
/// See the [module docs](crate::frequency_block_test).
/// If test_arg is [FrequencyBlockTestArg::ChooseAutomatically], a reasonable default, based on 2.2.7, is chosen.
/// If an error happens, it means either arithmetic underflow or overflow - beware.
pub fn frequency_block_test<R: TestRunner>(
    runner: &R,
    data: &BitVec,
    test_arg: FrequencyBlockTestArg,
) -> Result<TestResult, Error> {
    // Step 0 - get the block length or calculate one
    match test_arg {
        FrequencyBlockTestArg::Bytewise(block_length) => {
            frequency_block_test_bytes(runner, data, block_length)
        }
        FrequencyBlockTestArg::Bitwise(block_length) => {
            frequency_block_test_bits(runner, data, block_length)
        }
        FrequencyBlockTestArg::ChooseAutomatically => {
            let block_size = choose_block_length(data.data.len());
            frequency_block_test_bytes(runner, data, block_size)
        }
    }
}

/// Frequency test within a block, optimization for block sizes that are whole bytes.
///
/// The passed block_length has to in bytes.
fn frequency_block_test_bytes<R: TestRunner>(
    runner: &R,
    data: &BitVec,
    block_length_bytes: usize,
) -> Result<TestResult, Error> {
    // Step 1 - calculate the amount of blocks
    let block_count = data.data.len() / block_length_bytes;
    let block_length_bits = block_length_bytes * BYTE_SIZE;

    let half_chi: f64 = data
        .data
        .chunks_exact(block_length_bytes)
        .map(|chunk| {
            // Step 2 - calculate pi_i = (ones in the block) / block_length for each block

            // count of ones in the block
            let count_ones = chunk
                .par_iter()
                .try_fold(
                    || 0_usize,
                    |sum, value| {
                        sum.checked_add(value.count_ones() as usize)
                            .ok_or(Error::Overflow(format!("adding ones to the sum: {sum}")))
                    },
                )
                .try_reduce(
                    || 0_usize,
                    |a, b| {
                        a.checked_add(b).ok_or(Error::Overflow(format!(
                            "Adding two parts of the sum: {a} + {b}"
                        )))
                    },
                )? as f64;

            let pi = count_ones / (block_length_bits as f64);

            // Step 3 - compute the chi^2 statistics - calculate the current part
            let chi_part = (pi - 0.5).powi(2);
            Ok::<_, Error>(chi_part)
        })
        // Step 3 - build sum and multiply with 4 * block_length
        // In Step 4, chi is again halved - do this now (replace 4 with 2)
        .sum::<Result<f64, _>>()?
        * 2.0
        * (block_length_bits as f64);

    check_f64(half_chi)?;

    // Step 4: compute p-value = igamc(block_count / 2, chi / 2)
    let p_value = igamc(block_count as f64 / 2.0, half_chi)?;

    check_f64(p_value)?;

    let result = TestResult { p_value };
    runner.store_result(Test::FrequencyTestWithinABlock, result);
    Ok(result)
}

/// Choose a block length based on 2.2.7. Needs the amount of bytes (each byte contains 8 values)
/// as the parameter. This method can only choose byte-sized blocks.
fn choose_block_length(byte_vec_length: usize) -> usize {
    const MIN_BLOCK_LENGTH: usize = 20 / BYTE_SIZE + 1;

    // Start with the recommended minimum block length based on the length of the data.
    // This also satisfies that less than 100 block should exist.
    let block_length = byte_vec_length / 100 + 1;

    if block_length < MIN_BLOCK_LENGTH {
        // has to be at least the min block length
        MIN_BLOCK_LENGTH
    } else {
        block_length
    }
}

/// Frequency test within a block for bit lengths that are not byte-sized.
fn frequency_block_test_bits<R: TestRunner>(
    runner: &R,
    data: &BitVec,
    block_length_bits: usize,
) -> Result<TestResult, Error> {
    // Step 1 - calculate the amount of blocks
    let block_count = data.len_bit() / block_length_bits;

    // Step 2 - calculate pi_i = (ones in the block) / block_length for each block.
    // We can't split in chunks here, because chunks would only catch whole bytes.

    // How many bytes are needed - there could be unused bytes at the end
    let bytes_needed = if block_length_bits * block_count % BYTE_SIZE == 0 {
        // no remainder
        block_length_bits * block_count / BYTE_SIZE
    } else {
        // a remainder is left: 1 additional byte is needed for it.
        block_length_bits * block_count / BYTE_SIZE + 1
    };

    // if the remainder needs to be used
    let (bytes_needed, add_remainder) = if bytes_needed > data.data.len() {
        (data.data.len(), true)
    } else {
        (bytes_needed, false)
    };

    let mut count_ones_per_block = data.data[0..bytes_needed]
        .into_par_iter()
        .enumerate()
        .try_fold(
            || vec![0_usize; block_count],
            |mut sum, (idx, value)| {
                // Calculate, based on the index, the index of the current block
                let current_block_idx = idx * BYTE_SIZE / block_length_bits;

                // Calculate, based on the index, how many bits of this block need to go where.
                // This formula is best explained by an example:
                //
                // Block size: 9 bits
                // Data:
                // 0 0 0 0 0 0 0 0 || 0|0 0 0 0 0 0 0 || 0 0|0 0 0 0 0 0 || 0 0 0|0 0 0 0 0 || 0 0 0 0| ...
                // 1 | means a block border, 2 || mean a byte border
                // Here we have 5 bytes with 4 blocks.
                //
                // The formula for calculation is
                // (idx + 1) * BYTE_SIZE % block_length_bits >= BYTE_SIZE
                // with BYTE_SIZE = 8
                //
                // For byte 0: (0 + 1) * 8 % 9 = 8 --> take the full byte for the current block
                // For byte 1: 2 * 8 % 9 = 7 --> take 8 - 7 = 1 bits for the current block, 7 bits for the next
                // For byte 2: 3 * 8 % 9 = 6 --> take 8 - 6 = 2 bits for the current block, 6 bits for the next
                // For byte 3: 4 * 8 % 9 = 5 --> take 8 - 5 = 3 bits for the current block, 5 bits for the next
                // For byte 4: 5 * 8 % 9 = 4 --> take 8 - 4 = 4 bits for the current block, 4 bits for the next
                //
                // Other examples:
                // Block size: 7 bits
                // 0 0 0 0 0 0 0|0 || 0 0 0 0 0 0|0 0 || 0 0 0 0 0|0 0 0 || 0 0 0 0|0 0 0 0 || 0 0 0| ...
                //
                // For byte 0: 1 * 8 % 7 = 1 --> take 8 - 1 = 7 bits for the current block, 1 bit for the next
                // For byte 1: 2 * 8 % 7 = 2 --> take 8 - 2 = 6 bits for the current block, 2 bits for the next
                // For byte 2: 3 * 8 % 7 = 3 --> take 8 - 3 = 5 bits for the current block, 3 bits for the next
                // For byte 3: 4 * 8 % 7 = 4 --> take 8 - 4 = 4 bits for the current block, 4 bits for the next
                // For byte 4: 5 * 8 % 7 = 4 --> take 8 - 5 = 3 bits for the current block, 5 bits for the next
                //
                // Block size: 11 bits
                // 0 0 0 0 0 0 0 0 || 0 0 0|0 0 0 0 0 || 0 0 0 0 0 0|0 0 || 0 0 0 0 0 0 0 0 || 0| ...
                // For byte 0: 1 * 8 % 11 = 8 --> take the full byte for the current block
                // For byte 1: 2 * 8 % 11 = 5 --> take 8 - 5 = 3 bits for the current block, 5 bits for the next
                // For byte 2: 3 * 8 % 11 = 2 --> take 8 - 2 = 6 bits for the current block, 2 bits for the next
                // For byte 3: 4 * 8 % 11 = 10 >= 8 --> take the full byte for the current block
                // For byte 4: 5 * 8 % 11 = 7 --> take 8 - 7 = 1 bits for the current block, 7 bits for the next
                let remainder = (idx + 1) * BYTE_SIZE % block_length_bits;
                if remainder >= BYTE_SIZE {
                    // take the whole block
                    sum[current_block_idx] = sum[current_block_idx]
                        .checked_add(value.count_ones() as usize)
                        .ok_or(Error::Overflow(format!(
                            "adding ones to the sum: {}",
                            sum[current_block_idx]
                        )))?;
                } else {
                    // see this example:
                    // Block size: 3 bits
                    // 0 0 0|0 0 0|0 0 || 0|0 0 0|0 0 0| ...
                    // 1 Byte can consist of more than 1 block!
                    // This can be solved by taking each bit, calculating the block offset for it
                    // (0 means current block) and adding the value to the right block.

                    // how many bits are left to be added to the current block_offset
                    let mut bits_left = remainder;
                    // the block offset from the current block
                    let mut block_offset =
                        ((BYTE_SIZE - remainder) / block_length_bits).clamp(1, usize::MAX);
                    for shift in 0..BYTE_SIZE {
                        if bits_left == 0 {
                            bits_left = block_length_bits;
                            block_offset -= 1;
                        }

                        // check if the block even exists
                        if current_block_idx + block_offset < block_count {
                            let value = (*value >> shift) & 0x01;
                            sum[current_block_idx + block_offset] = sum
                                [current_block_idx + block_offset]
                                .checked_add(value as usize)
                                .ok_or(Error::Overflow(format!(
                                    "adding {value} to the sum {}",
                                    sum[current_block_idx + block_offset]
                                )))?;
                        }

                        bits_left -= 1;
                    }
                }
                Ok::<_, Error>(sum)
            },
        )
        .try_reduce(
            || vec![0_usize; block_count],
            |a, b| {
                a.into_iter()
                    .zip(b.into_iter())
                    .map(|(a, b)| {
                        a.checked_add(b).ok_or(Error::Overflow(format!(
                            "Adding two parts of the sum: {a} + {b}"
                        )))
                    })
                    .collect::<Result<Vec<_>, Error>>()
            },
        )?;

    if add_remainder {
        // add the necessary part of the remainder to the last block
        let needed_bits = (BYTE_SIZE - ((data.data.len() + 1) * BYTE_SIZE % block_length_bits))
            % block_length_bits;

        count_ones_per_block[block_count - 1] = count_ones_per_block[block_count - 1]
            .checked_add(
                data.remainder[0..needed_bits]
                    .iter()
                    .map(|&bit| bit as usize)
                    .sum(),
            )
            .ok_or(Error::Overflow(format!(
                "Adding the remainder to the sum: {}",
                count_ones_per_block[block_count - 1]
            )))?;
    }

    let pis = count_ones_per_block
        .into_iter()
        .map(|count_ones| (count_ones as f64) / (block_length_bits as f64));

    // Step 3 - compute the chi^2 statistics - calculate the values for each element in the sum
    let chi_parts = pis.map(|pi| (pi - 0.5).powi(2));

    // Step 3 - compute the chi^2 statistics - build sum and multiply with 4 * block_length
    // In Step 4, chi is again halved - do this now (replace 4 with 2)
    let half_chi = chi_parts.sum::<f64>() * 2.0 * (block_length_bits as f64);

    check_f64(half_chi)?;

    // Step 4: compute p-value = igamc(block_count / 2, chi / 2)
    let p_value = igamc(block_count as f64 / 2.0, half_chi)?;

    check_f64(p_value)?;

    let result = TestResult { p_value };
    runner.store_result(Test::FrequencyTestWithinABlock, result);
    Ok(result)
}
