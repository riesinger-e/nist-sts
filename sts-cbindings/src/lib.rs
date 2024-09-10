#![doc = include_str!("../README.md")]

pub mod bitvec;
pub mod constants;
pub mod test_args;
pub mod test_result;
pub mod test_runner;
pub mod tests;

use crate::test_runner::test::RawTest;
use std::cell::RefCell;
use std::ffi::{c_char, c_int};
use std::num::NonZero;
use std::slice;
use sts_lib::test_runner::RunnerError;

thread_local! {
    /// This variable stores the Display impl of the last error.
    static LAST_ERROR: RefCell<(ErrorCode, String)> = const { RefCell::new((ErrorCode::NoError, String::new())) };
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
pub unsafe extern "C" fn get_last_error(ptr: *mut c_char, len: &mut usize) -> c_int {
    // check if there is an error
    if LAST_ERROR.with_borrow(|(e, _)| matches!(e, ErrorCode::NoError)) {
        return 0;
    }

    // LAST_ERROR is guaranteed to contain an error, we just checked. + 1 for the nul byte
    let (error_code, needed_length) =
        LAST_ERROR.with_borrow(|(error_code, msg)| (*error_code, msg.as_bytes().len() + 1));

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
            // again: LAST_ERROR is guaranteed to contain a valid error.
            let error_msg = LAST_ERROR.with_borrow_mut(|e| {
                let mut value = (ErrorCode::NoError, String::new());
                std::mem::swap(e, &mut value);
                value.1
            });

            // convert the buffer into a suitable type
            // SAFETY: it is the responsibility of the caller to ensure that the pointer is valid for
            //  writes of up to len bytes.
            let slice = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, *len) };
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
/// ## Return values
///
/// * 0: the call worked.
/// * 1: an error happened - use [get_last_error]
#[no_mangle]
pub extern "C" fn set_max_threads(max_threads: usize) -> c_int {
    let max_threads = match NonZero::new(max_threads) {
        Some(max_threads) => max_threads,
        None => {
            LAST_ERROR.with_borrow_mut(|err| {
                *err = (
                    ErrorCode::SetMaxThreads,
                    "0 is not a valid thread count".to_owned(),
                )
            });
            return 1;
        }
    };

    match sts_lib::set_max_threads(max_threads) {
        Ok(()) => 0,
        Err(e) => {
            LAST_ERROR.with_borrow_mut(|err| *err = (ErrorCode::SetMaxThreads, e.to_string()));
            1
        }
    }
}

/// The error codes that are returned by some fallible functions.
/// A human-readable error message can be retrieved with [get_last_error].
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
    /// The test whose result was tried to be retrieved from the test runner was not run.
    TestWasNotRun = 10,
}

/// Returns the minimum input length, in bits, for the specified test.
///
/// ## Return values
///
/// * >0: the call worked. Returned is minimum input length
/// * 0: an error happened - use [get_last_error]
#[no_mangle]
pub extern "C" fn get_min_length_for_test(test: RawTest) -> usize {
    let raw_test = test;
    let test = crate::test_runner::test::Test::try_from(raw_test);

    let test = match test {
        Ok(test) => test,
        Err(()) => {
            set_last_invalid_test(raw_test);
            return 0;
        }
    };

    // No test has a minimum length of 0 defined - NonZero<usize> guarantees that.
    sts_lib::get_min_length_for_test(test.into()).get()
}

/// Sets the last error from the specified [sts_lib::Error].
fn set_last_from_error(error: sts_lib::Error) {
    let (code, msg) = match error {
        e @ sts_lib::Error::Overflow(_) => (ErrorCode::Overflow, e.to_string()),
        e @ sts_lib::Error::NaN => (ErrorCode::NaN, e.to_string()),
        e @ sts_lib::Error::Infinite => (ErrorCode::Infinite, e.to_string()),
        e @ sts_lib::Error::GammaFunctionFailed(_) => {
            (ErrorCode::GammaFunctionFailed, e.to_string())
        }
        e @ sts_lib::Error::InvalidParameter(_) => (ErrorCode::InvalidParameter, e.to_string()),
    };

    LAST_ERROR.with_borrow_mut(|e| *e = (code, msg));
}

/// Sets the last error from the specified [RunnerError].
fn set_last_from_runner_error(error: RunnerError) {
    LAST_ERROR.with_borrow_mut(|e| *e = (ErrorCode::DuplicateTest, error.to_string()));
}

/// Sets the last error from the specified errors that happened when running test
/// with the test runner infrastructure.
fn set_last_from_test_failed(error: Box<[(sts_lib::Test, sts_lib::Error)]>) {
    LAST_ERROR.with_borrow_mut(|e| {
        *e = (
            ErrorCode::TestFailed,
            format!("Test runner: one or multiple tests, failed, {error:?}"),
        )
    })
}

/// Sets the last error to be about an invalid test (the given value was passed from FFI).
fn set_last_invalid_test(test_no: c_int) {
    let msg = format!("The numerical value {test_no} is not a valid test!");
    LAST_ERROR.with_borrow_mut(|e| *e = (ErrorCode::InvalidTest, msg));
}

/// Sets the last error to be about the fact that the specified test was not run.
fn set_last_test_was_not_run(test: sts_lib::Test) {
    let msg = format!("The test {test} was not run!");
    LAST_ERROR.with_borrow_mut(|e| *e = (ErrorCode::TestWasNotRun, msg));
}
