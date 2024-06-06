//! Everything needed to store the data to test.

use rayon::prelude::*;

/// How many bits a byte has
const BYTE_SIZE: usize = 8;

/// A list of bits, tightly packed - used in all tests
pub struct BitVec {
    // the main, compact, data storage
    pub(crate) data: Box<[u8]>,
    // additional bits that are not a full byte
    pub(crate) remainder: Box<[bool]>,
}

impl BitVec {
    /// How many bits the Vec contains
    pub fn len_bit(&self) -> usize {
        self.data.len() * BYTE_SIZE + self.remainder.len()
    }
}

impl From<Vec<u8>> for BitVec {
    fn from(value: Vec<u8>) -> Self {
        Self {
            data: value.into_boxed_slice(),
            remainder: Box::new([]), // no allocation
        }
    }
}

impl<'a> From<&'a [u8]> for BitVec {
    fn from(value: &'a [u8]) -> Self {
        Self {
            data: value.into(),
            remainder: Box::new([]), // no allocation
        }
    }
}

impl From<Vec<bool>> for BitVec {
    fn from(value: Vec<bool>) -> Self {
        Self::from(value.as_slice())
    }
}

impl<'a> From<&'a [bool]> for BitVec {
    fn from(value: &'a [bool]) -> Self {
        // split into byte sized chunks and convert
        let byte_chunks = value.par_chunks_exact(BYTE_SIZE);

        // the remainder: smaller than 1 byte
        let remainder = byte_chunks.remainder().into();

        let data: Box<[u8]> = byte_chunks
            .map(|chunk| {
                // [0] = MSB
                // [7] = LSB
                (0..BYTE_SIZE).fold(0u8, |byte, i| {
                    byte | ((chunk[i] as u8) << (BYTE_SIZE - i - 1))
                })
            })
            .collect();

        Self {
            data,
            remainder
        }
    }
}