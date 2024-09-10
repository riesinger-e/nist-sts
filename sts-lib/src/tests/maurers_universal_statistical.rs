//! Maurer's "Universal Statistical" Test
//!
//! This test detects if the given sequence if significantly compressible without information loss.
//! If it is, it is considered non-random.
//!
//! The recommended minimum length of the sequence is 387 840 bits. The absolute minimum length to
//! be used is 2020 bits, smaller inputs will raise an error.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, erfc};
use crate::{Error, TestResult, BYTE_SIZE};
use std::f64::consts::SQRT_2;
use std::num::NonZero;
use sts_lib_derive::use_thread_pool;

/// The minimum input length, in bits, for this test.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const { 
    match NonZero::new(2020) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

/// The expected statistic values µ. The index is the block length *L* - 1, i.e. the array is
/// defined for 1 <= *L* <= 16.
///
/// Source: "Handbook of Applied Cryptography", p. 184, table 5.3
const EXPECTED_VALUES: [f64; 16] = [
    0.7326495, 1.5374383, 2.4016068, 3.3112247, 4.2534266, 5.2177052, 6.1962507, 7.1836656,
    8.1764248, 9.1723243, 10.170032, 11.168765, 12.168070, 13.167693, 14.167488, 15.167379,
];

/// The expected statistics variances. The index is the block length *L* - 1, i.e. the array is
/// defined for 1 <= *L* <= 16.
///
/// Source: "Handbook of Applied Cryptography", p. 184, table 5.3
const VARIANCES: [f64; 16] = [
    0.690, 1.338, 1.901, 2.358, 2.705, 2.954, 3.125, 3.238,
    3.311, 3.356, 3.384, 3.401, 3.410, 3.416, 3.419, 3.421,
];

