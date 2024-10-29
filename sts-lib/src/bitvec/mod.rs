//! Everything needed to store the data to test.

use std::ffi::c_char;
use std::mem;
use std::ops::Deref;
use tinyvec::ArrayVec;
use sts_lib_derive::use_thread_pool;
use crate::bitvec::iter::{BitVecIterU8, ParBitVecIterU8};

pub mod array_chunks;
pub mod iter;

/// A list of bits, tightly packed - used in all tests
#[derive(Clone, Debug)]
pub struct BitVec {
    // data storage
    pub(crate) words: Box<[usize]>,
    // count of bits in the last word - maximum of usize::BITS - 1.
    pub(crate) bit_count_last_word: u8,
}

impl BitVec {
    /// How many bits the Vec contains
    pub fn len_bit(&self) -> usize {
        if self.bit_count_last_word == 0 {
            self.words.len() * (usize::BITS as usize)
        } else {
            (self.words.len() - 1) * (usize::BITS as usize) + (self.bit_count_last_word as usize)
        }
    }

    /// Crop the BitVec to the passed bit length. This operation does nothing
    /// if the previous length is greater than the new length.
    pub fn crop(&mut self, new_bit_len: usize) {
        if new_bit_len < self.len_bit() {
            let mut new_len = new_bit_len / (usize::BITS as usize);
            let additional_bits = (new_bit_len % (usize::BITS as usize)) as u8;

            if additional_bits > 0 {
                new_len += 1
            }

            let mut data = mem::take(&mut self.words).into_vec();
            data.truncate(new_len);
            if additional_bits > 0 {
                let mask = !((1 << (usize::BITS as u8 - additional_bits)) - 1);
                *data.last_mut().unwrap() &= mask;
            }
            self.words = data.into_boxed_slice();

            self.bit_count_last_word = additional_bits;
        }
    }

    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// No other character is allowed. [usize::MAX] bits can be read.
    ///
    /// This function runs in parallel.
    #[use_thread_pool(crate::internals::THREAD_POOL)]
    pub fn from_ascii_str(value: &str) -> Option<Self> {
        use rayon::iter::ParallelIterator;
        use rayon::slice::ParallelSlice;

        let words = value
            .as_bytes()
            .par_chunks(usize::BITS as usize)
            .map(|chunk| {
                // [0] = MSB
                chunk
                    .iter()
                    .enumerate()
                    .try_fold(0usize, |word, (i, char)| {
                        if *char == b'1' {
                            Some(word | (1 << ((usize::BITS as usize) - i - 1)))
                        } else if *char == b'0' {
                            // no need to change the value itself
                            Some(word)
                        } else {
                            None
                        }
                    })
            })
            .collect::<Option<_>>()?;

        let bit_count_last_word = (value.len() % (usize::BITS as usize)) as u8;

        Some(Self {
            words,
            bit_count_last_word,
        })
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

    /// Returns a list containing all full bytes of the BitVec, and 1 optional byte remainder.
    /// The remainder byte will be filled starting from the MSB, the count of bits in the remainder
    /// byte can be calculated using [Self::len_bit].
    ///
    /// This operation is expensive.
    #[use_thread_pool(crate::internals::THREAD_POOL)]
    pub fn to_bytes(&self) -> (Vec<u8>, Option<u8>) {
        use rayon::prelude::*;

        let (slice, value) = self.as_full_slice();

        let mut rest = None;
        let mut rest_for_iter = ArrayVec::new();

        if let Some(value) = value {
            let mut values = ArrayVec::from(value.to_be_bytes());

            for value in values
                .drain(..)
                .take((self.bit_count_last_word as usize) / (u8::BITS as usize))
            {
                rest_for_iter.push(value)
            }

            if (self.bit_count_last_word as usize) % (u8::BITS as usize) != 0 {
                rest = Some(values[0])
            }
        }

        let bytes = ParBitVecIterU8::new(BitVecIterU8::new(slice, rest_for_iter))
            .collect::<Vec<u8>>();

        (bytes, rest)
    }
}

// crate internals
impl BitVec {
    /// Returns the bits, stored as the given numerical primitives. The MSB of each value has the lowest index.
    /// Each value is filled - returns an optional additional value, that may be unfilled,
    pub(crate) fn as_full_slice(&self) -> (&[usize], Option<usize>) {
        let len = if self.bit_count_last_word == 0 {
            self.words.len()
        } else {
            self.words.len() - 1
        };

        (&self.words[..len], self.words.get(len).copied())
    }
}

// private functions
impl BitVec {
    /// Creates a [BitVec] from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
    /// Any other character is ignored. [usize::MAX] bits can be stored.
    /// If a max length is given, a maximum of `max_length` valid bits are read
    /// (not counting any invalid characters).
    ///
    /// This function runs sequential. (In contrast to [Self::from_ascii_str]).
    fn from_ascii_str_lossy_internal(value: &str, max_length: Option<usize>) -> Self {
        let mut full_words = Vec::new();
        let mut current_bit_idx = (usize::BITS as u8) - 1; // start with a wrap around
                                                           // we only need to increment if the length is relevant.
        let mut found_bit_len = max_length.map(|_| 0_usize);

        for char in value.bytes() {
            // only increment the current byte idx if the current value is a valid character
            if char == b'1' || char == b'0' {
                current_bit_idx += 1;
                found_bit_len = found_bit_len.map(|i| i + 1);

                if current_bit_idx == (usize::BITS as u8) {
                    // allocate an additional byte and reset index
                    current_bit_idx = 0;
                    full_words.push(0);
                }

                if char == b'1' {
                    // there is always at least 1 byte in the vec
                    if let Some(b) = full_words.last_mut() {
                        *b |= 1 << ((usize::BITS as usize) - (current_bit_idx as usize) - 1)
                    }
                }

                // if both values are equal (and Some())
                if found_bit_len == max_length && found_bit_len.is_some() {
                    break;
                }
            }
        }

        Self {
            words: full_words.into_boxed_slice(),
            bit_count_last_word: (current_bit_idx + 1) % (usize::BITS as u8),
        }
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

        let mut full_words = Vec::new();
        let mut current_bit_idx = (usize::BITS as u8) - 1; // start with a wrap around
        let mut found_bit_len = max_length.map(|_| 0_usize);

        // SAFETY: caller has provided a pointer to a valid C String.
        let mut current_value = unsafe { *ptr };
        while current_value != 0 {
            // only increment the current byte idx if the current value is a valid character
            if current_value == CHAR_1 || current_value == CHAR_0 {
                current_bit_idx += 1;
                found_bit_len = found_bit_len.map(|i| i + 1);

                if current_bit_idx == (usize::BITS as u8) {
                    // allocate an additional byte and reset index
                    current_bit_idx = 0;
                    full_words.push(0);
                }

                if current_value == CHAR_1 {
                    // there is always at least 1 byte in the vec
                    if let Some(b) = full_words.last_mut() {
                        *b |= 1 << ((usize::BITS as usize) - (current_bit_idx as usize) - 1)
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

        Self {
            words: full_words.into_boxed_slice(),
            bit_count_last_word: (current_bit_idx + 1) % (usize::BITS as u8),
        }
    }
}

// conversion functions
impl From<Vec<u8>> for BitVec {
    /// Creates a [BitVec] from a [Vec] of bytes, each containing 8 values.
    fn from(value: Vec<u8>) -> Self {
        Self::from(value.into_boxed_slice())
    }
}

impl<'a> From<&'a [u8]> for BitVec {
    /// Creates a [BitVec] from a slice of bytes, each containing 8 values.
    #[use_thread_pool(crate::internals::THREAD_POOL)]
    fn from(value: &'a [u8]) -> Self {
        use rayon::iter::ParallelIterator;
        use rayon::slice::ParallelSlice;

        const BYTES_PER_WORD: usize = (usize::BITS / u8::BITS) as usize;

        // multiplication in the first step would be unwise (overflow potential)
        let byte_count_last_word = (value.len() % BYTES_PER_WORD) as u8;
        let bit_count_last_word = byte_count_last_word * (u8::BITS as u8);

        // copy, converting to the right data type
        let words = value
            .par_chunks(BYTES_PER_WORD)
            .map(|chunk| {
                chunk.iter().enumerate().fold(0usize, |word, (i, byte)| {
                    let shift = (usize::BITS as usize) - ((u8::BITS as usize) * (i + 1));
                    word | (*byte as usize) << shift
                })
            })
            .collect();

        Self {
            words,
            bit_count_last_word,
        }
    }
}

impl From<Box<[u8]>> for BitVec {
    /// Creates a [BitVec] from a boxed slice of bytes, each containing 8 values.
    fn from(value: Box<[u8]>) -> Self {
        Self::from(value.deref())
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
    #[use_thread_pool(crate::internals::THREAD_POOL)]
    fn from(value: &'a [bool]) -> Self {
        use rayon::iter::ParallelIterator;
        use rayon::slice::ParallelSlice;

        let words = value
            .par_chunks(usize::BITS as usize)
            .map(|chunk| {
                // [0] = MSB
                chunk.iter().enumerate().fold(0usize, |word, (i, &bit)| {
                    word | ((bit as usize) << ((usize::BITS as usize) - i - 1))
                })
            })
            .collect();

        let bit_count_last_word = (value.len() % (usize::BITS as usize)) as u8;

        Self {
            words,
            bit_count_last_word,
        }
    }
}

impl AsRef<BitVec> for BitVec {
    fn as_ref(&self) -> &BitVec {
        self
    }
}
