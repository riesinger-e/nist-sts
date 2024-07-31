//! The serial test.
//!
//! This test checks the frequency of all 2^m overlapping m-bit patterns in the sequence. Random
//! sequences should be uniform. For *m = 1*, this would be the same as the
//! [Frequency Test](crate::tests::frequency).
//!
//! This test needs a parameter [SerialTestArg]. Check the described constraints there.
//!
//! The paper describes the test slightly wrong: in 2.11.5 step 5, both arguments need to be halved
//! in both *igamc* calculations. Also, the exponent is wrong: it needs to be *m-1* and *m-2*. Only
//! then are the calculated P-values equal to the P-values described in 2.11.6 and the reference
//! implementation.
//! 
//! The input length should be at least 2^19 bit, although this is not enforced. If the default 
//! value for [SerialTestArg] is used, a smaller input length will lead to an Error because
//! of constraint no. 3!

use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::{Error, TestResult, BYTE_SIZE};
use rayon::prelude::*;

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
        // block length > 1 (else this is just the frequency test) and maximum of usize bits (32 or 64)
        if block_length > 1 && block_length as u32 <= usize::BITS {
            Some(Self(block_length))
        } else {
            None
        }
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
            |mut a, mut b| {
                for i in 0..3 {
                    a[i].iter_mut()
                        .zip(std::mem::take(&mut b[i]))
                        .try_for_each(|(el_a, el_b)| {
                            *el_a = el_a.checked_add(el_b).ok_or(Error::Overflow(format!(
                                "Adding frequency counts {el_a} and {el_b}"
                            )))?;
                            Ok::<_, Error>(())
                        })?;
                }

                Ok::<_, Error>(a)
            },
        )?;

    println!("frequencies: {:?}", frequencies);

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
    println!("psi: {:?}", psi);

    // Step 4: compute delta = psi[0] - psi[1] and delta^2 = psi[0] - 2 * psi[1] + psi[2]
    let delta = psi[0] - psi[1];
    let delta_squared = psi[0] - 2.0 * psi[1] + psi[2];

    println!("delta: {delta}");
    println!("delta_squared: {delta_squared}");

    // Step 5: compute p_value_1 = igamc(2^(block_length - 2), delta / 2)
    // and p_value_2 = igamc(2^(block_length - 3), delta^2 / 2).
    // The paper is wrong here! Both the examples and the reference implementation agree on
    // delta / 2 and delta^2 / 2.
    let p_value_1 = igamc(f64::powi(2.0, block_length as i32 - 1) / 2.0, delta / 2.0)?;
    let p_value_2 = igamc(f64::powi(2.0, block_length as i32 - 2) / 2.0, delta_squared / 2.0)?;

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

