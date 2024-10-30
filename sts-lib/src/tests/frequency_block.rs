//! Frequency Test within a block.
//!
//! This tests for the same property as [crate::frequency_test], but within M-bit blocks.
//! It is recommended that each block  has a length of at least 100 bits.
//! This test needs an argument, see [FrequencyBlockTestArg].

use crate::bitvec::iter::BitVecIntoIter;
use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::{Error, TestResult};
use rayon::prelude::*;
use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};
use sts_lib_derive::use_thread_pool;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(100) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

// ratio of usize to byte
const WORD_BYTE_RATIO: usize = (usize::BITS / u8::BITS) as usize;

/// The argument for the Frequency test within a block: the block length.
///
/// The block length should be at least 20 bits, with the block length greater than 1% of the
/// total bit length and fewer than 100 total blocks.
#[derive(Copy, Clone, Default, Debug)]
pub enum FrequencyBlockTestArg {
    /// The block length is measured in bytes - this allows for faster performance.
    Bytewise(NonZero<usize>),
    /// Bitwise block length
    Bitwise(NonZero<usize>),
    /// A suitable block length will be chosen automatically, based on the criteria outlined in
    /// [FrequencyBlockTestArg].
    #[default]
    ChooseAutomatically,
}

impl FrequencyBlockTestArg {
    /// Creates a new block length argument for the test. The passed block length is in bits.
    /// If block_length is a multiple of 8 (bits in a byte), [Self::Bytewise] is automatically
    /// chosen. Else, [Self::Bitwise] is chosen.
    pub fn new(block_length: NonZero<usize>) -> Self {
        if block_length.get() % (u8::BITS as usize) == 0 {
            // For this value to be 0, block_length has to be 0, so it can't be, so unwrapping is
            // no problem.
            // Because: 0 % 8 == 0 - 8 % 8 == 0 but 1 % 8 == 1
            let value = NonZero::new(block_length.get() / (u8::BITS as usize)).unwrap();
            Self::Bytewise(value)
        } else {
            Self::Bitwise(block_length)
        }
    }
}

/// Frequency test within a block - No. 2
///
/// See the [module docs](crate::tests::frequency_block_test).
/// If test_arg is [FrequencyBlockTestArg::ChooseAutomatically], a reasonable default, based on 2.2.7, is chosen.
/// If an error happens, it means either arithmetic underflow or overflow - beware.
#[use_thread_pool]
pub fn frequency_block_test(
    data: &BitVec,
    test_arg: FrequencyBlockTestArg,
) -> Result<TestResult, Error> {
    // Step 0 - get the block length or calculate one
    let block_length = match test_arg {
        FrequencyBlockTestArg::Bytewise(block_length) => block_length.get(),
        FrequencyBlockTestArg::Bitwise(block_length) => {
            return frequency_block_test_bits(data, block_length.get())
        }
        FrequencyBlockTestArg::ChooseAutomatically => {
            let byte_length = BitVecIntoIter::<u8>::iter(data).len();
            choose_block_length(byte_length)
        }
    };

    if block_length % WORD_BYTE_RATIO == 0 {
        frequency_block_test_unit::<usize, _>(data, block_length / WORD_BYTE_RATIO)
    } else {
        frequency_block_test_unit::<u8, _>(data, block_length)
    }
}

// Trait to make the test generic over words vs. bytes
trait CountOnes: Copy + Clone + Send + Sync {
    const BITS: usize;

    fn count_ones(&self) -> usize;
}

impl CountOnes for u8 {
    const BITS: usize = u8::BITS as usize;

    fn count_ones(&self) -> usize {
        u8::count_ones(*self) as usize
    }
}

impl CountOnes for usize {
    const BITS: usize = usize::BITS as usize;

    fn count_ones(&self) -> usize {
        usize::count_ones(*self) as usize
    }
}

