//! The serial test and approximate entropy test. Since both share some code, this shared code
//! is defined here. The submodules are reexported in [crate::tests] for API consistency.

use crate::bitvec::BitVec;
use crate::BYTE_SIZE;

pub mod serial;
pub mod approximate_entropy;

/// Since the constraints for both test args are large the same, this function takes care of the validation.
fn validate_test_arg(block_length: u8) -> Option<u8> {
    // block length > 1 (else this is just the frequency test) and maximum of usize bits (32 or 64)
    if block_length > 1 && block_length as u32 <= usize::BITS {
        Some(block_length)
    } else {
        None
    }
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
        if end_bit_idx != 0 {
            last_byte(&mut number, data.data[end_byte_idx], end_bit_idx);
        }

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
        if end_bit_idx != 0 {
            last_byte(&mut number, data.data[end_byte_idx], end_bit_idx);
        }

        Some(number)
    }
}