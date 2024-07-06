use std::cmp::Ordering;
use crate::BYTE_SIZE;

/// An implementation of the Berlekamp-Massey algorithm for calculating the linear complexity of a
/// bit sequence, according to the Handbook of Applied Cryptography, p. 201, 6.30.
///
/// This implementation is optimized to reduce complexity.
///
/// Inputs: the sequence stored as packed binary (8 bits per byte) + 1 optional byte additional bits,
/// the bit length of the sequence to calculate the linear complexity for, the start bit in the
/// sequence.
pub(crate) fn berlekamp_massey(sequence: &[u8], additional_bits: Option<u8>, total_bit_len: usize, start_bit: usize) -> usize {
    // Initialize C(D) - saves the values of a binary polynom
    let mut c: Vec<bool> = vec![false; total_bit_len];
    // the linear complexity
    let mut l = 0_usize;
    // the value m
    let mut m = -1_isize;
    // B(D) - binary polynom
    let mut b: Vec<bool> = Vec::new();

    for n in 0..total_bit_len {
        // compute discrepancy
        let mut sum = false;
        for i in 1..(l + 1) {
            sum ^= c[i - 1] & get_bit(sequence, additional_bits, start_bit + n - i).unwrap();
        }

        let s_n = get_bit(sequence, additional_bits, start_bit + n)
            .unwrap();
        let d = s_n ^ sum;

        if d {
            let t = c.clone();

            // addition of polynoms: shift is the power
            let shift = (n as isize) - m;
            for (idx, &bit) in b.iter().enumerate() {
                let new_idx = ((idx as isize) + shift) as usize;
                if new_idx == c.len() {
                    break;
                }
                c[new_idx] ^= bit;
            }
            // xor the 1 at the beginning of the polynom
            c[(shift - 1) as usize] ^= true;

            if l <= n / 2 {
                l = n + 1 - l;
                m = n as isize;
                b = t;
            }
        }
    }

    l
}

/// Get the bit in a sequence + optional additional bits at the given position
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

#[test]
fn test_berlekamp_massey() {
    let sequence = [0b1101_0111, 0b1000_1000];
    let bit_len = 13;
    let start_bit = 0;

    assert_eq!(berlekamp_massey(&sequence, None, bit_len, start_bit), 4);
}