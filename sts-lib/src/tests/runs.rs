//! Runs test.
//!
//! This tests focuses on the number of runs in the sequence. A run is an uninterrupted sequence of
//! identical bits.
//! Each tested [BitVec] should have at least 100 bits length.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, erfc, get_bit_from_value};
use crate::{Error, TestResult};
use rayon::prelude::*;
use std::num::NonZero;
use std::ops::Range;
use sts_lib_derive::use_thread_pool;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(100) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

/// Runs test - No. 3
///
/// See the [module docs](crate::tests::runs).
/// If an error happens, it means either arithmetic underflow or overflow - beware.
#[use_thread_pool]
pub fn runs_test(data: &BitVec) -> Result<TestResult, Error> {
    // Step 1: calculate pi = count of ones / length of data
    let count_ones = data
        .words
        .par_iter()
        .try_fold(
            || 0_usize,
            |sum, value| {
                sum.checked_add(value.count_ones() as usize)
                    .ok_or(Error::Overflow(format!(
                        "adding the ones in the current byte to sum {sum}"
                    )))
            },
        )
        .try_reduce(
            || 0_usize,
            |a, b| {
                a.checked_add(b)
                    .ok_or(Error::Overflow(format!("adding the part-sums {a} and {b}")))
            },
        )?;
    // don't need to check if the last word was incomplete - we only care about 1, the empty bits
    // in the last word are always zero.
    let pi = (count_ones as f64) / (data.len_bit() as f64);

    // Step 2: determine if the frequency test passed: abs(pi - 1/2) < 2 / sqrt(len_bit) has to uphold.
    // Otherwise, the test should not run because the frequency test would not pass.
    if f64::abs(pi - 0.5) >= 2.0 / f64::sqrt(data.len_bit() as f64) {
        // Frequency test would fail, don't run the test
        return Ok(TestResult::new_with_comment(
            0.0,
            "Frequency test would not pass!",
        ));
    }

    // Step 3: compute the statistic V = (sum of r(k) for data[1..] - index k) + 1
    //  where r(k) = 0 if data[k] == data[k-1], else 1.
    let (full_units, last_unit) = data.as_full_slice();
    let v = calc_v_data_for_slice(full_units)?;

    let v = if let Some(unit) = last_unit {
        let bit_count = data.bit_count_last_word as usize;

        let v_rem = if let Some(&full_unit) = full_units.last() {
            // if full_units contained data, take the last bit of it
            let prev_value = full_unit & 0x01;

            calc_v_data_for_unit(unit, 0..bit_count, prev_value == 1)?
        } else {
            // need to take the first bit of the last word
            let prev_value = (unit >> (usize::BITS - 1)) & 0x1;
            calc_v_data_for_unit(unit, 1..bit_count, prev_value == 1)?
        };

        v + v_rem
    } else {
        v
    };

    let v = v
        .checked_add(1)
        .ok_or(Error::Overflow(format!("adding 1 to v {v}")))?;

    // Step 4: compute p_value = erfc( abs(v - 2*bit_len*pi*(1-pi)) / (2*sqrt(2*bit_len)*pi*(1-pi)) )
    let numerator = f64::abs((v as f64) - 2.0 * (data.len_bit() as f64) * pi * (1.0 - pi));
    check_f64(numerator)?;
    let denominator = 2.0 * f64::sqrt(2.0 * (data.len_bit() as f64)) * pi * (1.0 - pi);
    check_f64(denominator)?;
    let fraction = numerator / denominator;
    check_f64(fraction)?;
    let p_value = erfc(fraction);
    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}

/// Calculation of v statistic for the data array.
fn calc_v_data_for_slice(data: &[usize]) -> Result<usize, Error> {
    const BITS: usize = usize::BITS as usize;
    
    // guard clause
    if data.is_empty() {
        return Ok(0);
    }

    // Special casing for the first byte
    let v_first_word = {
        // prev_value = first bit
        let prev_value = get_bit_from_value(data[0], 0);

        // for the remaining bits, just get them and compare them to the previous bit to get the value
        calc_v_data_for_unit(data[0], 1..BITS, prev_value)?
    };

    // remaining bytes (every byte except first)
    let v_rem_words = data[1..]
        .par_iter()
        .enumerate()
        .try_fold(
            || 0_usize,
            |sum, (prev_idx, &word)| {
                // start with last bit of previous byte
                // prev_byte_idx is of the previous byte because we start with index 1 --> 0 in
                // the iterator
                let prev_value = get_bit_from_value(data[prev_idx], BITS - 1);

                calc_v_data_for_unit(word, 0..BITS, prev_value)?
                    .checked_add(sum)
                    .ok_or(Error::Overflow(format!("Adding byte sum to sum {sum}")))
            },
        )
        .try_reduce(
            || 0_usize,
            |a, b| {
                a.checked_add(b)
                    .ok_or(Error::Overflow(format!("adding sum {a} to {b}")))
            },
        )?;

    v_first_word
        .checked_add(v_rem_words)
        .ok_or(Error::Overflow(format!(
            "adding first byte sum {v_first_word} to rem byte sum {v_rem_words}"
        )))
}

/// Calculate v for a single byte
fn calc_v_data_for_unit(
    value: usize,
    mut bits: Range<usize>,
    mut prev_bit: bool,
) -> Result<usize, Error> {    
    bits.try_fold(0_usize, |sum, bit_idx| {
        let current_bit = get_bit_from_value(value, bit_idx);
        let res = if current_bit == prev_bit {
            Ok(sum)
        } else {
            sum.checked_add(1)
                .ok_or(Error::Overflow(format!("adding 1 to byte sum {sum}")))
        };
        prev_bit = current_bit;
        res
    })
}
