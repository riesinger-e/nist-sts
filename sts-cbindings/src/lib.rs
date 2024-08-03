#![doc = include_str!("../README.md")]

// TODO: create test runner infrastructure - pointers only?

pub mod bitvec;
pub mod test_args;
pub mod test_result;
pub mod tests;

use std::cell::RefCell;
use std::ffi::{c_char, c_int};
use std::ptr::slice_from_raw_parts_mut;
use sts_lib::Error;
use sts_lib::test_runner::RunnerError;

thread_local! {
    /// This variable stores the Display impl of the last error.
    static LAST_ERROR: RefCell<Option<(ErrorCode, String)>> = const { RefCell::new(None) };
}

/// Returns the last error that happened in the calling thread. This function works in 2 steps:
/// 1. the caller calls the function with `ptr` set to `NULL`. The necessary length is written to
///    `len`.
/// 2. the caller calls the function with `ptr` set to a valid buffer, and `len` set to the length of
///    the buffer. If the length is enough to store the error message, it is written to the buffer.
///    The error message is written with a nul-terminating byte.
///
/// ## Return values
///
/// - >0: the [ErrorCode] of the last error. Everything worked.
/// - 0: there is no error in storage.
/// - -1: the passed string buffer is too small.
///
/// ## Safety
///
/// * `len` must not be `NULL`.
/// * `ptr` must be valid for writes of up to `len` bytes.
/// * `ptr` may not be mutated for the duration of this call.
/// * All responsibility for `ptr` and `len`, especially for its de-allocation, remains with the caller.
#[no_mangle]
pub unsafe extern "C" fn get_last_error_str(ptr: *mut c_char, len: &mut usize) -> c_int {
    // check if there is an error
    if LAST_ERROR.with_borrow(|e| e.is_none()) {
        return 0;
    }

    // LAST_ERROR is guaranteed to be Some, we just checked. + 1 for the nul byte
    let (error_code, needed_length) = LAST_ERROR.with_borrow(|e| {
        let (error_code, msg) = e.as_ref().unwrap();
        (*error_code, msg.as_bytes().len() + 1)
    });

    if ptr.is_null() {
        // caller only asks for the length
        *len = needed_length;
        error_code as c_int
    } else {
        // caller wants the error message

        // check length
        if *len < needed_length {
            -1
        } else {
            // length is OK, write the String
            // again: LAST_ERROR is guaranteed to be Some
            let error_msg = LAST_ERROR.with_borrow_mut(|e| e.take()).unwrap().1;

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
                // it doesn't hurt to have the max length set explicitly
                .zip(&mut slice[..(*len - 1)])
                .for_each(|(input, output)| *output = *input);

            error_code as c_int
        }
    }
}

/// Sets the maximum of threads to be used by the tests. These method can only be called ONCE and only
/// BEFORE any test is started. If not used, a sane default will be chosen.
///
/// If called multiple times or after the first test, an error will be returned.
///
/// Since this library uses [rayon](https://docs.rs/rayon/latest/rayon/index.html), this function
/// effectively calls
/// [ThreadPoolBuilder::num_threads](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html#method.num_threads).
/// If you use rayon in the calling code, no rayon workload may have been run before calling this
/// function.
///
/// ## Return values
///
/// - 0: the call worked.
/// - 1: an error happened
#[no_mangle]
pub extern "C" fn set_max_threads(max_threads: usize) -> c_int {
    match sts_lib::set_max_threads(max_threads) {
        Ok(()) => 0,
        Err(e) => {
            LAST_ERROR
                .with_borrow_mut(|err| *err = Some((ErrorCode::SetMaxThreads, e.to_string())));
            1
        }
    }
}

/// The error codes that are returned by some fallible functions.
/// A human-readable error message can be retrieved with [get_last_error_str].
/// cbindgen:prefix-with-name=true
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum ErrorCode {
    /// No error
    NoError = 0,
    /// A numeric overflow happened in the called test.
    Overflow = 1,
    /// The result of a test was `NaN`
    NaN = 2,
    /// The result of a test was (positive or negative) Infinity.
    Infinite = 3,
    /// The gamma function used in a test failed.
    GammaFunctionFailed = 4,
    /// A test was called with an invalid parameter (value-wise, references are not checked!).
    InvalidParameter = 5,
    /// The function [set_max_threads] failed.
    SetMaxThreads = 6,
    /// A test passed to the test runner is invalid (Invalid value).
    InvalidTest = 7,
    /// A test was specified multiple times in the same call to the test runner.
    DuplicateTest = 8,
    /// One or multiple tests that were run with the test runner failed.
    TestFailed = 9,
}

/// Sets the last error from the specified [Error].
fn set_last_from_error(error: Error) {
    let (code, msg) = match error {
        e @ Error::Overflow(_) => (ErrorCode::Overflow, e.to_string()),
        e @ Error::NaN => (ErrorCode::NaN, e.to_string()),
        e @ Error::Infinite => (ErrorCode::Infinite, e.to_string()),
        e @ Error::GammaFunctionFailed(_) => (ErrorCode::GammaFunctionFailed, e.to_string()),
        e @ Error::InvalidParameter(_) => (ErrorCode::InvalidParameter, e.to_string()),
    };

    LAST_ERROR.with_borrow_mut(|e| *e = Some((code, msg)));
}

/// Sets the last error code from the specified [RunnerError].
fn set_last_from_runner_error(error: RunnerError) {
    let (code, msg) = match error {
        e @ RunnerError::Duplicate(_) => (ErrorCode::DuplicateTest, e.to_string()),
        e @ RunnerError::Test(_) => (ErrorCode::TestFailed, e.to_string()),
    };

    LAST_ERROR.with_borrow_mut(|e| *e = Some((code, msg)));
}