/// Frequency test within a block, optimization for block sizes that are whole primitive types.
///
/// The passed block_length has to in bytes.
fn frequency_block_test_unit<T, D>(data: &D, block_length_unit: usize) -> Result<TestResult, Error>
where
    T: CountOnes,
    D: BitVecIntoIter<T>,
{
    // Step 1 - calculate the amount of blocks
    let block_count = data.iter().len() / block_length_unit;
    let required_units = block_length_unit * block_count;

    // "magic" length where parallel runtime may be faster than serial.
    // Just tried different input lengths on the developers machine.
    let half_chi = if required_units >= 1_600_000 {
        frequency_block_test_par(
            data.par_iter().take(required_units),
            block_length_unit,
            block_count,
        )
    } else {
        frequency_block_test_serial(data.iter().take(required_units), block_length_unit)
    }?;

    // Step 4: compute p-value = igamc(block_count / 2, chi / 2)
    let p_value = igamc(block_count as f64 / 2.0, half_chi)?;

    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}

/// Calculate half_chi for a block length that is a multiple of whole T, without parallelization.
/// This is faster with smaller sequences, but slower with longer sequences.
/// See also [frequency_block_test_par].
fn frequency_block_test_serial<T, I>(data: I, block_length_unit: usize) -> Result<f64, Error>
where
    T: CountOnes,
    I: Iterator<Item = T>,
{
    let block_length_bits = block_length_unit * T::BITS;

    // Step 2 - calculate pi_i = (ones in the block) / block_length for each block
    // Step 3 - calculate half_chi
    let mut current_block_idx = 0;
    let mut current_count_of_ones: usize = 0;
    let mut half_chi = 0.0;

    // Result of benchmarking: parallelization is not worth it
    data.enumerate().try_for_each(|(idx, unit)| {
        let block_idx = idx / block_length_unit;

        if current_block_idx != block_idx {
            // remove old results
            let count_ones = current_count_of_ones as f64;
            let pi = count_ones / (block_length_bits as f64);
            // Step 3 - compute the chi^2 statistics - calculate the current part
            half_chi += (pi - 0.5).powi(2);

            current_block_idx = block_idx;
            current_count_of_ones = 0;
        }

        current_count_of_ones = current_count_of_ones
            .checked_add(unit.count_ones())
            .ok_or(Error::Overflow("adding ones to a block count".to_owned()))?;
        Ok::<_, Error>(())
    })?;

    // Step 2 - calculate pi_i = (ones in the block) / block_length for each block
    let count_ones = current_count_of_ones as f64;
    let pi = count_ones / (block_length_bits as f64);
    half_chi += (pi - 0.5).powi(2);

    // Calculate the half_chi (multiplication with 4.0 replaced by 2.0)
    let half_chi = half_chi * 2.0 * (block_length_bits as f64);

    check_f64(half_chi)?;

    Ok(half_chi)
}

/// Calculate half_chi for a block length that is a multiple of whole T, with parallelization.
/// This is faster with longer sequences, but slower with smaller sequences.
/// See also [frequency_block_test_serial].
fn frequency_block_test_par<T, I>(
    data: I,
    block_length_unit: usize,
    block_count: usize,
) -> Result<f64, Error>
where
    T: CountOnes,
    I: IndexedParallelIterator<Item = T>,
{
    let block_length_bits = block_length_unit * T::BITS;

    // Step 2 - calculate the count of ones in the block for each block
    let count_of_ones = {
        let mut vec = Vec::with_capacity(block_count);
        vec.resize_with(block_count, || AtomicUsize::new(0));
        vec.into_boxed_slice()
    };

    data.enumerate().try_for_each(|(idx, byte)| {
        let block_idx = idx / block_length_unit;

        let prev = count_of_ones[block_idx].fetch_add(byte.count_ones(), Ordering::Relaxed);
        if prev == usize::MAX {
            Err(Error::Overflow("adding ones to a block count".to_owned()))
        } else {
            Ok(())
        }
    })?;

    // Step 2 - calculate pi_i = (ones in the block) / block_length for each block
    // Step 3 - calculate half_chi
    let half_chi = Box::into_iter(count_of_ones)
        .map(|count_of_ones| {
            let count_ones = count_of_ones.into_inner() as f64;
            let pi = count_ones / (block_length_bits as f64);
            (pi - 0.5).powi(2)
        })
        .sum::<f64>()
        * 2.0
        * (block_length_bits as f64);

    check_f64(half_chi)?;

    Ok(half_chi)
}

