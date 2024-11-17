//! Test for the Longest Run of Ones in a Block
//!
//! This test determines whether the longest run (See [runs_test](super::runs)) of ones
//! in a block is consistent with the expected value for a random sequence.
//!
//! An irregularity in the length of longest run of ones also implies an irregularity in the length
//! of the longest runs of zeroes, meaning that only this test is necessary. See the NIST publication.
//!
//! The data has to be at least 128 bits in length.
//!
//! The probability constants were recalculated, so you might see a deviation when comparing the
//! output with the reference implementation. In testing, the deviations were not too big.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc, BitPrimitive};
use crate::{Error, TestResult};
use rayon::prelude::*;
use std::num::NonZero;
use sts_lib_derive::use_thread_pool;
use crate::bitvec::chunks::Chunk;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(128) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

// Table sorting criteria for the three possible block lengths.
const TABLE_SORTING_CRITERIA_8: [usize; 4] = [1, 2, 3, 4];
const TABLE_SORTING_CRITERIA_128: [usize; 6] = [4, 5, 6, 7, 8, 9];
const TABLE_SORTING_CRITERIA_10_4: [usize; 7] = [10, 11, 12, 13, 14, 15, 16];

// probabilities from section 3.4, but recalculated with `longest_runs_of_ones_in_a_block.py` for
// more accuracy (4 decimal places are not very much).
const PROBABILITIES_8: [f64; 4] = [0.21484375, 0.3671875, 0.23046875, 0.1875];
const PROBABILITIES_128: [f64; 6] = [
    0.11740357883779325,
    0.2429559592774549,
    0.2493634831790783,
    0.17517706034678193,
    0.10270107130405359,
    0.11239884705483805,
];
const PROBABILITIES_10_4: [f64; 7] = [
    0.08663231107995277,
    0.2082006483876035,
    0.24841858194169963,
    0.1939127867416558,
    0.12145848508900658,
    0.06801108930393818,
    0.07336609745614353,
];

/// Test for the longest run of ones in a block - No. 4
///
/// See the [module docs](crate::tests::longest_run_of_ones)
#[use_thread_pool]
pub fn longest_run_of_ones_test(data: &BitVec) -> Result<TestResult, Error> {
    // Step 0: determine the block length and the block count, based on 2.4.2.
    // Also determine the values bucket_count (= K + 1) and n, as given 2.4.4
    // All possible values are whole bytes.
    match data.len_bit() {
        bit_len @ 0..=127 => Err(Error::InvalidParameter(format!(
            "Input length has to be at least 128 bits, is {}",
            bit_len
        ))),
        128..=6271 => {
            let data = data.par_chunks_exact(8 / (u8::BITS as usize));
            
            longest_run_of_ones_imp(data, TABLE_SORTING_CRITERIA_8, PROBABILITIES_8)
        }
        6272..=749_999 => {
            let data = data.par_chunks_exact(128 / (u8::BITS as usize));

            longest_run_of_ones_imp(data, TABLE_SORTING_CRITERIA_128, PROBABILITIES_128)
        }
        750_000.. => {
            let data = data.par_chunks_exact(10_000 / (u8::BITS as usize));

            longest_run_of_ones_imp(data, TABLE_SORTING_CRITERIA_10_4, PROBABILITIES_10_4)
        }
    }
}

