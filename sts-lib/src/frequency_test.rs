//! Frequency (mono bit) test
//!
//! This test focuses on the numbers of ones and zeros in the sequence - the proportion should
//! be roughly 50:50.

use crate::internals::erfc;
use crate::BYTE_SIZE;
use crate::{CommonResult, Error};
use rayon::prelude::*;
use std::f64::consts::FRAC_1_SQRT_2;
use crate::bitvec::BitVec;

/// Frequency (mono bit) test - No.1
/// 
/// See the [module docs](crate::frequency_test).
/// If an error happens, it means either arithmetic underflow or overflow - beware.
pub fn frequency_test(data: BitVec) -> Result<CommonResult, Error> {
    // Step 1: convert 0 values to -1 and calculate the sum of all bits.
    // This operation is done in parallel.
    // first sum up the full bytes, then the remaining bits.
    let sum = data
        .data
        .par_iter()
        .try_fold(
            || 0_isize,
            |mut sum, value| {
                // the count of bits with value '1' in the byte
                let count_ones = value.count_ones() as usize;
                // the count of zeros is built from the count of ones (1 byte = 8 bits)
                let count_zeros = BYTE_SIZE - count_ones;

                // Adding and subtracting the count from the sum ist the same as conversion to -1 and +1.
                // Conversion to usize is definitely safe - count_ones and count_zeros range `0..=8`
                sum = sum
                    .checked_add_unsigned(count_ones)
                    .ok_or(Error::Overflow(format!(
                        "adding Ones to the sum: {sum} + {count_ones}"
                    )))?;
                sum = sum
                    .checked_sub_unsigned(count_zeros)
                    .ok_or(Error::Overflow(format!(
                        "removing Zeroes from the sum: {sum} + {count_zeros}"
                    )))?;
                Ok(sum)
            },
        )
        .try_reduce(
            || 0_isize,
            |a, b| {
                a.checked_add(b).ok_or(Error::Overflow(format!(
                    "Adding two parts of the sum: {a}, {b}"
                )))
            },
        )?;
    // remainder: not parallel, always a maximum of 8
    let sum = data.remainder.iter().try_fold(sum, |sum, value| {
        if *value {
            // true is 1
            sum.checked_add_unsigned(1)
        } else {
            // false is -1
            sum.checked_sub_unsigned(1)
        }
            .ok_or(Error::Overflow(format!("adding the remainder to the sum: {sum}")))
    })?;
    
    // Step 2: compute s_obs = abs(sum) / sqrt(n)
    let s_obs = (sum
        .checked_abs()
        .ok_or(Error::Overflow(format!("abs({sum}) - type isize")))? as f64)
        / f64::sqrt(data.len_bit() as f64);
    
    // Step 3: compute P-value = erfc(s_obs / sqrt(2))
    let p_value = erfc(s_obs * FRAC_1_SQRT_2);

    Ok(CommonResult { p_value })
}
