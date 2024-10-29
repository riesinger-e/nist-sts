//! Test runner to run several tests in a batch.

use crate::bitvec::BitVec;
use crate::{tests, Error, Test, TestArgs, TestResult};
use std::collections::HashSet;
use strum::IntoEnumIterator;
use tests::template_matching::non_overlapping;
use tests::template_matching::overlapping;
use tests::*;

/// Error type when using the test runner: In the iterator with the tests to run, one test is contained more than 1 time.
#[derive(Debug, Error)]
#[error("Test {0} is a duplicate!")]
pub struct RunnerError(pub Test);

/// Runs all available tests automatically, with necessary arguments automatically chosen.
///
/// Returns all test results.
pub fn run_all_tests_automatic(
    data: impl AsRef<BitVec>,
) -> Result<impl Iterator<Item = (Test, Result<Vec<TestResult>, Error>)>, RunnerError> {
    run_tests_automatic(data, Test::iter())
}

/// Runs all given tests automatically, with necessary arguments automatically chosen.
///
/// Only unique tests may be passed.
///
/// Returns all test results.
pub fn run_tests_automatic(
    data: impl AsRef<BitVec>,
    tests: impl Iterator<Item = Test>,
) -> Result<impl Iterator<Item = (Test, Result<Vec<TestResult>, Error>)>, RunnerError> {
    run_tests(data, tests, TestArgs::default())
}

/// Runs all available tests with the used arguments taken from the passed [args](TestArgs).
///
/// Returns all test results.
pub fn run_all_tests(
    data: impl AsRef<BitVec>,
    args: TestArgs,
) -> Result<impl Iterator<Item = (Test, Result<Vec<TestResult>, Error>)>, RunnerError> {
    run_tests(data, Test::iter(), args)
}

/// Runs all given tests with the used arguments taken from the passed [args](TestArgs).
///
/// Only unique tests may be passed.
///
/// Returns all test results.
pub fn run_tests(
    data: impl AsRef<BitVec>,
    mut tests: impl Iterator<Item = Test>,
    args: TestArgs,
) -> Result<impl Iterator<Item = (Test, Result<Vec<TestResult>, Error>)>, RunnerError> {
    // check for duplicate tests.
    let mut unique_tests = HashSet::with_capacity(tests.size_hint().0);

    let duplicate = tests.find(|&test| !unique_tests.insert(test));
    if let Some(test) = duplicate {
        // duplicate test
        Err(RunnerError(test))
    } else {
        // unique_tests contains all tests
        let output = unique_tests
            .into_iter()
            .map(move |test| run_test(test, data.as_ref(), args));

        Ok(output)
    }
}

/// internally used function to run the test and store the result.
fn run_test(test: Test, data: &BitVec, args: TestArgs) -> (Test, Result<Vec<TestResult>, Error>) {
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
            return (
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
            maurers_universal_statistical::maurers_universal_statistical_test(data)
        }
        Test::LinearComplexity => {
            linear_complexity::linear_complexity_test(data, args.linear_complexity)
        }
        Test::Serial => return (test, serial::serial_test(data, args.serial).map(From::from)),
        Test::ApproximateEntropy => {
            approximate_entropy::approximate_entropy_test(data, args.approximate_entropy)
        }
        Test::CumulativeSums => {
            return (
                test,
                cumulative_sums::cumulative_sums_test(data).map(From::from),
            )
        }
        Test::RandomExcursions => {
            return (
                test,
                random_excursions::random_excursions_test(data).map(From::from),
            )
        }
        Test::RandomExcursionsVariant => {
            return (
                test,
                random_excursions_variant::random_excursions_variant_test(data).map(From::from),
            )
        }
    };

    (test, result.map(|res| vec![res]))
}
