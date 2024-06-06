//! Everything needed to store the data to test.

use std::ffi::{c_char};
use rayon::prelude::*;
use crate::BYTE_SIZE;

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

    /// Creates a [BitVec] from a string, which may only consists of the ASCII characters
    /// "0" and "1". If the String contains any other character, `None` is returned.
    pub fn from_ascii_str(value: &str) -> Option<Self> {
        // split into byte sized chunks and convert
        let byte_chunks = value.as_bytes().par_chunks_exact(BYTE_SIZE);

        // the remainder: smaller than 1 byte
        let remainder = byte_chunks.remainder().iter()
            .map(|&bit| {
                if bit == b'0' {
                    Some(false)
                } else if bit == b'1' {
                    Some(true)
                } else {
                    None
                }
            })
            .collect::<Option<Box<[bool]>>>()?;

        let data: Box<[u8]> = byte_chunks
            .map(|chunk| {
                // [0] = MSB
                // [7] = LSB
                (0..BYTE_SIZE).try_fold(0u8, |byte, i| {
                    if chunk[i] == b'1' {
                        Some(byte | (1 << (BYTE_SIZE - i - 1)))
                    } else if chunk[i] == b'0' {
                        // no need to change the byte itself
                        Some(byte)
                    } else {
                        None
                    }
                })
            })
            .collect::<Option<_>>()?;

        Some(Self {
            data,
            remainder
        })
    }

    /// Creates a [BitVec] from a string, which may only consists of the ASCII characters
    /// "0" and "1". If the String contains any other character, `None` is returned.
    ///
    /// ## Safety
    /// Similar restrictions apply as for [CStr::from_ptr](std::ffi::CStr::from_ptr):
    /// * The memory pointed to by `ptr` must contain a valid nul terminator at the end of the string.
    /// * `ptr` must be valid, as defined by the module safety documentation of `std::ptr`, for reads
    ///   of bytes up to and including the nul terminator.
    ///     * The entire memory range must be contained within a single allocated object!
    /// * `ptr` must have at least length 1: the nul terminator.
    /// * The memory referenced by `ptr` must not be mutated for the duration of this method call.
    /// * `ptr`, particularly the de-allocation of it, remains in the responsibility of the caller.
    ///
    /// Note that the nul terminator **DOES NOT** need to be within [isize::MAX] from `ptr`.
    /// Every valid [CStr](std::ffi::CStr) is a valid pointer for this method.
    pub unsafe fn from_c_str(mut ptr: *const c_char) -> Option<Self> {
        const CHAR_0: c_char = b'0' as c_char;
        const CHAR_1: c_char = b'1' as c_char;

        let mut full_bytes= Vec::new();
        let mut current_byte_idx = BYTE_SIZE - 1; // start with a wrap around

        // SAFETY: caller has provided a pointer to a valid C String.
        let mut current_value = unsafe { *ptr };
        while current_value != 0 {
            current_byte_idx += 1;

            if current_byte_idx == BYTE_SIZE {
                // allocate an additional byte and reset index
                current_byte_idx = 0;
                full_bytes.push(0);
            }

            if current_value == CHAR_1 {
                // there is always at least 1 byte in the vec
                *full_bytes.last_mut().unwrap() |= 1 << (BYTE_SIZE - current_byte_idx - 1);
            } else if current_value != CHAR_0 {
                // current character is not "1" or "0"
                return None;
            }


            // SAFETY: caller has provided a pointer to a valid C String, and the end
            // has not yet been reached (otherwise current_value would be 0)
            unsafe {
                ptr = ptr.add(1);
                current_value = *ptr;
            };
        }

        // string has ended - if the last byte is incomplete, it has to be saved separately
        let remainder: Box<[bool]> = if current_byte_idx != BYTE_SIZE {
            // vec cannot be empty
            let byte = full_bytes.pop().unwrap();

            (0..=current_byte_idx)
                .map(|idx| {
                    ((byte >> (BYTE_SIZE - idx - 1)) & 0x01) != 0
                })
                .collect()
        } else {
            Box::new([])
        };

        Some(Self {
            data: full_bytes.into_boxed_slice(),
            remainder,
        })
    }
}

impl From<Vec<u8>> for BitVec {
    /// Creates a [BitVec] from a [Vec] of bytes, each containing 8 values.
    fn from(value: Vec<u8>) -> Self {
        Self::from(value.into_boxed_slice())
    }
}

impl<'a> From<&'a [u8]> for BitVec {
    /// Creates a [BitVec] from a slice of bytes, each containing 8 values.
    fn from(value: &'a [u8]) -> Self {
        Self {
            data: value.into(),
            remainder: Box::new([]), // no allocation
        }
    }
}

impl From<Box<[u8]>> for BitVec {
    /// Creates a [BitVec] from a boxed slice of bytes, each containing 8 values.
    fn from(value: Box<[u8]>) -> Self {
        Self {
            data: value,
            remainder: Box::new([]), // no allocation
        }
    }
}

impl From<Vec<bool>> for BitVec {
    /// Creates a [BitVec] from a [Vec] of booleans, each boolean representing one bit.
    fn from(value: Vec<bool>) -> Self {
        Self::from(value.as_slice())
    }
}

impl<'a> From<&'a [bool]> for BitVec {
    /// Creates a [BitVec] from a slice of booleans, each boolean representing one bit.
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