/// Maurers "Universal Statistical" Test  - No. 9
///
/// See also the [module docs](crate::tests::maurers_universal_statistical).
#[use_thread_pool(crate::internals::THREAD_POOL)]
pub fn maurers_universal_statistical_test(data: &BitVec) -> Result<TestResult, Error> {
    // Step 0: calculate which block length L is fitting and the other inputs based on that
    let data_len = data.len_bit();
    let block_length = (1..17).rev().find(|&l| {
        let min_data_len = (1010 * usize::pow(2, l as u32)) * l;
        data_len >= min_data_len
    });

    let Some(block_length) = block_length else {
        return Err(Error::InvalidParameter(format!(
            "length of data ({data_len}) is too small!"
        )));
    };

    // result should contain a warning if input size is smaller than recommended
    let result_comment = if block_length < 6 {
        Some("length of data is < 387 840!")
    } else {
        None
    };

    // based on L, calculate count of initialization blocks Q and count of test blocks K
    let count_init_blocks = 10 * usize::pow(2, block_length as u32);
    let count_test_blocks = data_len / block_length - count_init_blocks;

    // Step 2: create a table for each possible L-bit value
    let mut table = vec![0; 1 << block_length].into_boxed_slice();

    // Step 2: fill the table with the block number of the last occurrence of the pattern in the
    // init blocks.
    for block_idx in (0..count_init_blocks).rev() {
        // calculate the start byte and the bit position in the start byte for this block
        let total_start_bit =
            block_idx
                .checked_mul(block_length)
                .ok_or(Error::Overflow(format!(
                    "multiplying {block_idx} by {block_length}"
                )))?;

        let start_byte = total_start_bit / BYTE_SIZE;
        let start_bit = total_start_bit % BYTE_SIZE;

        let end_bit = start_bit + block_length - 1;
        let end_byte = start_byte + end_bit / BYTE_SIZE;

        let shift = BYTE_SIZE - end_bit % BYTE_SIZE - 1;

        // we don't need to think about the edge case that the last byte is from the additional bits
        // here, because where are test blocks after the initialization blocks
        let current_block = &data.data[start_byte..=end_byte];
        // arguments must be valid.
        let current_block = extract_usize(current_block, shift, block_length).unwrap();

        // save the block idx if it no later block was already found
        if table[current_block] == 0 {
            table[current_block] = block_idx + 1;
        }

        // if all table entries are filled, the loop can be stopped early
        if table.iter().all(|&v| v != 0) {
            break;
        }
    }

    // Step 3: examine all test blocks, for each block determine the number of blocks (distance)
    // since the last occurrence of the same block (index of the last block is stored in the table).
    // Add log2(distance) to the sum.
    // Because of the needed mutable (and ordered) access to the table, this operation cannot run parallel.
    let mut sum = 0.0;

    for block_idx in 0..count_test_blocks {
        let block_idx = block_idx + count_init_blocks;

        let total_start_bit =
            block_idx
                .checked_mul(block_length)
                .ok_or(Error::Overflow(format!(
                    "multiplying {block_idx} by {block_length}"
                )))?;

        let start_byte = total_start_bit / BYTE_SIZE;
        let start_bit = total_start_bit % BYTE_SIZE;

        let end_bit = start_bit + block_length - 1;
        let end_byte = start_byte + end_bit / BYTE_SIZE;

        let shift = BYTE_SIZE - end_bit % BYTE_SIZE - 1;

        // edge case: last byte is not stored in data.data
        let current_block = if end_byte >= data.data.len() {
            let mut current_block = Vec::from(&data.data[start_byte..]);
            let last_byte = data.get_last_byte();
            current_block.push(last_byte);
            // arguments must be valid.
            extract_usize(current_block, shift, block_length).unwrap()
        } else {
            let current_block = &data.data[start_byte..=end_byte];
            // arguments must be valid.
            extract_usize(current_block, shift, block_length).unwrap()
        };

        let last_block_idx = table[current_block];
        table[current_block] = block_idx + 1;
        sum += f64::log2((block_idx + 1 - last_block_idx) as f64);
    }

    check_f64(sum)?;

    // Step 4: compute the test statistic: f_n = sum / K .
    // K denotes the count of test blocks.
    let count_test_blocks = count_test_blocks as f64;
    let f_n = sum / count_test_blocks;
    check_f64(f_n)?;

    // Step 5: compute p_value = erfc(abs((f_n - expectedValue) / (sqrt(2) * sigma))).
    // Here, expectedValue and variance are taken from their respective tables and
    // sigma = c * sqrt(variance / K), c = 0.7 - 0.8 / L + (4 + 32 / L) * (K^(-3/L)) / 15
    let variance = VARIANCES[block_length - 1];
    let expected_value = EXPECTED_VALUES[block_length - 1];

    let block_length = block_length as f64;
    let c = 0.7 - (0.8 / block_length)
        + (4.0 + 32.0 / block_length) * (f64::powf(count_test_blocks, -3.0 / block_length) / 15.0);
    let sigma = c * f64::sqrt(variance / count_test_blocks);

    let p_value = erfc(f64::abs((f_n - expected_value) / (SQRT_2 * sigma)));
    check_f64(p_value)?;

    Ok(TestResult {
        p_value,
        comment: result_comment,
    })
}

/// Extract a big-endian usize value contained somewhere in the given byte vector.
///
/// 1. The byte list may not contain more than 4 bytes.
/// 2. The necessary right shift may not be more than 7 bits.
/// 3. The bit length of the value itself may be max. 32 - shift bits.
///
/// The output is of type usize to be used in indexing - either 32 or 64 bits.
fn extract_usize(bytes: impl AsRef<[u8]>, shift: usize, value_length_bits: usize) -> Option<usize> {
    let bytes = bytes.as_ref();
    if shift >= BYTE_SIZE || bytes.len() > 4 || bytes.is_empty() {
        return None;
    }

    let bytes = match bytes.len() {
        1 => [0, 0, 0, bytes[0]],
        2 => [0, 0, bytes[0], bytes[1]],
        3 => [0, bytes[0], bytes[1], bytes[2]],
        4 => [bytes[0], bytes[1], bytes[2], bytes[3]],
        _ => unreachable!(),
    };

    // create an u32 from the array - big endian because it is read from the sequence
    let not_shifted_value = u32::from_be_bytes(bytes);

    // shift
    let shifted_value = not_shifted_value >> shift;

    // create the mask & apply it
    let mask = (1 << value_length_bits) - 1;

    // on all (known to me) platforms, usize is either 32 bit or 64 bit.
    Some((shifted_value & mask) as usize)
}
