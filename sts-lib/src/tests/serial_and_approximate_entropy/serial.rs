//! The serial test.
//!
//! This test checks the frequency of all 2^m overlapping m-bit patterns in the sequence. Random
//! sequences should be uniform. For *m = 1*, this would be the same as the
//! [Frequency Test](crate::tests::frequency).
//!
//! This test needs a parameter [SerialTestArg]. Check the described constraints there.
//!
//! The paper describes the test slightly wrong: in 2.11.5 step 5, the second argument need to be 
//! halved in both *igamc* calculations. Only then are the calculated P-values equal to the P-values
//! described in 2.11.6 and the reference implementation.
//! 
//! The input length should be at least 2^19 bit, although this is not enforced. If the default 
//! value for [SerialTestArg] is used, a smaller input length will lead to an Error because
//! of constraint no. 3!

use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::{Error, TestResult};
use rayon::prelude::*;
use crate::tests::serial_and_approximate_entropy::{access_bits, validate_test_arg};

// calculation: minimum block length = 2
// Following relation must be true:
// 2 < (log2(len_bit) as int) - 2
// -> log2(2^5) - 2 = 3
/// The minimum input length for this test.
pub const MIN_INPUT_LENGTH: usize = 1 << 5;

/// The argument for the serial test: the block length in bits to check.
///
/// Argument constraints:
/// 1. the given block length must be >= 2.
/// 2. each value of with the bit length the given block length must be representable as usize,
///     i.e. depending on the platform, 32 or 64 bits.
/// 3. the block length must be < (log2([BitVec::len_bit]) as int) - 2
///
/// Constraints 1 and 2 are checked when creating the arguments.
///
/// Constraint 3 is checked on executing the test, [serial_test]. If the constraint is violated,
/// [Error::InvalidParameter] will be returned.
/// 
/// The default value for this argument is 16. For this to work, the input length must be at least
/// 2^19 bit.
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct SerialTestArg(u8);

impl SerialTestArg {
    /// To create a new instance of [SerialTestArg]. This function checks some constraints,
    /// for details, see [SerialTestArg].
    pub fn new(block_length: u8) -> Option<Self> {
        validate_test_arg(block_length).map(Self)
    }
}

impl Default for SerialTestArg {
    fn default() -> Self {
        Self(16)
    }
}


/// Serial Test  - No. 11
///
/// See also the [module docs](crate::tests::serial).
/// If the combination of the given data ([BitVec]) and [SerialTestArg] is invalid,
/// [Error::InvalidParameter] is raised. For the exact constraints, see [SerialTestArg].
//noinspection DuplicatedCode
pub fn serial_test(data: &BitVec, SerialTestArg(block_length): SerialTestArg) -> Result<[TestResult; 2], Error> {
    // only check the argument when not testing
    #[cfg(not(test))]
    {
        // check that the block length and the input parameter work with each other.
        let max_block_length = f64::log2(data.len_bit() as f64) as usize - 2;

        if (block_length as usize) >= max_block_length {
            return Err(Error::InvalidParameter(format!(
                "Given block length must be lesser than log2(len_bit) - 2 (={max_block_length}). Is: {block_length}"
            )));
        }
    }

    // Step 1 is skipped: we just read from the start again, see access_bits()
    // Step 2: determine the frequency of all possible overlapping m, (m-1) and (m-2) bit blocks.
    // (m == block_length)
    let frequencies = (0..data.len_bit())
        .into_par_iter()
        .try_fold(
            || create_frequency_slices(block_length),
            |mut frequencies, idx| {
                for i in 0..3 {
                    // this can happen when block_length = 2
                    if block_length - i == 0 {
                        continue;
                    }

                    let idx = access_bits(data, idx, block_length - i).unwrap_or_else(|| {
                        panic!("serial_test: idx for (m - {i}) should be valid")
                    });
                    frequencies[i as usize][idx] = frequencies[i as usize][idx]
                        .checked_add(1)
                        .ok_or(Error::Overflow(format!(
                            "Adding 1 to frequency count {}",
                            frequencies[i as usize][idx]
                        )))?;
                }

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
    
    // Step 3: for each tested block length m (3 in total), compute
    // psi^2(m) = (2^m) / n * sum(v_mi^2) - n
    // with n denoting the bit length of the sequence and v_mi denoting on element in the frequency list
    // of the block length.
    let mut psi = [0.0; 3];
    psi.iter_mut()
        .zip(frequencies)
        .enumerate()
        .try_for_each(|(i, (psi, frequency))| {
            // this can happen when block_length = 2
            if block_length - i as u8 == 0 {
                *psi = 0.0;
                return Ok(())
            }

            let sum = frequency
                .into_vec()
                .into_par_iter()
                .map(|v| (v * v) as f64)
                .sum::<f64>();

            check_f64(sum)?;

            *psi = f64::powi(2.0, block_length as i32 - i as i32) / (data.len_bit() as f64) * sum
                - (data.len_bit() as f64);

            check_f64(*psi)
        })?;

    // Step 4: compute delta = psi[0] - psi[1] and delta^2 = psi[0] - 2 * psi[1] + psi[2]
    let delta = psi[0] - psi[1];
    let delta_squared = psi[0] - 2.0 * psi[1] + psi[2];

    // Step 5: compute p_value_1 = igamc(2^(block_length - 2), delta / 2)
    // and p_value_2 = igamc(2^(block_length - 3), delta^2 / 2).
    // The paper is wrong here! Both the examples and the reference implementation agree on
    // delta / 2 and delta^2 / 2.
    let p_value_1 = igamc(f64::powi(2.0, block_length as i32 - 2), delta / 2.0)?;
    let p_value_2 = igamc(f64::powi(2.0, block_length as i32 - 3), delta_squared / 2.0)?;

    Ok([TestResult::new(p_value_1), TestResult::new(p_value_2)])
}

/// Returns 3 boxed slices used for storing the measured frequency of a given pattern.
/// \[0] is for patterns with bit length `block_length`, \[1] for `block_length - 1` and
/// \[2] for `block_length - 2`.
/// The pattern is used as the index for each boxed slice, the value itself stores the frequency.
#[inline]
fn create_frequency_slices(block_length: u8) -> [Box<[usize]>; 3] {
    #[inline]
    fn create_frequency_slice(block_length: u8) -> Box<[usize]> {
        let len = 1 << block_length;
        vec![0; len].into_boxed_slice()
    }

    [
        create_frequency_slice(block_length),
        create_frequency_slice(block_length - 1),
        create_frequency_slice(block_length - 2),
    ]
}
