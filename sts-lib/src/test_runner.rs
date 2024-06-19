//! The necessary test runner types to run the tests. These are necessary because some
//! tests have a precondition that another test has to pass - the runner allows for easy checking.

use crate::{Test, TestResult};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Mutex;

/// Trait for a testrunner, to be used in every test.
// private bound here is used to seal the trait.
#[allow(private_bounds)]
pub trait TestRunner: RunnerBase {
    /// Create a new Testrunner that uses the passed test data.
    fn new() -> Self
    where
        Self: Sized;

    /// Get the test result for the specific tests. Test results are always stored when a test
    /// is run and overwrite earlier results.
    fn get_test_result(&self, test: Test) -> Option<TestResult> {
        self.use_state(|state| state.stored_results.get(&test).copied())
    }

    // TODO: Add run_all_tests (with custom args), run_all_tests_automatic (args chosen automatically)
}

/// Internal trait for using the runner in tests
pub(crate) trait RunnerBase {
    /// Use the internal state, independent of the concrete implementation.
    fn use_state<F, T>(&self, call: F) -> T
    where
        F: FnOnce(&mut RunnerState) -> T;

    /// Store the test result.
    fn store_result(&self, test: Test, result: TestResult) {
        self.use_state(|state| state.stored_results.insert(test, result));
    }
}

/// Single threaded implementation of a test runner.
#[repr(C)]
pub struct SingleThreadedTestRunner {
    state: RefCell<RunnerState>,
}

impl RunnerBase for SingleThreadedTestRunner {
    fn use_state<F, T>(&self, call: F) -> T
    where
        F: FnOnce(&mut RunnerState) -> T
    {
        let mut state = self.state.borrow_mut();
        call(state.deref_mut())
    }
}

impl TestRunner for SingleThreadedTestRunner {
    fn new() -> Self
    where
        Self: Sized
    {
        Self {
            state: RefCell::new(RunnerState::default())
        }
    }
}

/// Implementation of a test runner that can be used multithreaded.
#[repr(C)]
pub struct MultiThreadedTestRunner {
    state: Mutex<RunnerState>,
}

impl RunnerBase for MultiThreadedTestRunner {
    fn use_state<F, T>(&self, call: F) -> T
    where
        F: FnOnce(&mut RunnerState) -> T
    {
        let mut state = self.state.lock().unwrap();
        call(state.deref_mut())
    }
}

impl TestRunner for MultiThreadedTestRunner {
    fn new() -> Self
    where
        Self: Sized
    {
        Self {
            state: Mutex::new(RunnerState::default()),
        }
    }
}

/// Internal runner state
#[derive(Default)]
pub(crate) struct RunnerState {
    // the stored test results
    pub(crate) stored_results: HashMap<Test, TestResult>,
}
