//! The linear complexity test.
//!
//! This test determines the randomness of a sequence by calculating the minimum length of a linear
//! feedback shift register that can create the sequence. Random sequences need longer LSFRs.
//!
//! This test needs a parameter, [LinearComplexityTestArg]. Additionally, the input sequence
//! must have a minimum length of 10^6 bits.
//!
//! The probability constants are used as fractions in this implementation. This, and the fact that
//! NIST uses wrong probability constants (there is a typo in pi\[0] - 0.01047 is given instead of
//! 0.010417) means that results may deviate significantly from the NIST reference implementation.
//! This is expected behaviour.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::{Error, TestResult};
use rayon::prelude::*;
use std::num::NonZero;
use sts_lib_derive::use_thread_pool;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(1_000_000) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

/// freedom degrees
const FREEDOM_DEGREES: usize = 6;

/// pi values for calculating chi^2. These are the values as given in 2.10.4 step 6, but
/// expressed as fractions instead of decimal constants for more precision.
const PI_VALUES: [f64; FREEDOM_DEGREES + 1] = [
    1.0 / (32.0 * 3.0),
    1.0 / 32.0,
    1.0 / 8.0,
    1.0 / 2.0,
    1.0 / 4.0,
    1.0 / 16.0,
    2.0 / (32.0 * 3.0),
];

/// The argument for the [linear_complexity_test].
/// Allows to choose the block length manually or automatically.
///
/// If the block length is chosen manually, the following equations must be true:
/// * 500 <= block length <= 5000
/// * total bit length / block length >= 200
#[derive(Copy, Clone, Debug, Default)]
pub enum LinearComplexityTestArg {
    /// Choose the block length (in bit) manually. Must be between 500 and 5000.
    /// See also [LinearComplexityTestArg].
    ManualBlockLength(NonZero<usize>),
    /// Choose the block length automatically.
    #[default]
    ChooseAutomatically,
}

/// The linear complexity test - No. 10
///
/// See also the [module docs](crate::tests::linear_complexity).
#[use_thread_pool(crate::internals::THREAD_POOL)]
pub fn linear_complexity_test(
    data: &BitVec,
    arg: LinearComplexityTestArg,
) -> Result<TestResult, Error> {
    // Step 0: validate input arguments
    if data.len_bit() < 1_000_000 {
        return Err(Error::InvalidParameter(format!(
            "Length of input data must be >= 10^6. Is: {}",
            data.len_bit()
        )));
    }

    let (block_length, count_blocks) = match arg {
        LinearComplexityTestArg::ManualBlockLength(block_length) => {
            let block_length = block_length.get();
            // validate block length and count blocks
            if !(500..=5000).contains(&block_length) {
                return Err(Error::InvalidParameter(format!(
                    "block length must be between 500 and 5000. Is: {block_length}"
                )));
            }

            let count_blocks = data.len_bit() / block_length;

            if count_blocks < 200 {
                return Err(Error::InvalidParameter(
                    "the chosen block length leads to fewer than 200 blocks!".to_owned(),
                ));
            }

            (block_length, count_blocks)
        }
        LinearComplexityTestArg::ChooseAutomatically => {
            // always choose 512 bit
            (512, data.len_bit() / 512)
        }
    };

    // Step 3: calculate the theoretical mean
    let mean = (block_length as f64) / 2.0
        + (9.0 + f64::powi(-1.0, block_length as i32 + 1)) / 36.0
        - ((block_length as f64) / 3.0 + 2.0 / 9.0) / f64::powi(2.0, block_length as i32);

    // Step 2: for each block, calculate the linear complexity L_i according to berlekamp massey
    // Step 4: for each block, calculate T_i = (-1)^block_length * (L_i - mean) + 2/9
    // Step 5: sort the T_i value into an array depending on their value
    let table = (0..count_blocks)
        .into_par_iter()
        .try_fold(
            || [0_usize; FREEDOM_DEGREES + 1],
            |mut sum, block_idx| {
                // calculate the start byte and the bit position in the start byte for this block
                let total_start_bit =
                    block_idx
                        .checked_mul(block_length)
                        .ok_or(Error::Overflow(format!(
                            "multiplying {block_idx} by {block_length}"
                        )))?;

                let start_idx = total_start_bit / (usize::BITS as usize);
                let start_bit_idx = total_start_bit % (usize::BITS as usize);

                let end_idx =
                    ((block_idx + 1)
                        .checked_mul(block_length)
                        .ok_or(Error::Overflow(format!(
                            "multiplying {} by {block_length}",
                            block_idx + 1,
                        )))?
                        - 1)
                        / (usize::BITS as usize);

                // Step 2
                let l_i = berlekamp_massey(
                    &data.words[start_idx..=end_idx],
                    block_length,
                    start_bit_idx,
                );

                // Step 4
                let t_i = f64::powi(-1.0, block_length as i32) * ((l_i as f64) - mean) + 2.0 / 9.0;
                check_f64(t_i)?;

                // Step 5
                let idx_to_increment = if t_i <= -2.5 {
                    0
                } else if t_i <= -1.5 {
                    1
                } else if t_i <= -0.5 {
                    2
                } else if t_i <= 0.5 {
                    3
                } else if t_i <= 1.5 {
                    4
                } else if t_i <= 2.5 {
                    5
                } else {
                    6
                };

                sum[idx_to_increment] =
                    sum[idx_to_increment]
                        .checked_add(1)
                        .ok_or(Error::Overflow(format!(
                            "adding 1 to {}",
                            sum[idx_to_increment]
                        )))?;

                Ok::<_, Error>(sum)
            },
        )
        .try_reduce(
            || [0_usize; FREEDOM_DEGREES + 1],
            |mut a, b| {
                for i in 0..(FREEDOM_DEGREES + 1) {
                    a[i] = a[i]
                        .checked_add(b[i])
                        .ok_or(Error::Overflow(format!("adding {} to {}", a[i], b[i])))?;
                }

                Ok(a)
            },
        )?;

    // Step 6: compute chi^2 = sum of ( (tables[i] - count_blocks * pi[i])^2 / (count_blocks * pi[i]) )
    let chi = table
        .into_iter()
        .zip(PI_VALUES)
        .map(|(v_i, pi_i)| {
            f64::powi((v_i as f64) - (count_blocks as f64) * pi_i, 2)
                / ((count_blocks as f64) * pi_i)
        })
        .sum::<f64>();
    check_f64(chi)?;

    // Step 7: compute p-value = igamc(freedom_degrees / 2, chi^2 / 2)
    let p_value = igamc(FREEDOM_DEGREES as f64 / 2.0, chi / 2.0)?;

    Ok(TestResult::new(p_value))
}

