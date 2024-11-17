//! Frequency Test within a block.
//!
//! This tests for the same property as [crate::frequency_test], but within M-bit blocks.
//! It is recommended that each block  has a length of at least 100 bits.
//! This test needs an argument, see [FrequencyBlockTestArg].

use crate::bitvec::BitVec;
use crate::internals::{check_f64, get_bit_from_value, igamc};
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

/// The argument for the Frequency test within a block: the block length.
///
/// The block length should be at least 20 bits, with the block length greater than 1% of the
/// total bit length and fewer than 100 total blocks.
#[derive(Copy, Clone, Default, Debug)]
pub enum FrequencyBlockTestArg {
    /// Manual block length
    Manual(NonZero<usize>),
    /// A suitable block length will be chosen automatically, based on the criteria outlined in
    /// [FrequencyBlockTestArg].
    #[default]
    ChooseAutomatically,
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
    const BITS: usize = usize::BITS as usize;

    // Step 0 - get the block length or calculate one
    let block_length = match test_arg {
        FrequencyBlockTestArg::Manual(block_length) => block_length.get(),
        FrequencyBlockTestArg::ChooseAutomatically => choose_block_length(data.len_bit()),
    };

    // Step 1 - calculate the amount of blocks
    let block_count = data.len_bit() / block_length;

    // Step 2 - calculate pi_i = (ones in the block) / block_length for each block.
    // We can't split in chunks here, because chunks would only catch whole words.

    // How many words are needed - there could be unused words at the end
    let words_needed = if block_length * block_count % BITS == 0 {
        // no remainder
        block_length * block_count / BITS
    } else {
        // a remainder is left: 1 additional byte is needed for it.
        block_length * block_count / BITS + 1
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
            // returns the block idx for the specified bit idx
            let block_idx = |bit_idx: usize| (idx * BITS + bit_idx) / block_length;

            if block_idx(0) == block_count {
                return;
            }

            if block_idx(0) == block_idx(BITS - 1) {
                // the whole word is the same block.
                count_ones_per_block[block_idx(0)]
                    .fetch_add(value.count_ones() as usize, Ordering::Relaxed);
            } else {
                // have to go bit by bit
                for bit_idx in 0..BITS {
                    let block_idx = block_idx(bit_idx);
                    if block_idx == block_count {
                        break;
                    }

                    if get_bit_from_value(*value, bit_idx) {
                        count_ones_per_block[block_idx].fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });

    let pis = Box::into_iter(count_ones_per_block).map(|count_ones| {
        let count_ones = count_ones.into_inner();
        (count_ones as f64) / (block_length as f64)
    });

    // Step 3 - compute the chi^2 statistics - calculate the values for each element in the sum
    let chi_parts = pis.map(|pi| (pi - 0.5).powi(2));

    // Step 3 - compute the chi^2 statistics - build sum and multiply with 4 * block_length
    // In Step 4, chi is again halved - do this now (replace 4 with 2)
    let half_chi = chi_parts.sum::<f64>() * 2.0 * (block_length as f64);

    check_f64(half_chi)?;

    // Step 4: compute p-value = igamc(block_count / 2, chi / 2)
    let p_value = igamc(block_count as f64 / 2.0, half_chi)?;

    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}

/// Choose a block length based on 2.2.7. Needs the amount of bits as the parameter. If possible,
/// it chooses usize-aligned blocks.
fn choose_block_length(length: usize) -> usize {
    const BITS: usize = usize::BITS as usize;
    const MIN_BLOCK_LENGTH: usize = 20;

    // Start with the recommended minimum block length based on the length of the data.
    // This also satisfies that less than 100 block should exist.
    let block_length = length / 100 + 1;

    if block_length < MIN_BLOCK_LENGTH {
        MIN_BLOCK_LENGTH
    } else {
        // Round up to the next block length that is usize-aligned.
        // This works by adding 63 and than truncating the lower bits.
        let ideal_block_length = (block_length + BITS - 1) & !(BITS - 1);

        // the ideal block length is possible as long as there are at least 2 blocks.
        // 1 block would just be the frequency test.
        if ideal_block_length * 2 <= length {
            ideal_block_length
        } else {
            block_length
        }
    }
}
