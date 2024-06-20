//! Test for the Longest Run of Ones in a Block
//!
//! This test determines whether the longest run (See [runs_test](super::runs)) of ones
//! in a block is consistent with the expected value for a random sequence.
//!
//! An irregularity in the length of longest run of ones also implies an irregularity in the length
//! of the longest runs of zeroes, meaning that only this test is necessary. See the NIST publication.
//!
//! The data has to be at least 128 bits in length.

use crate::bitvec::BitVec;
use crate::{Error, TestResult, BYTE_SIZE};
use rayon::prelude::*;
use crate::internals::{check_f64, igamc};

// Table sorting criteria for the three possible block lengths.
const TABLE_SORTING_CRITERIA_8: [usize; 4] = [1, 2, 3, 4];
const TABLE_SORTING_CRITERIA_128: [usize; 6] = [4, 5, 6, 7, 8, 9];
const TABLE_SORTING_CRITERIA_10_4: [usize; 7] = [10, 11, 12, 13, 14, 15, 16];

//TODO: recalculate
// probabilities from section 3.4, but recalculated with longest_runs_of_ones_in_a_block.py for
// more accuracy (4 decimal places are not very much).
const PROBABILITIES_8: [f64; 4] = [0.2148, 0.3672, 0.2305, 0.1875];
const PROBABILITIES_128: [f64; 6] = [0.1174, 0.2430, 0.2493, 0.1752, 0.1027, 0.1124];
const PROBABILITIES_10_4: [f64; 7] = [0.0882, 0.2092, 0.2483, 0.1933, 0.1208, 0.0675, 0.0727];

/// Test for the longest run of ones in a block - No. 4
///
/// See the [module docs](crate::tests::longest_run_of_ones)
pub fn longest_run_of_ones_test(data: &BitVec) -> Result<TestResult, Error> {
    // Step 0: determine the block length and the block count, based on 2.4.2.
    // Also determine the values bucket_count (= K + 1) and n, as given 2.4.4
    // All possible values are whole bytes.
    let (block_length_bits, bucket_count, table_criteria, probabilities) = match data.len_bit() {
        0..=127 => return Ok(TestResult::new_with_comment(0.0, "Test data is too short!")),
        128..=6271 => (
            8,
            4,
            TABLE_SORTING_CRITERIA_8.as_slice(),
            PROBABILITIES_8.as_slice(),
        ),
        6272..=749_999 => (
            128,
            6,
            TABLE_SORTING_CRITERIA_128.as_slice(),
            PROBABILITIES_128.as_slice(),
        ),
        750_000.. => (
            10_000,
            7,
            TABLE_SORTING_CRITERIA_10_4.as_slice(),
            PROBABILITIES_10_4.as_slice(),
        ),
    };
    let block_length_bytes = block_length_bits / BYTE_SIZE;
    let block_count = data.data.len() / block_length_bytes;

    // Step 1: divide the sequence into blocks
    // Step 2: Calculate the length of the longest run per block and sort it into a table based on its length.
    // Since block_count should always be higher than block_length, the outer loop is parallel here.
    let run_table = data
        .data
        .par_chunks_exact(block_length_bytes)
        .try_fold(
            || vec![0_usize; bucket_count],
            |table, chunk| {
                // only runs of 1 are relevant here
                let mut current_run_length: usize = 0;
                let mut max_run_length: usize = 0;

                for &byte in chunk {
                    if byte.count_ones() as usize == BYTE_SIZE {
                        // easy case: all ones
                        current_run_length =
                            current_run_length
                                .checked_add(BYTE_SIZE)
                                .ok_or(Error::Overflow(format!(
                                    "adding {BYTE_SIZE} to run length {current_run_length}"
                                )))?;
                    } else {
                        // we have to inspect bit by bit
                        for shift in (0..BYTE_SIZE).rev() {
                            let bit = (byte >> shift) & 0x01;
                            if bit == 1 {
                                current_run_length =
                                    current_run_length.checked_add(1).ok_or(Error::Overflow(
                                        format!("adding 1 to run length {current_run_length}"),
                                    ))?;
                            } else {
                                // run of ones ended here
                                if current_run_length > max_run_length {
                                    max_run_length = current_run_length;
                                }
                                current_run_length = 0;
                            }
                        }
                    }
                }

                // for the last bit
                if current_run_length > max_run_length {
                    max_run_length = current_run_length;
                }

                println!("Byte: {:b}: {}", chunk[0], max_run_length);
                add_run_to_table(table, table_criteria, max_run_length)
            },
        )
        .try_reduce(
            || vec![0_usize; bucket_count],
            |a, b| {
                println!("{a:?}, {b:?}");
                a.into_iter()
                    .zip(b.into_iter())
                    .map(|(a, b)| {
                        a.checked_add(b)
                            .ok_or(Error::Overflow(format!("Adding run part sums {a} and {b}")))
                    })
                    .collect::<Result<Vec<_>, _>>()
            },
        )?;

    // Step 3: compute chi = sum of ( (v - n * pi_i)^2 / (n * pi_i) ) for each entry v in the run table.
    // The values of pi_i are provided in section 3.4 and were recalculated (4 decimal places is
    // very likely too much rounding).
    // Here block_count is taken for n = N.
    let chi = (0..bucket_count)
        .map(|idx| f64::powi((run_table[idx] as f64) - (block_count as f64) * probabilities[idx], 2) / ((block_count as f64) * probabilities[idx]))
        .sum::<f64>();

    check_f64(chi)?;

    // Step 4: compute p_value = igamc(K / 2, chi / 2)
    let param1 = ((bucket_count - 1) as f64) / 2.0;
    check_f64(param1)?;
    let param2 = chi / 2.0;
    check_f64(param2)?;
    let p_value = igamc(param1, param2)?;
    check_f64(p_value)?;
    Ok(TestResult::new(p_value))
}

/// to sort a given run length into the run table described in 2.4.4 (2)
fn add_run_to_table(
    mut table: Vec<usize>,
    criteria: &[usize],
    run_length: usize,
) -> Result<Vec<usize>, Error> {
    // length is at least 4 (table is one of three constants)
    let last_idx = criteria.len() - 1;

    // first and last element need different comparisons
    if run_length <= criteria[0] {
        table[0] = table[0].checked_add(1).ok_or(Error::Overflow(format!(
            "adding 1 to table value {}",
            table[0]
        )))?;
    } else if run_length >= criteria[last_idx] {
        table[last_idx] = table[last_idx].checked_add(1).ok_or(Error::Overflow(format!(
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

    Ok(table)
}
