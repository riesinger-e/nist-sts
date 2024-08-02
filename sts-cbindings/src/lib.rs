#![doc = include_str!("../README.md")]

// TODO: create test runner infrastructure - pointers only?

pub mod bitvec;
pub mod tests;
pub mod test_result;
pub mod test_args;

use std::cell::RefCell;
use std::ffi::{c_char, c_int};
use std::ptr::slice_from_raw_parts_mut;

thread_local! {
    /// This variable stores the Display impl of the last error.
    static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Returns the last error that happened in the calling thread. This function works in 2 steps:
/// 1. the caller calls the function with `ptr` set to `NULL`. The necessary length is written to
///    `len`.
/// 2. the caller calls the function with `ptr` set to a valid buffer, and `len` set to the length of
///    the buffer. If the length is enough to store the error message, it is written to the buffer.
///    The error message is written with a nul-terminating byte.
///
/// # Return values
///
/// - 0: everything's OK.
/// - 1: there is no error to store.
/// - 2: the passed string buffer is too small.
///
/// # Safety
///
/// * `len` must not be `NULL`.
/// * `ptr` must be valid for writes of up to `len` bytes.
/// * `ptr` may not be mutated for the duration of this call.
/// * All responsibility for `ptr` and `len`, especially for its de-allocation, remains with the caller.
#[no_mangle]
pub unsafe extern "C" fn get_last_error_str(ptr: *mut c_char, len: &mut usize) -> c_int {
    // check if there is an error
    if LAST_ERROR.with_borrow(|e| e.is_none()) {
        return 1;
    }

    // LAST_ERROR is guaranteed to be Some, we just checked. + 1 for the nul byte
    let needed_length = LAST_ERROR.with_borrow(|e| e.as_ref().unwrap().as_bytes().len()) + 1;

    if ptr.is_null() {
        // caller only asks for the length

        *len = needed_length;
        0
    } else {
        // caller wants the error message

        // check length
        if *len < needed_length {
            2
        } else {
            // length is OK, write the String
            // again: LAST_ERROR is guaranteed to be Some
            let error_msg = LAST_ERROR.with_borrow_mut(|e| e.take()).unwrap();

            // convert the buffer into a suitable type
            let buffer = slice_from_raw_parts_mut(ptr as *mut u8, *len);
            // SAFETY: it is the responsibility of the caller to ensure that the pointer is valid for
            //  writes of up to len bytes.
            let slice = unsafe { &mut *buffer };
            // set last NUL byte
            slice[*len - 1] = 0;
            // set message
            error_msg
                .as_bytes()
                .iter()
                .zip(slice)
                .for_each(|(input, output)| *output = *input);

            0
        }
    }
}


