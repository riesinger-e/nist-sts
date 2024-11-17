//! Frequency (mono bit) test
//!
//! This test focuses on the numbers of ones and zeros in the sequence - the proportion should
//! be roughly 50:50.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, checked_add, checked_add_unsigned, checked_sub_unsigned, erfc};
use crate::{Error, TestResult};
use rayon::prelude::*;
use std::f64::consts::FRAC_1_SQRT_2;
use std::num::NonZero;
use sts_lib_derive::use_thread_pool;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(100) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

/// Frequency (mono bit) test - No. 1
///
/// See the [module docs](crate::tests::frequency).
/// If an error happens, it means either arithmetic underflow or overflow - beware.
#[use_thread_pool]
pub fn frequency_test(data: &BitVec) -> Result<TestResult, Error> {
    // Step 1: convert 0 values to -1 and calculate the sum of all bits.
    // This operation is done in parallel.
    // first sum up the full bytes, then the remaining bits.
    let mut sum = data
        .words
        .par_iter()
        .try_fold(
            || 0_isize,
            |mut sum, value| {
                // the count of bits with value '1' in the byte
                let count_ones = value.count_ones() as usize;
                // the count of zeros is built from the count of ones (1 byte = 8 bits)
                let count_zeros = (usize::BITS as usize) - count_ones;

                // Adding and subtracting the count from the sum ist the same as conversion to -1 and +1.
                // Conversion to usize is definitely safe - count_ones and count_zeros range `0..=8`
                sum = checked_add_unsigned!(sum, count_ones)?;
                sum = checked_sub_unsigned!(sum, count_zeros)?;
                Ok(sum)
            },
        )
        .try_reduce(|| 0_isize, |a, b| checked_add!(a, b))?;

    if data.bit_count_last_word != 0 {
        // subtracted too many zeros in the last word, add them again
        let zeroes = (usize::BITS as usize) - (data.bit_count_last_word as usize);

        sum = checked_add_unsigned!(sum, zeroes)?;
    }

    // Step 2: compute s_obs = abs(sum) / sqrt(n)
    let s_obs =
        (sum.checked_abs()
            .ok_or_else(|| Error::Overflow(format!("abs({sum})")))? as f64)
            / f64::sqrt(data.len_bit() as f64);

    check_f64(s_obs)?;

    // Step 3: compute P-value = erfc(s_obs / sqrt(2))
    let p_value = erfc(s_obs * FRAC_1_SQRT_2);

    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}
