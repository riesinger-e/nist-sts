//! The necessary test runner types to run the tests. These are necessary because some
//! tests have a precondition that another test has to pass - the runner allows for easy checking.

use crate::bitvec::BitVec;
use crate::{Error, Test, TestArgs, TestResult, tests};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Mutex;
use std::{thread};
use strum::IntoEnumIterator;
use tests::*;
use crate::tests::template_matching::non_overlapping;

/// Trait for a testrunner, to be used in every test.
// private bound here is used to seal the trait.
#[allow(private_bounds)]
pub trait TestRunner: RunnerBase {
    /// Get the test results for the specific tests. Test results are always stored when a test
    /// is run.
    ///
    /// Because some tests yield multiple results, a list of test results is returned.
    ///
    /// A call to this function takes the indicated result out of storage, meaning that a call after
    /// the first for the same test will return `None`.
    fn get_test_result(&self, test: Test) -> Option<Vec<TestResult>> {
        self.use_state(|state| state.stored_results.remove(&test))
    }

    /// Runs all available tests automatically, with necessary arguments automatically chosen.
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    ///
    /// Depending on the runner, this may execute tests in parallel.
    fn run_all_tests_automatic(&self, data: &BitVec) -> Box<[(Test, Error)]> {
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
    ) -> Box<[(Test, Error)]> {
        <Self as TestRunner>::run_tests(self, tests, data, TestArgs::default())
    }

    /// Runs all available tests with the used arguments taken from the passed [args](TestArgs).
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    ///
    /// Depending on the runner, this may execute tests in parallel.
    fn run_all_tests(&self, data: &BitVec, args: TestArgs) -> Box<[(Test, Error)]> {
        <Self as TestRunner>::run_tests(self, Test::iter(), data, args)
    }

    /// Runs all given tests with the used arguments taken from the passed [args](TestArgs).
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    ///
    /// All previous state is cleared.
    ///
    /// Depending on the runner, this may execute tests in parallel.
    fn run_tests(
        &self,
        tests: impl Iterator<Item = Test>,
        data: &BitVec,
        args: TestArgs,
    ) -> Box<[(Test, Error)]> {
        // clear all previous state, store the arguments
        self.use_state(|state| {
            state.stored_results.clear();
        });

        <Self as RunnerBase>::run_tests(self, tests, data, args)
    }
}

impl<T: RunnerBase> TestRunner for T {}

/// Internal trait for using the runner in tests
pub(crate) trait RunnerBase {
    /// Use the internal state, independent of the concrete implementation.
    fn use_state<F, T>(&self, call: F) -> T
    where
        F: FnOnce(&mut RunnerState) -> T;

    /// Store the test result.
    fn store_result(&self, test: Test, result: TestResult) {
        self.use_state(|state| {
            state
                .stored_results
                .entry(test)
                .and_modify(|result_list| result_list.push(result))
                .or_insert_with(|| vec![result]);
        });
    }

    /// Store multiple test results at once
    fn store_results(&self, test: Test, results: Vec<TestResult>) {
        self.use_state(|state| state.stored_results.insert(test, results));
    }

    /// Run the tests, as passed by [TestRunner::run_tests]. This function is used because
    /// the function in [TestRunner] contains some glue that the internal implementations should
    /// not need to care about.
    fn run_tests(
        &self,
        tests: impl Iterator<Item = Test>,
        data: &BitVec,
        args: TestArgs,
    ) -> Box<[(Test, Error)]>;
}

/// Single threaded implementation of a test runner.
#[repr(C)]
#[derive(Default)]
pub struct SingleThreadedTestRunner {
    state: RefCell<RunnerState>,
}

impl SingleThreadedTestRunner {
    /// Create a new instance of the runner.
    pub fn new() -> Self {
        Self {
            state: RefCell::new(RunnerState::default()),
        }
    }
}

impl RunnerBase for SingleThreadedTestRunner {
    fn use_state<F, T>(&self, call: F) -> T
    where
        F: FnOnce(&mut RunnerState) -> T,
    {
        let mut state = self.state.borrow_mut();
        call(state.deref_mut())
    }

    fn run_tests(
        &self,
        tests: impl Iterator<Item = Test>,
        data: &BitVec,
        args: TestArgs
    ) -> Box<[(Test, Error)]> {
        tests.filter_map(move |test| run_test(self, test, data, &args))
            .collect()
    }
}

/// Implementation of a test runner that can be used multithreaded.
#[repr(C)]
#[derive(Default)]
pub struct MultiThreadedTestRunner {
    state: Mutex<RunnerState>,
}

impl MultiThreadedTestRunner {
    /// Create a new instance of the runner.
    pub fn new() -> Self {
        Self {
            state: Mutex::new(RunnerState::default()),
        }
    }
}

impl RunnerBase for MultiThreadedTestRunner {
    fn use_state<F, T>(&self, call: F) -> T
    where
        F: FnOnce(&mut RunnerState) -> T,
    {
        let mut state = self.state.lock().unwrap();
        call(state.deref_mut())
    }

    fn run_tests(
        &self,
        tests: impl Iterator<Item = Test>,
        data: &BitVec,
        args: TestArgs,
    ) -> Box<[(Test, Error)]> {
        thread::scope(|scope| {
            // Spawn all tests as a thread
            let handles = tests
                .map(|test| scope.spawn(move || run_test(self, test, data, &args)))
                .collect::<Vec<_>>();

            // wait for all threads to finish and collect the result once again
            // this has to be done because otherwise, the iterator is bound to the scope of the thread
            // and cannot escape it
            handles
                .into_iter()
                // propagate panics happening in one thread to this main thread - the
                // single-threaded implementation does so too
                .filter_map(move |handle| handle.join().unwrap())
                .collect()
        })
    }
}

/// Internal runner state
#[derive(Default)]
pub(crate) struct RunnerState {
    // the stored test results
    pub(crate) stored_results: HashMap<Test, Vec<TestResult>>,
}

/// internally used function to run the test and store the result, used by both runners
fn run_test<R: TestRunner>(runner: &R, test: Test, data: &BitVec, args: &TestArgs) -> Option<(Test, Error)> {
    let result = match test {
        Test::Frequency => frequency::frequency_test(data),
        Test::FrequencyWithinABlock => {
            frequency_block::frequency_block_test(data, args.frequency_block_test_arg)
        }
        Test::Runs => runs::runs_test(data),
        Test::LongestRunOfOnes => longest_run_of_ones::longest_run_of_ones_test(data),
        Test::BinaryMatrixRank => binary_matrix_rank::binary_matrix_rank_test(data),
        Test::SpectralDft => spectral_dft::spectral_dft_test(data),
        // early return for the few tests that give multiple results
        Test::NonOverlappingTemplateMatching => {
            return handle_multiple_test_results(
                runner,
                test,
                non_overlapping::non_overlapping_template_matching_test(
                    data, args.non_overlapping_template_test_args,
                ),
            );
        }
    };

    match result {
        Ok(result) => {
            runner.store_result(test, result);
            None
        }
        Err(e) => Some((test, e)),
    }
}

/// Handle results of tests that yield multiple values
fn handle_multiple_test_results<R: TestRunner>(
    runner: &R,
    test: Test,
    results: Result<Vec<TestResult>, Error>,
) -> Option<(Test, Error)> {
    match results {
        Ok(results) => {
            runner.store_results(test, results);
            None
        }
        Err(e) => Some((test, e)),
    }
}
