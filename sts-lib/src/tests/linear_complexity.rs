use crate::BYTE_SIZE;
use std::cmp::Ordering;

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
    // the linear complexity
    let mut l = 0_usize;
    // the value m
    let mut m = -1_isize;
    // B(D) - binary polynom
    let mut b: Vec<u8> = Vec::new();

    // for all following calculations:
    // In a base 2 system, PLUS is the same as XOR and MULT is the same as AND.
    // Raising to the power can be done with bit shifts.
    for n in 0..total_bit_len {
        // compute discrepancy
        let mut sum = false;
        for i in 1..(l + 1) {
            sum ^= get_bit(&c, None, i - 1).unwrap()
                & get_bit(sequence, additional_bits, start_bit + n - i).unwrap();
        }

        let s_n = get_bit(sequence, additional_bits, start_bit + n).unwrap();
        let d = s_n ^ sum;

        if d {
            let t = c.clone();

            // addition of polynoms: shift is the power
            let shift = (n as isize) - m;
            for (idx, bit) in b.iter().enumerate() {
                let mut shifted_value = bit >> shift;
                if idx == 0 {
                    // add the 1 at the beginning of the polynomial
                    shifted_value |= 1 << (BYTE_SIZE as isize - shift)
                }

                let carry_over = bit << (BYTE_SIZE as isize - shift);

                c[idx] ^= shifted_value;
                if c.len() < idx + 1 {
                    c[idx + 1] ^= carry_over;
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