/// An implementation of the Berlekamp-Massey algorithm for calculating the linear complexity of a
/// bit sequence, according to the Handbook of Applied Cryptography, p. 201, 6.30.
///
/// Inputs: the sequence stored as packed binary (8 bits per byte) + 1 optional byte additional bits,
/// the bit length of the sequence to calculate the linear complexity for, the start bit in the
/// sequence.
pub(crate) fn berlekamp_massey(
    sequence: &[usize],
    total_bit_len: usize,
    start_bit: usize,
) -> usize {
    // Initialize C(D) - saves the values of a binary polynom
    let mut c: Vec<usize> = vec![0; total_bit_len / (usize::BITS as usize) + 1];
    c[0] = 1 << (usize::BITS - 1);
    // the linear complexity
    let mut l = 0_usize;
    // the value m
    let mut m = -1_isize;
    // B(D) - binary polynom
    let mut b: Vec<usize> = vec![1 << (usize::BITS - 1)];

    // for all following calculations:
    // In a base 2 system, PLUS is the same as XOR and MULT is the same as AND.
    // Raising to the power can be done with bit shifts.
    for n in 0..total_bit_len {
        // compute discrepancy
        let mut sum = false;
        for i in 1..(l + 1) {
            sum ^= get_bit(&c, i).unwrap() & get_bit(sequence, start_bit + n - i).unwrap();
        }

        let s_n = get_bit(sequence, start_bit + n).unwrap();
        let d = s_n ^ sum;

        if d {
            let t = c.clone();

            // addition of polynoms: shift is the power
            let shift = ((n as isize) - m) as usize;
            let idx_forward = shift / (usize::BITS as usize);
            let shift = shift % (usize::BITS as usize);

            for (idx, bit) in b.iter().enumerate() {
                if idx + idx_forward < c.len() {
                    let shifted_value = bit >> shift;
                    c[idx + idx_forward] ^= shifted_value;

                    if idx + idx_forward + 1 < c.len() && shift > 0 {
                        let carry_over = bit << ((usize::BITS as usize) - shift);
                        c[idx + idx_forward + 1] ^= carry_over;
                    }
                }
            }

            if l <= n / 2 {
                l = n + 1 - l;
                m = n as isize;
                b = t;
            }
        }
    }

    l
}

/// Get the bit in a sequence with optional additional bits at the given position
fn get_bit(sequence: &[usize], position: usize) -> Option<bool> {
    let idx = position / (usize::BITS as usize);
    let bit_idx = position % (usize::BITS as usize);

    let value = sequence.get(idx)?;

    let bit = (value >> ((usize::BITS as usize) - bit_idx - 1)) & 0x1;
    Some(bit == 1)
}
