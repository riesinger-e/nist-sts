//! The necessary test runner types to run the tests. These are necessary because some
//! tests have a precondition that another test has to pass - the runner allows for easy checking.

use crate::bitvec::BitVec;
use crate::{tests, Error, Test, TestResult, TestArgs};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Mutex;
use std::thread;
use strum::IntoEnumIterator;

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

    /// Runs all available tests automatically, with necessary arguments automatically chosen.
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    ///
    /// Depending on the runner, this may execute tests in parallel.
    fn run_all_tests_automatic(&self, data: &BitVec) -> impl Iterator<Item = (Test, Error)> {
        self.run_tests_automatic(Test::iter(), data)
    }

    /// Runs all given tests automatically, with necessary arguments automatically chosen.
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    ///
    /// Depending on the runner, this may execute tests in parallel.
    fn run_tests_automatic(
        &self,
        tests: impl Iterator<Item = Test>,
        data: &BitVec,
    ) -> impl Iterator<Item = (Test, Error)> {
        self.run_tests(tests, data, TestArgs::default())
    }

    /// Runs all available tests with the used arguments taken from the passed [args](TestArgs).
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    ///
    /// Depending on the runner, this may execute tests in parallel.
    fn run_all_tests(&self, data: &BitVec, args: TestArgs) -> impl Iterator<Item = (Test, Error)> {
        self.run_tests(Test::iter(), data, args)
    }

    /// Runs all given tests with the used arguments taken from the passed [args](TestArgs).
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    ///
    /// Depending on the runner, this may execute tests in parallel.
    fn run_tests(&self, tests: impl Iterator<Item = Test>, data: &BitVec, args: TestArgs) -> impl Iterator<Item = (Test, Error)>;
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
        F: FnOnce(&mut RunnerState) -> T,
    {
        let mut state = self.state.borrow_mut();
        call(state.deref_mut())
    }
}

impl TestRunner for SingleThreadedTestRunner {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            state: RefCell::new(RunnerState::default()),
        }
    }

    fn run_tests(&self, tests: impl Iterator<Item=Test>, data: &BitVec, args: TestArgs) -> impl Iterator<Item=(Test, Error)> {
        tests.filter_map(move |test| run_test(self, test, data, &args))
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
        F: FnOnce(&mut RunnerState) -> T,
    {
        let mut state = self.state.lock().unwrap();
        call(state.deref_mut())
    }
}

impl TestRunner for MultiThreadedTestRunner {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            state: Mutex::new(RunnerState::default()),
        }
    }

    fn run_tests(&self, tests: impl Iterator<Item=Test>, data: &BitVec, args: TestArgs) -> impl Iterator<Item=(Test, Error)> {
        let args = &args;
        thread::scope(|scope| {
            // Spawn all tests as a thread
            let handles = tests
                .map(|test| scope.spawn(move || run_test(self, test, data, args)))
                .collect::<Vec<_>>();

            // wait for all threads to finish and collect the result once again
            // this has to be done because otherwise, the iterator is bound to the scope of the thread
            // and cannot escape it
            handles.into_iter()
                // propagate panics happening in one thread to this main thread - the
                // single-threaded implementation does so too
                .filter_map(move |handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        })
            // iterate over the results
            .into_iter()
    }
}

/// Internal runner state
#[derive(Default)]
pub(crate) struct RunnerState {
    // the stored test results
    pub(crate) stored_results: HashMap<Test, TestResult>,
}

/// internally used function to run the test and store the result, used by both runners
fn run_test<R: TestRunner>(runner: &R, test: Test, data: &BitVec, args: &TestArgs) -> Option<(Test, Error)> {
    let result = match test {
        Test::Frequency => tests::frequency::frequency_test(data),
        Test::FrequencyWithinABlock => tests::frequency_block::frequency_block_test(
            data,
            args.frequency_block_test_arg,
        ),
        Test::Runs => tests::runs::runs_test(data),
        Test::LongestRunOfOnes => tests::longest_run_of_ones::longest_run_of_ones_test(data),
    };

    match result {
        Ok(result) => {
            runner.store_result(test, result);
            None
        }
        Err(e) => Some((test, e)),
    }
}