/// The real implementation. Bucket count is decided based on the block size.
/// Each chunk must have exactly block_size bits.
fn longest_run_of_ones_imp<'a, const BUCKET_COUNT: usize>(
    data: impl IndexedParallelIterator<Item = Chunk<'a>>,
    table_criteria: [usize; BUCKET_COUNT],
    probabilities: [f64; BUCKET_COUNT],
) -> Result<TestResult, Error> {
    let block_count = data.len();

    // Step 1: divide the sequence into blocks
    // Step 2: Calculate the length of the longest run per block and sort it into a table based on its length.
    // Since block_count should always be higher than block_length, the outer loop is parallel here.
    let run_table = data
        .try_fold(
            || [0_usize; BUCKET_COUNT],
            |mut table, chunk| {
                // only runs of 1 are relevant here
                let mut current_run_length: usize = 0;
                let mut max_run_length: usize = 0;

                handle_chunk_part(&chunk.start, &mut current_run_length, &mut max_run_length)?;
                handle_chunk_part(chunk.middle, &mut current_run_length, &mut max_run_length)?;
                handle_chunk_part(&chunk.end, &mut current_run_length, &mut max_run_length)?;

                // for the last bit
                if current_run_length > max_run_length {
                    max_run_length = current_run_length;
                }

                add_run_to_table(&mut table, &table_criteria, max_run_length)?;
                Ok(table)
            },
        )
        .try_reduce(
            || [0_usize; BUCKET_COUNT],
            |mut a, b| -> Result<_, Error> {
                a.iter_mut()
                    .zip(b.into_iter())
                    .try_for_each(|(a, b)| -> Result<(), Error> {
                        *a = a
                            .checked_add(b)
                            .ok_or(Error::Overflow(format!("Adding run part sums {a} and {b}")))?;
                        Ok(())
                    })?;
                Ok(a)
            },
        )?;

    // Step 3: compute chi = sum of ( (v - n * pi_i)^2 / (n * pi_i) ) for each entry v in the run table.
    // The values of pi_i are provided in section 3.4 and were recalculated (4 decimal places is
    // very likely too much rounding).
    // Here block_count is taken for n = N.
    let chi = (0..BUCKET_COUNT)
        .map(|idx| {
            f64::powi(
                (run_table[idx] as f64) - (block_count as f64) * probabilities[idx],
                2,
            ) / ((block_count as f64) * probabilities[idx])
        })
        .sum::<f64>();

    check_f64(chi)?;

    // Step 4: compute p_value = igamc(K / 2, chi / 2)
    let param1 = ((BUCKET_COUNT - 1) as f64) / 2.0;
    check_f64(param1)?;
    let param2 = chi / 2.0;
    check_f64(param2)?;
    let p_value = igamc(param1, param2)?;
    check_f64(p_value)?;
    Ok(TestResult::new(p_value))
}

/// Handles a part of the current Chunk: start, middle, or end. Calculates the current run length
/// and saves the max run length if it changes.
fn handle_chunk_part<T: BitPrimitive>(
    chunk: &[T],
    current_run_length: &mut usize,
    max_run_length: &mut usize,
) -> Result<(), Error> {
    for unit in chunk {
        if unit.count_ones() == T::BITS {
            // easy case: all ones
            *current_run_length =
                current_run_length
                    .checked_add(T::BITS as usize)
                    .ok_or(Error::Overflow(format!(
                        "adding {} to run length {current_run_length}",
                        T::BITS
                    )))?;
        } else {
            // we have to inspect bit by bit
            for idx in 0..T::BITS {
                if unit.get_bit(idx) {
                    *current_run_length =
                        current_run_length
                            .checked_add(1)
                            .ok_or(Error::Overflow(format!(
                                "adding 1 to run length {current_run_length}"
                            )))?;
                } else {
                    // run of ones ended here
                    if current_run_length > max_run_length {
                        *max_run_length = *current_run_length;
                    }
                    *current_run_length = 0;
                }
            }
        }
    }

    Ok(())
}

/// to sort a given run length into the run table described in 2.4.4 (2)
fn add_run_to_table<const BUCKET_COUNT: usize>(
    table: &mut [usize; BUCKET_COUNT],
    criteria: &[usize; BUCKET_COUNT],
    run_length: usize,
) -> Result<(), Error> {
    // length is at least 4 (table is one of three constants)
    let last_idx = BUCKET_COUNT - 1;

    // first and last element need different comparisons
    if run_length <= criteria[0] {
        table[0] = table[0].checked_add(1).ok_or(Error::Overflow(format!(
            "adding 1 to table value {}",
            table[0]
        )))?;
    } else if run_length >= criteria[last_idx] {
        table[last_idx] = table[last_idx]
            .checked_add(1)
            .ok_or(Error::Overflow(format!(
                "adding 1 to table value {}",
                table[last_idx]
            )))?;
    } else {
        // this is an index in the middle - iterate over every criterion except first and last
        for i in 1..last_idx {
            if run_length == criteria[i] {
                table[i] = table[i].checked_add(1).ok_or(Error::Overflow(format!(
                    "adding 1 to table value {}",
                    table[i]
                )))?;
                break;
            }
        }
    }

    Ok(())
}
