//! The test runner, for running multiple tests in one call.

use crate::bitvec::BitVec;
use crate::test_result::TestResult;
use crate::test_runner::test::{RawTest, Test};
use crate::test_runner::test_args::RunnerTestArgs;
use crate::{set_last_from_runner_error, set_last_invalid_test};
use std::ffi::c_int;
use std::slice;
use sts_lib::test_runner;

pub mod test;
pub mod test_args;

/// This test runner can be used to run several / all tests on a sequence in one call.
pub struct TestRunner(test_runner::TestRunner);

/// Creates a new test runner. This test runner can be used to run multiple tests on 1 sequence in
/// 1 function call.
///
/// The result pointer must be freed with [test_runner_destroy].
#[no_mangle]
pub extern "C" fn test_runner_new() -> &'static mut TestRunner {
    let runner = test_runner::TestRunner::new();
    let runner = Box::new(TestRunner(runner));
    Box::leak(runner)
}

/// Destroys the given test runner.
///
/// ## Safety
///
/// * `runner` must have been created by [test_runner_new()]
/// * `runner` must be valid for reads and writes and non-null.
/// * `runner` may not be mutated for the duration of this call.
/// * `runner` will be an invalid pointer after this call, trying to access its memory will lead to
///   undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn test_runner_destroy(runner: &'static mut TestRunner) {
    // SAFETY: caller has to ensure that runner is a valid reference from a box
    let _ = unsafe { Box::from_raw(runner) };
}

/// Returns the result of the given test, if it was run. Since some tests return multiple results,
/// the returned pointer is an array, the count of elements will be stored into `length`.
///
/// After this call, the result is no longer stored inside the runner.
///
/// The resulting list of test results must be destroyed with
/// [test_result_list_destroy](crate::test_result::test_result_list_destroy).
///
/// ## Safety
///
/// * `runner` must have been created by [test_runner_new()]
/// * `runner` must be valid for reads and writes and non-null.
/// * `runner` may not be mutated for the duration of this call.
/// * `length` must be a non-null pointer valid for writes.
/// * `length` may not be mutated for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn test_runner_get_result(
    runner: &mut TestRunner,
    test: RawTest,
    length: &mut usize,
) -> *mut *mut TestResult {
    // parse the test
    let Ok(test) = Test::try_from(test) else {
        set_last_invalid_test(test);
        return std::ptr::null_mut();
    };

    let test = test.into();

    match runner.0.get_test_result(test) {
        None => {
            crate::set_last_test_was_not_run(test);
            std::ptr::null_mut()
        }
        Some(result) => {
            let result: Box<[*mut TestResult]> = Box::into_iter(result.into_boxed_slice())
                .map(|res| Box::into_raw(Box::new(TestResult(res))))
                .collect();
            *length = result.len();
            Box::into_raw(result) as *mut *mut TestResult
        }
    }
}

/// Runs all tests on the given bit sequence with the default test arguments.
///
/// ## Return value
///
/// * If all tests ran successfully, `0` is returned.
/// * If an error occurred while running the tests, `1` is returned, and the error message and code
///   can be found out with [get_last_error_str](crate::get_last_error_str).
///
/// ## Safety
///
/// * `runner` must have been created by [test_runner_new()]
/// * `runner` must be valid for reads and writes and non-null.
/// * `runner` may not be mutated for the duration of this call.
/// * `bitvec` must have been created by either [bitvec_from_str](crate::bitvec::bitvec_from_str),
///   [bitvec_from_str_with_max_length](crate::bitvec::bitvec_from_str_with_max_length),
///   [bitvec_from_bytes](crate::bitvec::bitvec_from_bytes),
///   [bitvec_from_bits](crate::bitvec::bitvec_from_bits) or
///   [bitvec_clone](crate::bitvec::bitvec_clone).
/// * `bitvec` must be a non-null pointer valid for reads.
/// * `bitvec` may not be mutated for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn test_runner_run_all_automatic(
    runner: &mut TestRunner,
    data: &BitVec,
) -> c_int {
    match runner.0.run_all_tests_automatic(&data.0) {
        Ok(()) => 0,
        Err(e) => {
            set_last_from_runner_error(e);
            1
        }
    }
}

/// Runs all chosen tests on the given bit sequence with the default test arguments.
///
/// ## Return value
///
/// * If all tests ran successfully, `0` is returned.
/// * If one of the tests specified was a duplicate of a previous test, `1` is returned.
/// * If one of the tests specified was not a valid test as per the enum [Test], `1` is returned.
/// * If an error occurred while running the tests, `1` is returned.
///
/// In each error case, the error message and code can be found out with
/// [get_last_error_str](crate::get_last_error_str).
///
/// ## Safety
///
/// * `runner` must have been created by [test_runner_new()]
/// * `runner` must be valid for reads and writes and non-null.
/// * `runner` may not be mutated for the duration of this call.
/// * `bitvec` must have been created by either [bitvec_from_str](crate::bitvec::bitvec_from_str),
///   [bitvec_from_str_with_max_length](crate::bitvec::bitvec_from_str_with_max_length),
///   [bitvec_from_bytes](crate::bitvec::bitvec_from_bytes),
///   [bitvec_from_bits](crate::bitvec::bitvec_from_bits) or
///   [bitvec_clone](crate::bitvec::bitvec_clone).
/// * `bitvec` must be a non-null pointer valid for reads.
/// * `bitvec` may not be mutated for the duration of this call.
/// * `tests` must be a valid, non-null pointer readable for up to `tests_len` elements.
/// * `tests` may not be mutated for the duration of this call.
#[no_mangle]
pub unsafe extern "C" fn test_runner_run_automatic(
    runner: &mut TestRunner,
    data: &BitVec,
    tests: *const RawTest,
    tests_len: usize,
) -> c_int {
    // SAFETY: same considerations apply to the call as for this function, caller has to ensure
    // that the requirements are met.
    let tests = unsafe { try_get_tests(tests, tests_len) };

    let tests = match tests {
        Some(tests) => tests,
        // Error message was already set
        None => return 1,
    };

    let result = runner.0.run_tests_automatic(tests.into_iter(), &data.0);
    match result {
        Ok(()) => 0,
        Err(e) => {
            set_last_from_runner_error(e);
            1
        }
    }
}

