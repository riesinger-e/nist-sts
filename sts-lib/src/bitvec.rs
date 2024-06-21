//! Everything needed to store the data to test.

use crate::BYTE_SIZE;
use rayon::prelude::*;
use std::ffi::c_char;
use std::mem;

/// A list of bits, tightly packed - used in all tests
#[derive(Clone)]
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

    /// Crop the BitVec to the passed bit length. This operation does nothing
    /// if the previous length is greater than the new length.
    pub fn crop(&mut self, new_bit_len: usize) {
        if new_bit_len < self.len_bit() {
            let new_byte_len = new_bit_len / BYTE_SIZE;
            let rem_bit_len = new_bit_len % BYTE_SIZE;

            if new_byte_len < self.data.len() {
                // for the remainder, use the byte after the last
                let rem_byte = self.data[new_byte_len];

                let mut data = mem::take(&mut self.data).into_vec();
                data.truncate(new_byte_len);
                self.data = data.into_boxed_slice();

                let remainder = (0..rem_bit_len)
                    .map(|idx| ((rem_byte >> (BYTE_SIZE - 1 - idx)) & 0x01) == 1)
                    .collect();
                self.remainder = remainder;
            } else {
                // self.data does not need to be truncated
                // for the remainder, use self.remainder
                let mut remainder = mem::take(&mut self.remainder).into_vec();
                remainder.truncate(rem_bit_len);
                self.remainder = remainder.into_boxed_slice();
            }
        }
    }

    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// No other character is allowed. [usize::MAX] bits can be read.
    ///
    /// This function runs in parallel.
    pub fn from_ascii_str(value: &str) -> Option<Self> {
        // split into byte sized chunks and convert
        let byte_chunks = value.as_bytes().par_chunks_exact(BYTE_SIZE);

        // the remainder: smaller than 1 byte
        let remainder = byte_chunks
            .remainder()
            .iter()
            .map(|&bit| {
                if bit == b'0' {
                    Some(false)
                } else if bit == b'1' {
                    Some(true)
                } else {
                    None
                }
            })
            .collect::<Option<_>>()?;

        let data = byte_chunks
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

        Some(Self { data, remainder })
    }

    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// Any other character is ignored. [usize::MAX] bits can be read.
    ///
    /// This function runs sequential. (In contrast to [Self::from_ascii_str]).
    pub fn from_ascii_str_lossy(value: &str) -> Self {
        Self::from_ascii_str_lossy_internal(value, None)
    }

    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// Any other character is ignored. [usize::MAX] bits can be stored.
    /// A maximum of `max_length` valid bits are read (not counting any invalid characters).
    ///
    /// This function runs sequential. (In contrast to [Self::from_ascii_str]).
    pub fn from_ascii_str_lossy_with_max_length(value: &str, max_length: usize) -> Self {
        Self::from_ascii_str_lossy_internal(value, Some(max_length))
    }

    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// Any other character is ignored. [usize::MAX] bits can be stored.
    /// If a max length is given, a maximum of `max_length` valid bits are read
    /// (not counting any invalid characters).
    ///
    /// This function runs sequential. (In contrast to [Self::from_ascii_str]).
    fn from_ascii_str_lossy_internal(value: &str, max_length: Option<usize>) -> Self {
        let mut full_bytes = Vec::new();
        let mut current_byte_idx = BYTE_SIZE - 1; // start with a wrap around
        let mut found_bit_len = max_length.map(|_| 0_usize);

        for char in value.bytes() {
            // only increment the current byte idx if the current value is a valid character
            if char == b'1' || char == b'0' {
                current_byte_idx += 1;
                found_bit_len = found_bit_len.map(|i| i + 1);

                if current_byte_idx == BYTE_SIZE {
                    // allocate an additional byte and reset index
                    current_byte_idx = 0;
                    full_bytes.push(0);
                }

                if char == b'1' {
                    // there is always at least 1 byte in the vec
                    if let Some(b) = full_bytes.last_mut() {
                        *b |= 1 << (BYTE_SIZE - current_byte_idx - 1)
                    }
                }

                // if both values are equal (and Some())
                if found_bit_len == max_length && found_bit_len.is_some() {
                    break;
                }
            }
        }

        // string has ended - if the last byte is incomplete, it has to be saved separately
        Self::remainder_from_data_and_byte_idx(full_bytes, current_byte_idx)
    }

    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// Any other character is ignored.
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
    /// Note that the nul terminator **DOES NOT** need to be within [isize::MAX] from `ptr`, but
    /// instead within [usize::MAX] * 8 + 7.
    /// Every valid [CStr](std::ffi::CStr) is a valid pointer for this method.
    pub unsafe fn from_c_str(ptr: *const c_char) -> Self {
        // SAFETY: for the call of the function, the same safety considerations apply
        // as for the call of this function.
        Self::from_c_str_internal(ptr, None)
    }

    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// Any other character is ignored.  A maximum of `max_length` valid bits are read
    /// (not counting any invalid characters). This also means that the maximum valid bit length here
    /// is [usize::MAX].
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
    /// Note that the nul terminator **DOES NOT** need to be within [isize::MAX] from `ptr`, but
    /// instead within [usize::MAX] * 8 + 7.
    /// Every valid [CStr](std::ffi::CStr) is a valid pointer for this method.
    pub unsafe fn from_c_str_with_max_length(ptr: *const c_char, max_length: usize) -> Self {
        // SAFETY: for the call of the function, the same safety considerations apply
        // as for the call of this function.
        Self::from_c_str_internal(ptr, Some(max_length))
    }

    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// Any other character is ignored.  If a `max_length` is given, a maximum of `max_length` valid
    /// bits are read (not counting any invalid characters) and the maximum bit length ist
    /// [usize::MAX].
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
    /// Note that the nul terminator **DOES NOT** need to be within [isize::MAX] from `ptr`, but
    /// instead within [usize::MAX] * 8 + 7.
    /// Every valid [CStr](std::ffi::CStr) is a valid pointer for this method.
    unsafe fn from_c_str_internal(mut ptr: *const c_char, max_length: Option<usize>) -> Self {
        const CHAR_0: c_char = b'0' as c_char;
        const CHAR_1: c_char = b'1' as c_char;

        let mut full_bytes = Vec::new();
        let mut current_byte_idx = BYTE_SIZE - 1; // start with a wrap around
        let mut found_bit_len = max_length.map(|_| 0_usize);

        // SAFETY: caller has provided a pointer to a valid C String.
        let mut current_value = unsafe { *ptr };
        while current_value != 0 {
            // only increment the current byte idx if the current value is a valid character
            if current_value == CHAR_1 || current_value == CHAR_0 {
                current_byte_idx += 1;
                found_bit_len = found_bit_len.map(|i| i + 1);

                if current_byte_idx == BYTE_SIZE {
                    // allocate an additional byte and reset index
                    current_byte_idx = 0;
                    full_bytes.push(0);
                }

                if current_value == CHAR_1 {
                    // there is always at least 1 byte in the vec
                    if let Some(b) = full_bytes.last_mut() {
                        *b |= 1 << (BYTE_SIZE - current_byte_idx - 1)
                    }
                }

                // if both values are equal (and Some())
                if found_bit_len == max_length && found_bit_len.is_some() {
                    break;
                }
            }

            // look at next value.
            // SAFETY: caller has provided a pointer to a valid C String, and the end
            // has not yet been reached (otherwise current_value would be 0)
            unsafe {
                ptr = ptr.add(1);
                current_value = *ptr;
            };
        }

        // string has ended - if the last byte is incomplete, it has to be saved separately
        Self::remainder_from_data_and_byte_idx(full_bytes, current_byte_idx)
    }

    /// Creates an instance from a byte array that may have an incomplete last byte -
    /// used by [Self::from_c_str_internal] and [Self::from_ascii_str_lossy_internal].
    fn remainder_from_data_and_byte_idx(mut data: Vec<u8>, current_byte_idx: usize) -> Self {
        let remainder = if current_byte_idx != BYTE_SIZE {
            // vec cannot be empty
            let byte = data.pop().unwrap();

            (0..=current_byte_idx)
                .map(|idx| ((byte >> (BYTE_SIZE - idx - 1)) & 0x01) != 0)
                .collect()
        } else {
            Default::default()
        };

        Self {
            data: data.into_boxed_slice(),
            remainder,
        }
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
            remainder: Default::default(), // no allocation
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

        let data = byte_chunks
            .map(|chunk| {
                // [0] = MSB
                // [7] = LSB
                (0..BYTE_SIZE).fold(0u8, |byte, i| {
                    byte | ((chunk[i] as u8) << (BYTE_SIZE - i - 1))
                })
            })
            .collect();

        Self { data, remainder }
    }
}
