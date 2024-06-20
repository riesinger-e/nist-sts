//! Runs test.
//! 
//! This tests focuses on the number of runs in the sequence. A run is an uninterrupted sequence of
//! identical bits.
//! Each tested [BitVec] should have at least 100 bits length.


use crate::bitvec::BitVec;
use crate::{BYTE_SIZE, Error, TestResult};
use rayon::prelude::*;
use crate::internals::{check_f64, erfc};

/// Runs test - No. 3
///
/// See the [module docs](crate::tests::runs_test).
/// If an error happens, it means either arithmetic underflow or overflow - beware.
pub fn runs_test(data: &BitVec) -> Result<TestResult, Error> {
    // Step 1: calculate pi = count of ones / length of data
    let count_ones = data.data.par_iter()
        .try_fold(|| 0_usize, |sum, value| {
            sum.checked_add(value.count_ones() as usize)
                .ok_or(Error::Overflow(format!("adding the ones in the current byte to sum {sum}")))
        })
        .try_reduce(|| 0_usize, |a, b| {
            a.checked_add(b)
                .ok_or(Error::Overflow(format!("adding the part-sums {a} and {b}")))
        })?;
    // add remainder
    let count_ones = data.remainder.iter().map(|&b| b as usize).sum::<usize>()
        .checked_add(count_ones)
        .ok_or(Error::Overflow(format!("adding remainder sum to {count_ones}")))?
        as f64;
    let pi = count_ones / (data.len_bit() as f64);

    // Step 2: determine if the frequency test passed: abs(pi - 1/2) < 2 / sqrt(len_bit) has to uphold.
    // Otherwise, the test should not run because the frequency test would not pass.
    if f64::abs(pi - 0.5) >= 2.0 / f64::sqrt(data.len_bit() as f64) {
        // Frequency test would fail, don't run the test
        return Ok(TestResult::new_with_comment(0.0, "Frequency test would not pass!"))
    }

    // Step 3: compute the statistic V = (sum of r(k) for data[1..] - index k) + 1
    //  where r(k) = 0 if data[k] == data[k-1], else 1.
    // Work with bitwise indices here
    let v_data = (1..(data.data.len() * BYTE_SIZE)).into_par_iter()
        .try_fold(|| 0_usize, |sum, bit_idx| {
            // byte index of current bit
            let byte_idx = bit_idx / BYTE_SIZE;
            // byte index of previous bit
            let byte_idx_prev = (bit_idx - 1) / BYTE_SIZE;

            // index in the byte of the current bit
            let bit_idx_in_byte = bit_idx % BYTE_SIZE;
            let bit_idx_in_byte_prev = (bit_idx - 1) % BYTE_SIZE;

            let current_bit = (data.data[byte_idx] >> (BYTE_SIZE - bit_idx_in_byte - 1)) & 0x01;
            let previous_bit = (data.data[byte_idx_prev] >> (BYTE_SIZE - bit_idx_in_byte_prev - 1)) & 0x01;

            if current_bit == previous_bit {
                Ok(sum)
            } else {
                sum.checked_add(1)
                    .ok_or(Error::Overflow(format!("adding 1 to {sum}")))
            }
        })
        .try_reduce(|| 0_usize, |a, b| {
            a.checked_add(b)
                .ok_or(Error::Overflow(format!("adding sum {a} to {b}")))
        })?;
    // calculate for remainder
    let v_rem = if !data.remainder.is_empty() {
        let start_idx = if data.data.is_empty() { 1 } else { 0 };
        let mut prev_value = if start_idx == 0 {
            // if data.data contained values, take the last bit of it
            *data.data.last().unwrap() & 0x01 == 1
        } else {
            // else take the first bit from the remainder, direct index is OK because remainder
            // is not empty
            data.remainder[0]
        };

        data.remainder[start_idx..].iter()
            .try_fold(0_usize, |sum, &bit| {
                let res = if bit == prev_value {
                    Ok(sum)
                } else {
                    sum.checked_add(1)
                        .ok_or(Error::Overflow(format!("adding 1 to remainder sum {sum}")))
                };
                prev_value = bit;
                res
            })?
    } else {
        // if remainder is empty, just use 0
        0
    };

    let v = v_data.checked_add(v_rem)
        .ok_or(Error::Overflow(format!("adding v_data {v_data} to v_rem {v_rem}")))?;
    let v = v.checked_add(1)
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