/// Runs all tests on the given bit sequence with the given test arguments.
///
/// ## Return value
///
/// * If all tests ran successfully, `0` is returned.
/// * If an error occurred while running the tests, `1` is returned, and the error message and code
///   can be found out with [get_last_error_str](crate::get_last_error_str).
///
/// ## Safety
///
/// * `runner` must have been created by [test_runner_new()]
/// * `runner` must be valid for reads and writes and non-null.
/// * `runner` may not be mutated for the duration of this call.
/// * `bitvec` must have been created by either [bitvec_from_str](crate::bitvec::bitvec_from_str),
///   [bitvec_from_str_with_max_length](crate::bitvec::bitvec_from_str_with_max_length),
///   [bitvec_from_bytes](crate::bitvec::bitvec_from_bytes),
///   [bitvec_from_bits](crate::bitvec::bitvec_from_bits) or
///   [bitvec_clone](crate::bitvec::bitvec_clone).
/// * `bitvec` must be a non-null pointer valid for reads.
/// * `bitvec` may not be mutated for the duration of this call.
/// * `test_args` must have been created by [runner_test_args_new](test_args::runner_test_args_new).
/// * `test_args` must be a non-null pointer valid for reads.
#[no_mangle]
pub unsafe extern "C" fn test_runner_run_all_tests(
    runner: &mut TestRunner,
    data: &BitVec,
    test_args: &RunnerTestArgs,
) -> c_int {
    let args = test_args.0;

    match runner.0.run_all_tests(&data.0, args) {
        Ok(()) => 0,
        Err(e) => {
            set_last_from_runner_error(e);
            1
        }
    }
}

/// Runs all chosen tests on the given bit sequence with the given test arguments.
///
/// ## Return value
///
/// * If all tests ran successfully, `0` is returned.
/// * If one of the tests specified was a duplicate of a previous test, `1` is returned.
/// * If one of the tests specified was not a valid test as per the enum [Test], `1` is returned.
/// * If an error occurred while running the tests, `1` is returned.
///
/// In each error case, the error message and code can be found out with
/// [get_last_error_str](crate::get_last_error_str).
///
/// ## Safety
///
/// * `runner` must have been created by [test_runner_new()]
/// * `runner` must be valid for reads and writes and non-null.
/// * `runner` may not be mutated for the duration of this call.
/// * `bitvec` must have been created by either [bitvec_from_str](crate::bitvec::bitvec_from_str),
///   [bitvec_from_str_with_max_length](crate::bitvec::bitvec_from_str_with_max_length),
///   [bitvec_from_bytes](crate::bitvec::bitvec_from_bytes),
///   [bitvec_from_bits](crate::bitvec::bitvec_from_bits) or
///   [bitvec_clone](crate::bitvec::bitvec_clone).
/// * `bitvec` must be a non-null pointer valid for reads.
/// * `bitvec` may not be mutated for the duration of this call.
/// * `tests` must be a valid, non-null pointer readable for up to `tests_len` elements.
/// * `tests` may not be mutated for the duration of this call.
/// * `test_args` must have been created by [runner_test_args_new](test_args::runner_test_args_new).
/// * `test_args` must be a non-null pointer valid for reads.
#[no_mangle]
pub unsafe extern "C" fn test_runner_run_tests(
    runner: &mut TestRunner,
    data: &BitVec,
    tests: *const RawTest,
    tests_len: usize,
    test_args: &RunnerTestArgs,
) -> c_int {
    // SAFETY: same considerations apply to the call as for this function, caller has to ensure
    // that the requirements are met.
    let tests = unsafe { try_get_tests(tests, tests_len) };

    let tests = match tests {
        Some(tests) => tests,
        // Error message was already set
        None => return 1,
    };

    let args = test_args.0;

    let result = runner.0.run_tests(tests.into_iter(), &data.0, args);
    match result {
        Ok(()) => 0,
        Err(e) => {
            set_last_from_runner_error(e);
            1
        }
    }
}


/// Try to convert the pointer with offset to a list of tests.
/// Returns None and sets an error if any of the tests was invalid.
///
/// ## Safety
///
/// * `tests` must be a valid, non-null pointer readable for up to `tests_len` elements.
/// * `tests` may not be mutated for the duration of this call.
unsafe fn try_get_tests(tests: *const RawTest, tests_len: usize) -> Option<Vec<sts_lib::Test>> {
    // SAFETY: caller has to ensure that tests is valid for read of tests_len elements.
    let tests = unsafe { slice::from_raw_parts(tests, tests_len) };

    tests
        .iter()
        .map(|&raw_test| match Test::try_from(raw_test) {
            Ok(test) => Some(sts_lib::Test::from(test)),
            Err(()) => {
                set_last_invalid_test(raw_test);
                None
            }
        })
        .collect()
}
