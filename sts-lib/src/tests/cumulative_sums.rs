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
use crate::internals::{check_f64, erfc};
use crate::{Error, TestResult, BYTE_SIZE};
use std::f64::consts::SQRT_2;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: usize = 100;

/// Cumulative Sums Test - No. 13
///
/// See also the [module docs](crate::tests::cumulative_sums).
/// If the bit length is less than 100 bits, [Error::InvalidParameter] is raised.
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
    // Step 1: form a normalized sequence: 1 -> 1, 0 -> -1
    // Step 2: compute partial sums of subsequences of the original sequence, each starting with
    // [0] (if mode == false) or [^1] (if mode == true)
    // Step 3: compute the largest absolute value out of the partial sums
    // This is all one big operation - we don't need to save the list, we can just compare with the prev maximum.
    let max = if mode {
        // Start with last bits
        let (max, prev) = add_bit_iter(0, 0, data.remainder.iter().copied().rev());
        add_byte_iter(max, prev, data.data.iter().copied(), true).0
    } else {
        // Start from the beginning.
        let (max, prev) = add_byte_iter(0, 0, data.data.iter().copied(), false);
        add_bit_iter(max, prev, data.remainder.iter().copied()).0
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

    let sum_upper_bound = (n / z - 1) / 4 + 1;

    let sum_1 = {
        let lower_bound = (-n / z + 1) / 4;
        let z = z as f64;

        (lower_bound..sum_upper_bound)
            .map(|k| {
                let k = k as f64;
                norm(((4.0 * k + 1.0) * z) / sqrt_n) - norm(((4.0 * k - 1.0) * z) / sqrt_n)
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
                norm(((4.0 * k + 3.0) * z) / sqrt_n) - norm(((4.0 * k + 1.0) * z) / sqrt_n)
            })
            .sum::<f64>()
    };
    check_f64(sum_2)?;

    let p_value = 1.0 - sum_1 + sum_2;
    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}

/// Add the increasing cumulative sums of the bits to the state variables.
/// Returns the new state variables.
fn add_bit_iter(mut max: u64, mut prev: i64, iter: impl Iterator<Item = bool>) -> (u64, i64) {
    for bit in iter {
        // set the previous value to the current value.
        if bit {
            prev += 1;
        } else {
            prev -= 1;
        }

        // set maximum if necessary
        if max < i64::unsigned_abs(prev) {
            max = i64::unsigned_abs(prev);
        }
    }

    (max, prev)
}

/// Add the increasing cumulative sums of the bytes to the state variables.
/// Parameter rev: if the bit order should be reversed.
/// Returns the new state variables.
fn add_byte_iter<I>(mut max: u64, mut prev: i64, iter: I, rev: bool) -> (u64, i64)
where
    I: Iterator<Item = u8> + DoubleEndedIterator,
{
    #[inline]
    fn handle_byte(
        mut max: u64,
        mut prev: i64,
        byte: u8,
        shift_iter: impl Iterator<Item = usize>,
    ) -> (u64, i64) {
        shift_iter.map(|shift| 1 << shift).for_each(|mask| {
            if byte & mask != 0 {
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
        for byte in iter.rev() {
            (max, prev) = handle_byte(max, prev, byte, 0..BYTE_SIZE);
        }
    } else {
        for byte in iter {
            (max, prev) = handle_byte(max, prev, byte, (0..BYTE_SIZE).rev());
        }
    }

    (max, prev)
}

/// The standard normal cumulative distribution function.
#[inline]
fn norm(x: f64) -> f64 {
    // from https://en.wikipedia.org/wiki/Error_function#Cumulative_distribution_function
    0.5 * erfc(-x / SQRT_2)
}
