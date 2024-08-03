//! Test runner to run several tests in a batch.

use crate::bitvec::BitVec;
use crate::{tests, Error, Test, TestArgs, TestResult};
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator;
use tests::template_matching::non_overlapping;
use tests::template_matching::overlapping;
use tests::*;

/// Error type when using the test runner. There are 2 error cases:
/// 1. In the iterator with the tests to run, one test is contained more than 1 time.
/// 2. One or more of the tests themselves returned an error.
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Test {0} is a duplicate!")]
    Duplicate(Test),
    #[error("Some tests failed: {0:?}")]
    Test(Box<[(Test, Error)]>),
}

/// This test runner can be used to run several / all tests on a sequence in one call.
#[derive(Default)]
pub struct TestRunner {
    stored_results: HashMap<Test, Vec<TestResult>>,
}

// Public methods
impl TestRunner {
    /// Create a new instance of the runner.
    pub fn new() -> Self {
        Self {
            stored_results: HashMap::new(),
        }
    }

    /// Get the test results for the specific tests. Test results are always stored when a test
    /// is run.
    ///
    /// Because some tests yield multiple results, a list of test results is returned.
    ///
    /// A call to this function takes the indicated result out of storage, meaning that a call after
    /// the first for the same test will return `None`.
    pub fn get_test_result(&mut self, test: Test) -> Option<Vec<TestResult>> {
        self.stored_results.remove(&test)
    }

    /// Runs all available tests automatically, with necessary arguments automatically chosen.
    /// All previous state is cleared.
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    pub fn run_all_tests_automatic(&mut self, data: &BitVec) -> Result<(), RunnerError> {
        self.run_tests_automatic(Test::iter(), data)
    }

    /// Runs all given tests automatically, with necessary arguments automatically chosen.
    /// All previous state is cleared.
    ///
    /// Only unique tests may be passed.
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    pub fn run_tests_automatic(
        &mut self,
        tests: impl Iterator<Item = Test>,
        data: &BitVec,
    ) -> Result<(), RunnerError> {
        self.run_tests(tests, data, TestArgs::default())
    }

    /// Runs all available tests with the used arguments taken from the passed [args](TestArgs).
    /// All previous state is cleared.
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    pub fn run_all_tests(&mut self, data: &BitVec, args: TestArgs) -> Result<(), RunnerError> {
        self.run_tests(Test::iter(), data, args)
    }

    /// Runs all given tests with the used arguments taken from the passed [args](TestArgs).
    /// All previous state is cleared.
    ///
    /// Only unique tests may be passed.
    ///
    /// Returns all errors that happened when running the tests, the results can be queried by
    /// using [Self::get_test_result].
    pub fn run_tests(
        &mut self,
        mut tests: impl Iterator<Item = Test>,
        data: &BitVec,
        args: TestArgs,
    ) -> Result<(), RunnerError> {
        // clear all previous state.
        self.stored_results.clear();

        // check for duplicate tests.
        let mut unique_tests = HashSet::with_capacity(tests.size_hint().0);

        let duplicate = tests
            .find(|&test| !unique_tests.insert(test));
        if let Some(test) = duplicate {
            // duplicate test
            Err(RunnerError::Duplicate(test))
        } else {
            // unique_tests contains all tests
            let errors = unique_tests
                .into_iter()
                .filter_map(|test| self.run_test(test, data, &args))
                .collect::<Box<[(Test, Error)]>>();

            if errors.is_empty() {
                Ok(())
            } else {
                Err(RunnerError::Test(errors))
            }
        }
    }
}

// Internal methods.
impl TestRunner {
    /// internally used function to run the test and store the result.
    fn run_test(
        &mut self,
        test: Test,
        data: &BitVec,
        args: &TestArgs,
    ) -> Option<(Test, Error)> {
        let result = match test {
            Test::Frequency => frequency::frequency_test(data),
            Test::FrequencyWithinABlock => {
                frequency_block::frequency_block_test(data, args.frequency_block)
            }
            Test::Runs => runs::runs_test(data),
            Test::LongestRunOfOnes => longest_run_of_ones::longest_run_of_ones_test(data),
            Test::BinaryMatrixRank => binary_matrix_rank::binary_matrix_rank_test(data),
            Test::SpectralDft => spectral_dft::spectral_dft_test(data),
            // early return for the few tests that give multiple results
            Test::NonOverlappingTemplateMatching => {
                return self.handle_multiple_test_results(
                    test,
                    non_overlapping::non_overlapping_template_matching_test(
                        data,
                        args.non_overlapping_template,
                    ),
                );
            }
            Test::OverlappingTemplateMatching => {
                overlapping::overlapping_template_matching_test(data, args.overlapping_template)
            }
            Test::MaurersUniversalStatistical => {
                maurers_universal_statistical::maurers_universal_statistic_test(data)
            }
            Test::LinearComplexity => {
                linear_complexity::linear_complexity_test(data, args.linear_complexity)
            }
            Test::Serial => {
                return self.handle_multiple_test_results(
                    test,
                    serial::serial_test(data, args.serial),
                )
            }
            Test::ApproximateEntropy => {
                approximate_entropy::approximate_entropy_test(data, args.approximate_entropy)
            }
            Test::CumulativeSums => {
                return self.handle_multiple_test_results(
                    test,
                    cumulative_sums::cumulative_sums_test(data),
                )
            }
            Test::RandomExcursions => {
                return self.handle_multiple_test_results(
                    test,
                    random_excursions::random_excursions_test(data),
                )
            }
            Test::RandomExcursionsVariant => {
                return self.handle_multiple_test_results(
                    test,
                    random_excursions_variant::random_excursions_variant_test(data),
                )
            }
        };

        match result {
            Ok(result) => {
                self.stored_results.insert(test, vec![result]);
                None
            }
            Err(e) => Some((test, e)),
        }
    }

    /// Handle results of tests that yield multiple values
    fn handle_multiple_test_results<T: Into<Vec<TestResult>>>(
        &mut self,
        test: Test,
        results: Result<T, Error>,
    ) -> Option<(Test, Error)> {
        match results {
            Ok(results) => {
                self.stored_results.insert(test, results.into());
                None
            }
            Err(e) => Some((test, e)),
        }
    }
}
