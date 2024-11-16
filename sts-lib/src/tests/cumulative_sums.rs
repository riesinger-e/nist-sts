//! The cumulative sums test.
//!
//! This test calculates cumulative partial sums of the bit sequence, once starting from the
//! first bit and once starting from the last bit, adjusting the digits to -1 and +1 and calculating
//! the maximum absolute partial sum. The test checks if this maximum is within the expected bounds
//! for random sequences.
//!
//! The input sequence should be at least 100 bits in length, smaller sequences will raise
//! [Error::InvalidParameter].

use crate::bitvec::BitVec;
use crate::internals::{check_f64, get_bit_from_value};
use crate::{Error, TestResult};
use statrs::distribution;
use statrs::distribution::ContinuousCDF;
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

/// Cumulative Sums Test - No. 13
///
/// See also the [module docs](crate::tests::cumulative_sums).
/// If the bit length is less than 100 bits, [Error::InvalidParameter] is raised.
#[use_thread_pool]
pub fn cumulative_sums_test(data: &BitVec) -> Result<[TestResult; 2], Error> {
    if data.len_bit() < 100 {
        Err(Error::InvalidParameter(format!(
            "Sequence length must be >= 100. Is: {}",
            data.len_bit()
        )))
    } else {
        Ok([
            cusum_test_internal(data, false)?,
            cusum_test_internal(data, true)?,
        ])
    }
}

/// Internal implementation of the cumulative sum test. Assumes that all constraints are met.
/// pub(crate) to allow for tests.
pub(crate) fn cusum_test_internal(data: &BitVec, mode: bool) -> Result<TestResult, Error> {
    // create a range iterator if the last value needs to be handled differently (not a full word).
    // range iterator goes from LSB to MSB by default (reverse order).
    let (full_words, last_word) = data.as_full_slice();
    let last_word = last_word.map(|w| (w, 0..(data.bit_count_last_word as usize)));

    // Step 1: form a normalized sequence: 1 -> 1, 0 -> -1
    // Step 2: compute partial sums of subsequences of the original sequence, each starting with
    // [0] (if mode == false) or [^1] (if mode == true)
    // Step 3: compute the largest absolute value out of the partial sums
    // This is all one big operation - we don't need to save the list, we can just compare with the prev maximum.
    let max = if mode {
        // Start with last bits, going in reverse
        if let Some((last_word, shifts)) = last_word {
            // if going backwards, the LSB is the first bit to watch
            let (max, prev) = handle_value(0, 0, last_word, shifts, true);

            handle_slice(max, prev, full_words, true).0
        } else {
            handle_slice(0, 0, &data.words, true).0
        }
    } else {
        // Start with first bits, normal order
        if let Some((last_word, shifts)) = last_word {
            let (max, prev) = handle_slice(0, 0, full_words, false);

            // if going forwards, the MSB is the first bit to watch
            handle_value(max, prev, last_word, shifts, false).0
        } else {
            handle_slice(0, 0, &data.words, false).0
        }
    };

    // Step 4: compute p_value = 1
    //  - sum_{k = (-n/z + 1) / 4}^{ (n/z - 1) / 4}(
    //      phi(((4k + 1) * z) / sqrt(n)) - phi(((4k - 1) * z) / sqrt(n))
    //  )
    //  + sum_{k = (-n/z - 3) / 4}^{ (n/z - 1) / 4}(
    //      phi(((4k + 3) * z) / sqrt(n)) - phi(((4k + 1) * z) / sqrt(n))
    //  )
    // where z = max, n = data.len_bit(), phi(x) = standard normal cumulative distribution function
    let z = max as i64;
    let n = data.len_bit() as i64;
    let sqrt_n = f64::sqrt(n as f64);

    let normal_distribution = distribution::Normal::standard();

    let sum_upper_bound = (n / z - 1) / 4 + 1;

    let sum_1 = {
        let lower_bound = (-n / z + 1) / 4;
        let z = z as f64;

        (lower_bound..sum_upper_bound)
            .map(|k| {
                let k = k as f64;
                normal_distribution.cdf(((4.0 * k + 1.0) * z) / sqrt_n)
                    - normal_distribution.cdf(((4.0 * k - 1.0) * z) / sqrt_n)
            })
            .sum::<f64>()
    };
    check_f64(sum_1)?;

    let sum_2 = {
        let lower_bound = (-n / z - 3) / 4;
        let z = z as f64;

        (lower_bound..sum_upper_bound)
            .map(|k| {
                let k = k as f64;
                normal_distribution.cdf(((4.0 * k + 3.0) * z) / sqrt_n)
                    - normal_distribution.cdf(((4.0 * k + 1.0) * z) / sqrt_n)
            })
            .sum::<f64>()
    };
    check_f64(sum_2)?;

    let p_value = 1.0 - sum_1 + sum_2;
    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}

/// Add the increasing cumulative sums of the bytes to the state variables.
/// Parameter rev: if the bit order should be reversed.
/// Returns the new state variables.
fn handle_slice(mut max: u64, mut prev: i64, data: &[usize], rev: bool) -> (u64, i64) {
    if rev {
        for &value in data.iter().rev() {
            (max, prev) = handle_value(max, prev, value, 0..(usize::BITS as usize), rev);
        }
    } else {
        for &value in data.iter() {
            (max, prev) = handle_value(max, prev, value, 0..(usize::BITS as usize), rev);
        }
    }

    (max, prev)
}

/// Handle an individual value:
/// shifts denotes which bits to read
#[inline]
fn handle_value(
    max: u64,
    prev: i64,
    value: usize,
    bits_to_read: Range<usize>,
    rev: bool,
) -> (u64, i64) {
    fn internal(
        mut max: u64,
        mut prev: i64,
        value: usize,
        indexes: impl Iterator<Item = usize>,
    ) -> (u64, i64) {
        indexes.for_each(|idx| {
            if get_bit_from_value(value, idx) {
                prev += 1
            } else {
                prev -= 1
            }

            // set maximum if necessary
            if max < i64::unsigned_abs(prev) {
                max = i64::unsigned_abs(prev);
            }
        });
        (max, prev)
    }

    if rev {
        // if going backwards, the LSB is the first bit to watch
        internal(max, prev, value, bits_to_read.rev())
    } else {
        // if going forward, the MSB is the first bit to watch
        internal(max, prev, value, bits_to_read)
    }
}
