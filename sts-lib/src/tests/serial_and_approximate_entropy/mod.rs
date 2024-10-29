//! The serial test and approximate entropy test. Since both share some code, this shared code
//! is defined here. The submodules are reexported in [crate::tests] for API consistency.

use crate::bitvec::BitVec;

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
/// This function may wrap-around, meaning if start_idx + block_length >= data.len_bit(), bits from
/// the start will be read.
///
/// Bounds: start_idx < [BitVec::len_bit], block_length <= [usize::BITS]
fn access_bits(data: &BitVec, start_idx: usize, block_length: u8) -> Option<usize> {
    let data_len = data.len_bit();

    if start_idx >= data_len || block_length as u32 > usize::BITS {
        return None;
    }

    let end_idx = start_idx + block_length as usize;

    let res = if end_idx >= data_len {
        // wrap-around
        let end_idx = end_idx % data_len;
        let end_bit_idx = end_idx % (usize::BITS as usize);

        // read all bits from start_idx to the end (1 or 2 words)
        let start_word_idx = start_idx / (usize::BITS as usize);
        let start_bit_idx = start_idx % (usize::BITS as usize);

        // first part is either last word or second-to-last word
        let mut result: usize = data.words[start_word_idx] << start_bit_idx;

        if start_word_idx != data.words.len() - 1 {
            // add last word, if necessary
            result |= data.words[data.words.len() - 1] >> (usize::BITS as usize - start_bit_idx);
        }

        // add first word
        let shift = (block_length as usize) - end_bit_idx;
        let last_part = data.words[0] >> shift;

        result | last_part
    } else {
        // read "normally", maximum of 2 words possible
        let start_word_idx = start_idx / (usize::BITS as usize);
        let end_word_idx = end_idx / (usize::BITS as usize);

        let start_bit_idx = start_idx % (usize::BITS as usize);

        let first_part = data.words[start_word_idx] << start_bit_idx;

        if start_word_idx == end_word_idx {
            first_part
        } else {
            let second_part = data.words[end_word_idx] >> (usize::BITS as usize - start_bit_idx);
            first_part | second_part
        }
    };

    // now the block_size high bits contain the value --> shift to low bits
    let res = res >> (usize::BITS as u8 - block_length);

    Some(res)
}
