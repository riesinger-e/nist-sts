//! The serial test and approximate entropy test. Since both share some code, this shared code
//! is defined here. The submodules are reexported in [crate::tests] for API consistency.

use crate::bitvec::BitVec;
use crate::BYTE_SIZE;

pub mod approximate_entropy;
pub mod serial;

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
/// Bounds: start_idx < [BitVec::len_bit], block_length <= [usize::BITS]
fn access_bits(data: &BitVec, start_idx: usize, block_length: u8) -> Option<usize> {
    let data_len = data.len_bit();

    if start_idx >= data_len || block_length as u32 > usize::BITS {
        return None;
    }

    let mut number: usize = 0;
    let mut bits_left = block_length;
    let mut idx = start_idx;

    while bits_left > 0 {
        idx %= data_len;

        let byte_idx = idx / BYTE_SIZE;

        if byte_idx < data.data.len() {
            let bit_idx = idx % BYTE_SIZE;
            let byte = data.data[byte_idx];

            if bit_idx == 0 && bits_left >= BYTE_SIZE as u8 {
                // still inside the byte array, byte aligned storing possible
                bits_left -= BYTE_SIZE as u8;

                number |= (byte as usize) << bits_left;

                idx += BYTE_SIZE;
            } else {
                // still inside the byte array, but not byte aligned
                bits_left -= 1;

                let bit = ((byte >> (BYTE_SIZE - bit_idx - 1)) & 1) != 0;

                if bit {
                    number |= 1 << bits_left;
                }

                idx += 1;
            }
        } else {
            // in data.remainder
            bits_left -= 1;

            let rem_idx = idx - data.data.len() * BYTE_SIZE;

            if data.remainder[rem_idx] {
                number |= 1 << bits_left;
            }

            idx += 1;
        }
    }

    Some(number)
}
