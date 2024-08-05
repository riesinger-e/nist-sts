//! Everything about test results.

use std::ffi::{c_char, c_int};
use std::slice;
use sts_lib::TestResult as InternalTestResult;

/// The default threshold for determining if a test passes its criteria.
pub const DEFAULT_THRESHOLD: f64 = 0.01;

/// The result of a statistical test.
#[derive(Copy, Clone)]
pub struct TestResult(pub(crate) InternalTestResult);

/// Destroys the given test result. If you want to destroy a whole list, use [test_result_list_destroy].
/// You cannot destroy only a part of a list with this function.
///
/// ## Safety
///
/// * `ptr` must have been created by one of the tests and must have been returned as a single pointer.
/// * `ptr` must be a valid, non-null element.
/// * `ptr` must not be mutated for the duration of this call.
/// * `ptr` will be invalid after this call, access will lead to undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn test_result_destroy(ptr: Option<Box<TestResult>>) {
    _ = ptr;
}

/// Destroys the given list of test results. If you want to destroy only a single test result,
/// use [test_result_destroy].
///
/// ## Safety
///
/// * `ptr` must have been created by one of the tests or by the test runner, and must have been
///   returned by the creating function as a list.
/// * `ptr` must be valid allocation with `count` elements.
/// * `ptr` must not be mutated for the duration of this call.
/// * `ptr` will be invalid after this call, access will lead to undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn test_result_list_destroy(ptr: *mut Box<TestResult>, count: usize) {
    // SAFETY: caller has to ensure that the pointer is valid with count elements
    _ = unsafe {
        Box::from_raw(slice::from_raw_parts_mut(ptr, count))
    };
}

/// Returns the p_value of the test result.
///
/// ## Safety
///
/// * `result` must have been created by one of the tests.
/// * `result` must be a valid pointer.
/// * `result` may not be mutated for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn test_result_get_p_value(result: &TestResult) -> f64 {
    result.0.p_value()
}

/// Checks if the contained p_value passed the given threshold (i.e. if test passed).
///
/// ## Safety
///
/// * `result` must have been created by one of the tests.
/// * `result` must be a valid pointer.
/// * `result` may not be mutated for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn test_result_passed(result: &TestResult, threshold: f64) -> bool {
    result.0.passed(threshold)
}

/// Extracts the (maybe existing) comment contained in the test result.
/// This function works in 2 steps:
/// 1. the caller calls the function with `ptr` set to `NULL`. The necessary length is written to
///    `len`.
/// 2. the caller calls the function with `ptr` set to a valid buffer, and `len` set to the length of
///    the buffer. If the length is enough to store the error message, it is written to the buffer.
///    The error message is written with a nul-terminating byte.
///
/// # Return values
///
/// - 0: everything's OK.
/// - 1: there is no comment to store.
/// - 2: the passed string buffer is too small.
///
/// ## Safety
///
/// * `result` must have been created by one of the tests.
/// * `result` must be a valid pointer.
/// * `result` may not be mutated for the duration of this call.
/// * `len` must not be `NULL`.
/// * `ptr` must be valid for writes of up to `len` bytes.
/// * `ptr` may not be mutated for the duration of this call.
/// * All responsibility for `ptr` and `len`, especially for its de-allocation, remains with the caller.
#[no_mangle]
pub unsafe extern "C" fn test_result_get_comment(result: &TestResult, ptr: *mut c_char, len: &mut usize) -> c_int {
    // check if there is an error
    if result.0.comment().is_none() {
        return 1;
    }

    let comment = result.0.comment().unwrap();

    // LAST_ERROR is guaranteed to be Some, we just checked. + 1 for the nul byte
    let needed_length = comment.as_bytes().len() + 1;

    if ptr.is_null() {
        // caller only asks for the length

        *len = needed_length;
        0
    } else {
        // caller wants the comment

        // check length
        if *len < needed_length {
            2
        } else {
            // length is OK, write the String
            
            // SAFETY: it is the responsibility of the caller to ensure that the pointer is valid for
            //  writes of up to len bytes.
            let slice = unsafe { slice::from_raw_parts_mut(ptr as *mut u8, *len) };
            // set last NUL byte
            slice[*len - 1] = 0;
            // set message
            comment
                .as_bytes()
                .iter()
                .zip(slice)
                .for_each(|(input, output)| *output = *input);

            0
        }
    }
}