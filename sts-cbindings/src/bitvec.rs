//! All exported functions related to a [BitVec].
//! BitVec is a container for storing bit sequences for use in testing.

use std::ffi::c_char;
use std::ptr::slice_from_raw_parts;

use sts_lib::bitvec::BitVec as InternalBitVec;

/// BitVec: a list of bits to run statistical tests on.
#[derive(Clone)]
pub struct BitVec(pub(crate) InternalBitVec);

/// Creates a Bit Vector from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
/// Any other character is ignored.
///
/// ## Safety
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the end of the string.
/// * `ptr` must be valid, as defined by the Rust module safety documentation of `std::ptr`, for reads
///   of bytes up to and including the nul terminator.
///     * The entire memory range must be contained within a single allocated object!
/// * `ptr` must have at least length 1: the nul terminator.
/// * The memory referenced by `ptr` must not be mutated for the duration of this method call.
/// * `ptr`, particularly the de-allocation of it, remains in the responsibility of the caller.
/// * The de-allocation of the returned [BitVec] must be done via [bitvec_destroy].
#[no_mangle]
pub unsafe extern "C" fn bitvec_from_str(ptr: *const c_char) -> Box<BitVec> {
    // SAFETY: it is the responsibility of the caller to ensure that the safety requirements are met.
    let bitvec = unsafe { InternalBitVec::from_c_str(ptr) };
    Box::new(BitVec(bitvec))
}

/// Same as [bitvec_from_str], but allows to specify a maximum count of bits to read from the
/// string. When this limit is reached, the String will not be read any further.
///
/// ## Safety
///
/// The same safety considerations apply as for [bitvec_from_str]
#[no_mangle]
pub unsafe extern "C" fn bitvec_from_str_with_max_length(
    ptr: *const c_char,
    max_length: usize,
) -> Box<BitVec> {
    // SAFETY: caller has to ensure that the requirements of the function are met.
    let bitvec = unsafe { InternalBitVec::from_c_str_with_max_length(ptr, max_length) };
    Box::new(BitVec(bitvec))
}

/// Creates a BitVec from a byte array, where each byte is filled with 8 bits.
///
/// ## Safety
///
/// * The memory pointed to by `ptr` must be valid for reads of up to `len` bytes.
/// * The memory referenced by `ptr` must not be mutated for the duration of this method call.
/// * `ptr`, particularly the de-allocation of it, remains in the responsibility of the caller.
/// * The de-allocation of the returned [BitVec] must be done via [bitvec_destroy].
#[no_mangle]
pub unsafe extern "C" fn bitvec_from_bytes(ptr: *const u8, len: usize) -> Box<BitVec> {
    // SAFETY: caller has to ensure that ptr is valid for reads up to len bytes / elements.
    let slice = unsafe { &*slice_from_raw_parts(ptr, len) };

    let bitvec = InternalBitVec::from(slice);
    Box::new(BitVec(bitvec))
}

/// Creates a BitVec from a bool array, with each bool representing one bit.
///
/// ## Safety
///
/// * The memory pointed to by `ptr` must be valid for reads of up to `len` elements.
/// * The memory referenced by `ptr` must not be mutated for the duration of this method call.
/// * `ptr`, particularly the de-allocation of it, remains in the responsibility of the caller.
/// * The de-allocation of the returned [BitVec] must be done via [bitvec_destroy].
#[no_mangle]
pub unsafe extern "C" fn bitvec_from_bits(ptr: *const bool, len: usize) -> Box<BitVec> {
    // SAFETY: caller has to ensure that ptr is valid for reads up to len bytes / elements.
    let slice = unsafe { &*slice_from_raw_parts(ptr, len) };

    let bitvec = InternalBitVec::from(slice);
    Box::new(BitVec(bitvec))
}

/// Destroys a created BitVec.
///
/// ## Safety
///
/// * `bitvec` must have been created by either [bitvec_from_str], [bitvec_from_str_with_max_length],
///   [bitvec_from_bytes] or [bitvec_from_bits].
/// * `bitvec` must be a valid pointer.
/// * `bitvec` may not be mutated for the duration of this call..
#[no_mangle]
pub unsafe extern "C" fn bitvec_clone(bitvec: &BitVec) -> Box<BitVec> {
    Box::new(bitvec.clone())
}

/// Destroys a created BitVec.
///
/// ## Safety
///
/// * `bitvec` must have been created by either [bitvec_from_str], [bitvec_from_str_with_max_length],
///   [bitvec_from_bytes], [bitvec_from_bits] or [bitvec_clone].
/// * `bitvec` may be null.
/// * There must be no other references to `bitvec`.
/// * After this call, the memory referenced by `bitvec` is freed. Trying to access this memory
///   will lead to undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn bitvec_destroy(bitvec: Option<Box<BitVec>>) {
    // this drops the BitVec
    _ = bitvec;
}

/// Returns the count of bits in the BitVec.
///
/// ## Safety
///
/// * `bitvec` must have been created by either [bitvec_from_str], [bitvec_from_str_with_max_length],
///   [bitvec_from_bytes], [bitvec_from_bits] or [bitvec_clone].
/// * `bitvec` must be a valid, non-null pointer.
/// * `bitvec` may not be mutated for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn bitvec_len_bit(bitvec: &BitVec) -> usize {
    bitvec.0.len_bit()
}

/// Crops the BitVec to the given count of bits. Values for `new_bit_len` that are larger than the
/// current bit length will do nothing.
///
/// ## Safety
///
/// * `bitvec` must have been created by either [bitvec_from_str], [bitvec_from_str_with_max_length],
///   [bitvec_from_bytes], [bitvec_from_bits] or [bitvec_clone].
/// * `bitvec` must be a valid, non-null pointer.
/// * `bitvec` may not be mutated by other functions for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn bitvec_crop(bitvec: &mut BitVec, new_bit_len: usize) {
    bitvec.0.crop(new_bit_len)
}