/// Retrieves the bits at start_idx + block_length (e.g. for block_length = 3, 3 bits are retrieved)
/// and returns them.
///
/// start_idx is measured in bits.
///
/// Bounds: start_idx < [BitVec::len_bit]
fn access_bits(data: &BitVec, start_idx: usize, block_length: u8) -> Option<usize> {
    /// Handles extracting the needed bits from the first byte. Return the start number.
    #[inline]
    fn first_byte(bits_todo: &mut u8, byte: u8, start_bit_idx: usize) -> usize {
        let mask = (1 << (BYTE_SIZE - start_bit_idx)) - 1;
        let first_byte = (byte as usize) & mask;

        *bits_todo -= (BYTE_SIZE - start_bit_idx) as u8;
        first_byte << *bits_todo
    }

    /// Handles extracting the needed bits from the last byte.
    #[inline]
    fn last_byte(number: &mut usize, byte: u8, end_bit_idx: usize) {
        // exclusive end index
        let last_byte = byte >> (BYTE_SIZE - end_bit_idx);
        *number |= last_byte as usize
    }

    if start_idx >= data.len_bit() {
        return None;
    }

    // convert index
    let start_byte_idx = start_idx / BYTE_SIZE;
    let start_bit_idx = start_idx % BYTE_SIZE;

    let end_byte_idx = (start_idx + block_length as usize - 1) / BYTE_SIZE;
    let end_byte_idx = if end_byte_idx == data.data.len() {
        let end_bit_idx = (start_idx + block_length as usize - 1) % BYTE_SIZE;
        if end_bit_idx < data.remainder.len() {
            end_byte_idx
        } else {
            end_byte_idx + 1
        }
    } else {
        end_byte_idx
    };

    if end_byte_idx == start_byte_idx {
        if start_byte_idx < data.data.len() {
            // starts and ends in the same byte
            let end_bit_idx = (start_idx + block_length as usize) % BYTE_SIZE;

            let byte = data.data[start_byte_idx] as usize;
            let mask = (1 << (BYTE_SIZE - start_bit_idx)) - 1;
            let byte = byte & mask;

            let number = if end_bit_idx != 0 {
                byte >> (BYTE_SIZE - end_bit_idx)
            } else {
                byte
            };

            Some(number)
        } else {
            // start_byte_idx == data.data.len() - exclusively from the last byte
            let end_bit_idx = (start_idx + block_length as usize) % BYTE_SIZE;
            let bit_length = end_bit_idx - start_bit_idx;

            let mut number = 0;

            for (i, &bit) in data.remainder[start_bit_idx..end_bit_idx].iter().enumerate() {
                if bit {
                    number |= 1 << (bit_length - i - 1)
                }
            }

            Some(number)
        }
    } else if end_byte_idx > data.data.len() {
        // overflow, last bits may not be a full byte, adjustment ist needed
        let additional_bits = start_idx + block_length as usize - data.len_bit();
        let end_byte_idx = additional_bits / BYTE_SIZE;
        let end_bit_idx = additional_bits % BYTE_SIZE;

        // the necessary left shift for the next bits
        let mut bits_todo = block_length;

        // different depending on if the first bit is in data.data or data.remainder
        let mut number = if start_byte_idx < data.data.len() {
            // special case: first byte
            let mut number = first_byte(&mut bits_todo, data.data[start_byte_idx], start_bit_idx);

            // first loop to end of data.data
            for &byte in &data.data[(start_byte_idx + 1)..] {
                bits_todo -= BYTE_SIZE as u8;
                number |= (byte as usize) << bits_todo;
            }

            // last byte of 'data'
            for &bit in &data.remainder {
                bits_todo -= 1;
                if bit {
                    number |= 1 << bits_todo;
                }
            }

            number
        } else {
            let mut number = 0;

            for &bit in &data.remainder[start_bit_idx..] {
                bits_todo -= 1;
                if bit {
                    number |= 1 << bits_todo;
                }
            }

            number
        };

        // second loop, starting from the beginning of data.data
        for &byte in &data.data[0..end_byte_idx] {
            bits_todo -= BYTE_SIZE as u8;
            number |= (byte as usize) << bits_todo;
        }

        // special case: last byte
        last_byte(&mut number, data.data[end_byte_idx], end_bit_idx);

        Some(number)
    } else if end_byte_idx == data.data.len() {
        // the necessary left shift for the next bits
        let mut bits_todo = block_length;

        // special case: first byte
        let mut number = first_byte(&mut bits_todo, data.data[start_byte_idx], start_bit_idx);

        // loop to end of data.data
        for &byte in &data.data[(start_byte_idx + 1)..] {
            bits_todo -= BYTE_SIZE as u8;
            number |= (byte as usize) << bits_todo;
        }

        // last byte of 'data'
        for &bit in &data.remainder {
            bits_todo -= 1;
            if bit {
                number |= 1 << bits_todo;
            }

            if bits_todo == 0 {
                break;
            }
        }

        Some(number)
    } else {
        // no overflow, conventional calculation
        let end_bit_idx = (start_idx + block_length as usize) % BYTE_SIZE;

        let mut bits_todo = block_length;

        // special case: first byte
        let mut number = first_byte(&mut bits_todo, data.data[start_byte_idx], start_bit_idx);

        for &byte in &data.data[(start_byte_idx + 1)..end_byte_idx] {
            bits_todo -= BYTE_SIZE as u8;
            number |= (byte as usize) << bits_todo;
        }

        // special case: last byte
        last_byte(&mut number, data.data[end_byte_idx], end_bit_idx);

        Some(number)
    }
}
