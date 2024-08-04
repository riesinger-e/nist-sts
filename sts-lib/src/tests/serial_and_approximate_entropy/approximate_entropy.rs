//! The approximate entropy test.
//!
//! This test is similar to the [serial test](crate::tests::serial). It compares the frequency
//! of overlapping blocks with the two block lengths *m* and *m + 1* against the expected result
//! of a random sequence.
//!
//! This test needs a parameter [ApproximateEntropyTestArg]. Check the described constraints there.
//!
//! The input length should be at least 2^16 bit, although this is not enforced. If the default
//! value for [ApproximateEntropyTestArg] is used, a smaller input length will lead to an Error because
//! of constraint no. 3!

use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::tests::serial_and_approximate_entropy::{access_bits, validate_test_arg};
use crate::{Error, TestResult};
use rayon::prelude::*;
use std::f64::consts::LN_2;

// calculation: minimum block length = 2
// Following relation must be true:
// 2 < (log2(len_bit) as int) - 5
// -> log2(2^8) - 5 = 3
/// The minimum input length for this test.
pub const MIN_INPUT_LENGTH: usize = 1 << 8;

/// The argument for the approximate entropy test: the block length in bits to check.
///
/// Argument constraints:
/// 1. the given block length must be >= 2.
/// 2. each value of with the bit length the given block length must be representable as usize,
///     i.e. depending on the platform, 32 or 64 bits.
/// 3. the block length must be < (log2([BitVec::len_bit]) as int) - 5
///
/// Constraints 1 and 2 are checked when creating the arguments.
///
/// Constraint 3 is checked on executing the test, [approximate_entropy_test]. If the constraint is violated,
/// [Error::InvalidParameter] will be returned.
///
/// The default value for this argument is 10. For this to work, the input length must be at least
/// 2^16 bit.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct ApproximateEntropyTestArg(u8);

impl ApproximateEntropyTestArg {
    /// To create a new instance of [ApproximateEntropyTestArg]. This function checks some constraints,
    /// for details, see [ApproximateEntropyTestArg].
    pub fn new(block_length: u8) -> Option<Self> {
        validate_test_arg(block_length).map(Self)
    }
}

impl Default for ApproximateEntropyTestArg {
    fn default() -> Self {
        Self(10)
    }
}

/// Approximate Entropy Test - No. 12
///
/// See also the [module docs](crate::tests::approximate_entropy).
/// If the combination of the given data ([BitVec]) and [ApproximateEntropyTestArg] is invalid,
/// [Error::InvalidParameter] is raised. For the exact constraints, see [ApproximateEntropyTestArg].
//noinspection DuplicatedCode
pub fn approximate_entropy_test(
    data: &BitVec,
    ApproximateEntropyTestArg(block_length): ApproximateEntropyTestArg,
) -> Result<TestResult, Error> {
    // only check the argument when not testing
    #[cfg(not(test))]
    {
        // check that the block length and the input parameter work with each other.
        let max_block_length = f64::log2(data.len_bit() as f64) as usize - 5;

        if (block_length as usize) >= max_block_length {
            return Err(Error::InvalidParameter(format!(
                "Given block length must be lesser than log2(len_bit) - 5 (={max_block_length}). Is: {block_length}"
            )));
        }
    }

    // Step 1 is skipped: we just read from the start again, see access_bits()
    // Step 2: determine the frequency of all possible overlapping m bit blocks.
    // Step 5.2: determine the frequency of all possible overlapping (m+1) bit blocks.
    // (m == block_length)
    let frequencies = (0..data.len_bit())
        .into_par_iter()
        .try_fold(
            || create_frequency_slices(block_length),
            |mut frequencies, idx| {
                frequencies
                    .iter_mut()
                    .enumerate()
                    .try_for_each(|(i, freq)| {
                        let idx =
                            access_bits(data, idx, block_length + i as u8).unwrap_or_else(|| {
                                panic!("serial_test: idx for (m + {i}) should be valid")
                            });

                        freq[idx] = freq[idx].checked_add(1).ok_or(Error::Overflow(format!(
                            "Adding 1 to frequency count {}",
                            freq[idx]
                        )))?;
                        Ok::<(), Error>(())
                    })?;

                Ok(frequencies)
            },
        )
        .try_reduce(
            || create_frequency_slices(block_length),
            |mut a, b| {
                a.iter_mut()
                    .zip(b)
                    .flat_map(|(a, b)| a.iter_mut().zip(b))
                    .try_for_each(|(el_a, el_b)| {
                        *el_a = el_a.checked_add(el_b).ok_or(Error::Overflow(format!(
                            "Adding frequency counts {el_a} and {el_b}"
                        )))?;
                        Ok::<_, Error>(())
                    })?;

                Ok::<_, Error>(a)
            },
        )?;

    // Step 3 / Step 5.3: for each frequency i, calculate i / len_bit
    // Step 4 / Step 5.4: calculate the sum of (i * ln(i)), where i denotes an entry in the frequency
    // array. Result is stored in phi
    let len_bit = data.len_bit();
    let phi = {
        let [frequency_0, frequency_1] = frequencies;
        [
            execute_step_3_and_4(frequency_0, len_bit)?,
            execute_step_3_and_4(frequency_1, len_bit)?,
        ]
    };

    // Step 5 is already finished (do step 1 to 4 for block_length + 1)

    // Step 6: compute the test statistic: chi^2 = 2 * n * [ln(2) - ( phi(m) - phi(m+1) )]
    let chi = 2.0 * (len_bit as f64) * (LN_2 - (phi[0] - phi[1]));
    check_f64(chi)?;

    // Step 7: compute p-value = igamc(2^(m-1), chi^2 / 2)
    let p_value = igamc(f64::powi(2.0, (block_length as i32) - 1), chi / 2.0)?;
    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}

/// Returns 2 boxed slices used for storing the measured frequency of a given pattern.
/// \[0] is for patterns with bit length `block_length`, \[1] for `block_length + 1`.
/// The pattern is used as the index for each boxed slice, the value itself stores the frequency.
#[inline]
fn create_frequency_slices(block_length: u8) -> [Box<[usize]>; 2] {
    #[inline]
    fn create_frequency_slice(block_length: u8) -> Box<[usize]> {
        let len = 1 << block_length;
        vec![0; len].into_boxed_slice()
    }

    [
        create_frequency_slice(block_length),
        create_frequency_slice(block_length + 1),
    ]
}

/// Executes step 3 and 4:
/// * Step 3: for each frequency i, calculate i / len_bit for the given frequency slice.
/// * Step 4: calculate the sum of (i * ln(i)), where *i* denotes an entry in the frequency.
///
/// Since resulting values are checked to be valid normal f64s, an error may be returned.
#[inline]
fn execute_step_3_and_4(frequency: Box<[usize]>, len_bit: usize) -> Result<f64, Error> {
    let phi = frequency
        .into_vec()
        .into_par_iter()
        .map(|el| {
            // step 3
            let pi = (el as f64) / (len_bit as f64);

            // step 4
            if pi != 0.0 {
                // ln(0) = -inf, and infinity is contagious, even if multiplied with 0
                pi * f64::ln(pi)
            } else {
                0.0
            }
        })
        .sum::<f64>();
    check_f64(phi)?;
    Ok(phi)
}
