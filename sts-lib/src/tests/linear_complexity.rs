//! The linear complexity test.
//!
//! This test determines the randomness of a sequence by calculating the minimum length of a linear
//! feedback shift register that can create the sequence. Random sequences need longer LSFRs.
//!
//! This test needs a parameter, [LinearComplexityTestArg]. Additionally, the input sequence
//! must have a minimum length of 10^6 bits.

use crate::{BYTE_SIZE, Error, TestResult};
use std::cmp::Ordering;
use crate::bitvec::BitVec;
use rayon::prelude::*;
use crate::internals::{check_f64, igamc};

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
    2.0 / (32.0 * 3.0)
];

/// The argument for the [linear_complexity_test].
/// Allows to choose the block length manually or automatically.
///
/// If the block length is chosen manually, the following equations must be true:
/// * 500 <= block length <= 5000
/// * total bit length / block length >= 200
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub enum LinearComplexityTestArg {
    /// Choose the block length (in bit) manually. Must be between 500 and 5000.
    /// See also [LinearComplexityTestArg].
    ManualBlockLength(usize),
    /// Choose the block length automatically.
    #[default]
    ChooseAutomatically,
}

/// The linear complexity test - No. 10
///
/// See also the [module docs](crate::tests::linear_complexity).
pub fn linear_complexity_test(data: &BitVec, arg: LinearComplexityTestArg) -> Result<TestResult, Error> {
    // Step 0: validate input arguments
    if data.len_bit() < 1_000_000 {
        return Err(Error::InvalidParameter(format!("Length of input data must be >= 10^6. Is: {}", data.len_bit())))
    }

    let (block_length, count_blocks) = match arg {
        LinearComplexityTestArg::ManualBlockLength(block_length) => {
            // validate block length and count blocks
            if !(500..=5000).contains(&block_length) {
                return Err(Error::InvalidParameter(format!("block length must be between 500 and 5000. Is: {block_length}")))
            }

            let count_blocks = data.len_bit() / block_length;

            if count_blocks < 200 {
                return Err(Error::InvalidParameter("the chosen block length leads to fewer than 200 blocks!".to_owned()));
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
        .try_fold(|| [0_usize; FREEDOM_DEGREES + 1], |mut sum, block_idx| {
            // calculate the start byte and the bit position in the start byte for this block
            let total_start_bit =
                block_idx
                    .checked_mul(block_length)
                    .ok_or(Error::Overflow(format!(
                        "multiplying {block_idx} by {block_length}"
                    )))?;

            let start_byte = total_start_bit / BYTE_SIZE;
            let start_bit = total_start_bit % BYTE_SIZE;

            let end_byte = ((block_idx + 1)
                .checked_mul(block_length)
                .ok_or(Error::Overflow(format!(
                    "multiplying {} by {block_length}", block_idx + 1,
                )))? - 1) / BYTE_SIZE;

            // Step 2
            let l_i = if end_byte < data.data.len() {
                berlekamp_massey(&data.data[start_byte..=end_byte], None, block_length, start_bit)
            } else {
                berlekamp_massey(&data.data[start_byte..], Some(data.get_last_byte()), block_length, start_bit)
            };

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

            sum[idx_to_increment] = sum[idx_to_increment].checked_add(1)
                .ok_or(Error::Overflow(format!(
                    "adding 1 to {}", sum[idx_to_increment]
                )))?;

            Ok::<_, Error>(sum)
        })
        .try_reduce(|| [0_usize; FREEDOM_DEGREES + 1], |mut a, b| {
            for i in 0..(FREEDOM_DEGREES + 1) {
                a[i] = a[i]
                    .checked_add(b[i])
                    .ok_or(Error::Overflow(format!(
                        "adding {} to {}", a[i], b[i]
                    )))?;
            }

            Ok(a)
        })?;

    // Step 6: compute chi^2 = sum of ( (tables[i] - count_blocks * pi[i])^2 / (count_blocks * pi[i]) )
    let chi = table.into_iter().zip(PI_VALUES)
        .map(|(v_i, pi_i)| {
            f64::powi((v_i as f64) - (count_blocks as f64) * pi_i, 2) / ((count_blocks as f64) * pi_i)
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
    sequence: &[u8],
    additional_bits: Option<u8>,
    total_bit_len: usize,
    start_bit: usize,
) -> usize {
    // Initialize C(D) - saves the values of a binary polynom
    let mut c: Vec<u8> = vec![0; total_bit_len / BYTE_SIZE + 1];
    c[0] = 0b1000_0000;
    // the linear complexity
    let mut l = 0_usize;
    // the value m
    let mut m = -1_isize;
    // B(D) - binary polynom
    let mut b: Vec<u8> = vec![0b1000_0000];

    // for all following calculations:
    // In a base 2 system, PLUS is the same as XOR and MULT is the same as AND.
    // Raising to the power can be done with bit shifts.
    for n in 0..total_bit_len {
        // compute discrepancy
        let mut sum = false;
        for i in 1..(l + 1) {
            sum ^= get_bit(&c, None, i).unwrap()
                & get_bit(sequence, additional_bits, start_bit + n - i).unwrap();
        }

        let s_n = get_bit(sequence, additional_bits, start_bit + n).unwrap();
        let d = s_n ^ sum;

        if d {
            let t = c.clone();

            // addition of polynoms: shift is the power
            let shift = ((n as isize) - m) as usize;
            let bytes_forward = shift / BYTE_SIZE;
            let shift = shift % BYTE_SIZE;

            for (idx, bit) in b.iter().enumerate() {
                if idx + bytes_forward < c.len() {
                    let shifted_value = bit >> shift;
                    c[idx + bytes_forward] ^= shifted_value;

                    if idx + bytes_forward + 1 < c.len() && shift > 0 {
                        let carry_over = bit << (BYTE_SIZE - shift);
                        c[idx + bytes_forward + 1] ^= carry_over;
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
fn get_bit(sequence: &[u8], additional_bits: Option<u8>, position: usize) -> Option<bool> {
    let byte_pos = position / BYTE_SIZE;
    let bit_pos = position % BYTE_SIZE;

    let byte = match byte_pos.cmp(&sequence.len()) {
        Ordering::Less => sequence[byte_pos],
        Ordering::Equal => additional_bits?,
        Ordering::Greater => return None,
    };

    let bit = (byte >> (BYTE_SIZE - bit_pos - 1)) & 0x1;
    Some(bit == 1)
}