/// Choose a block length based on 2.2.7. Needs the amount of bytes (each byte contains 8 values)
/// as the parameter. This method can only choose byte-sized blocks. If possible, it chooses usize-
/// size blocks.
fn choose_block_length(byte_length: usize) -> usize {
    const MIN_BLOCK_LENGTH: usize = 20 / (u8::BITS as usize) + 1;

    if byte_length >= 80 * WORD_BYTE_RATIO {
        // use usize-based block length. 80 is chosen here, because it means we will have at least 10 blocks.
        // 80 * (64 / 8) = 80 * 8 = 640 --> 10 64-bit values
        let word_length = byte_length / (usize::BITS as usize);
        (word_length / 100 + 1) * WORD_BYTE_RATIO
    } else {
        // Start with the recommended minimum block length based on the length of the data.
        // This also satisfies that less than 100 block should exist.
        let block_length = byte_length / 100 + 1;

        if block_length < MIN_BLOCK_LENGTH {
            // has to be at least the min block length
            MIN_BLOCK_LENGTH
        } else {
            block_length
        }
    }
}

/// Frequency test within a block for bit lengths that are not byte-sized.
fn frequency_block_test_bits(data: &BitVec, block_length_bits: usize) -> Result<TestResult, Error> {
    const WORD_SIZE: usize = usize::BITS as usize;

    // Step 1 - calculate the amount of blocks
    let block_count = data.len_bit() / block_length_bits;

    // Step 2 - calculate pi_i = (ones in the block) / block_length for each block.
    // We can't split in chunks here, because chunks would only catch whole bytes.

    // How many bytes are needed - there could be unused bytes at the end
    let words_needed = if block_length_bits * block_count % WORD_SIZE == 0 {
        // no remainder
        block_length_bits * block_count / WORD_SIZE
    } else {
        // a remainder is left: 1 additional byte is needed for it.
        block_length_bits * block_count / WORD_SIZE + 1
    };

    let count_ones_per_block = {
        let mut vec = Vec::with_capacity(block_count);
        vec.resize_with(block_count, || AtomicUsize::new(0));
        vec.into_boxed_slice()
    };

    data.words[0..words_needed]
        .par_iter()
        .enumerate()
        .for_each(|(idx, value)| {
            for bit_idx in 0..WORD_SIZE {
                let block_idx = (idx * WORD_SIZE + bit_idx) / block_length_bits;

                if block_idx == block_count {
                    break;
                }

                let bit = value >> (WORD_SIZE - bit_idx - 1) & 1;

                if bit == 1 {
                    count_ones_per_block[block_idx].fetch_add(1, Ordering::Relaxed);
                }
            }
        });

    let pis = Box::into_iter(count_ones_per_block).map(|count_ones| {
        let count_ones = count_ones.into_inner();
        (count_ones as f64) / (block_length_bits as f64)
    });

    // Step 3 - compute the chi^2 statistics - calculate the values for each element in the sum
    let chi_parts = pis.map(|pi| (pi - 0.5).powi(2));

    // Step 3 - compute the chi^2 statistics - build sum and multiply with 4 * block_length
    // In Step 4, chi is again halved - do this now (replace 4 with 2)
    let half_chi = chi_parts.sum::<f64>() * 2.0 * (block_length_bits as f64);

    check_f64(half_chi)?;

    // Step 4: compute p-value = igamc(block_count / 2, chi / 2)
    let p_value = igamc(block_count as f64 / 2.0, half_chi)?;

    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}
