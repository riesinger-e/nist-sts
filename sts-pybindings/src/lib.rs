#![doc = include_str!("../README.md")]

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

pub mod bitvec;
pub mod test_args;
pub mod test_runner;
pub mod tests;

create_exception!(
    nist_sts,
    TestError,
    PyException,
    "A statistical test failed."
);
create_exception!(
    nist_sts,
    RunnerError,
    PyException,
    "A problem with the runner itself, and not with the tests it runs."
);
create_exception!(
    nist_sts,
    StsError,
    PyException,
    "The library was used very wrong."
);

#[pymodule]
pub mod nist_sts {
    use super::{RunnerError, StsError, TestError};
    use pyo3::prelude::*;
    use pyo3::PyResult;
    use std::num::NonZero;

    // re-exports of the BitVec and TestRunner
    #[pymodule_export]
    pub use crate::bitvec::BitVec;
    #[pymodule_export]
    pub use crate::test_runner::run_tests;

    /// Initialization function, takes care that the custom error types are in the module.
    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("TestError", m.py().get_type_bound::<TestError>())?;
        m.add("RunnerError", m.py().get_type_bound::<RunnerError>())?;
        m.add("LibError", m.py().get_type_bound::<StsError>())?;
        Ok(())
    }

    /// The common test result type, returned by all tests.
    #[pyclass(frozen)]
    #[derive(Copy, Clone)]
    pub struct TestResult(pub(crate) sts_lib::TestResult);

    #[pymethods]
    impl TestResult {
        /// The default for threshold, to be used in TestResult.passed().
        #[classattr]
        pub const DEFAULT_THRESHOLD: f64 = 0.01;

        /// Returns the stored P-Value of the result.
        pub fn p_value(&self) -> f64 {
            self.0.p_value()
        }

        /// Determines if the stored P-Value passed the test by comparing it to the given threshold.
        /// If the P-Value is greater than the threshold, the test passed.
        pub fn passed(&self, threshold: f64) -> bool {
            self.p_value() >= threshold
        }

        /// Returns the comment stored in the test result, or None if there is no comment.
        pub fn comment(&self) -> Option<&str> {
            self.0.comment()
        }

        // String representation
        pub fn __repr__(&self) -> String {
            if let Some(comment) = self.0.comment() {
                format!(
                    "TestResult(p_value = {}, comment = \"{}\")",
                    self.0.p_value(),
                    comment
                )
            } else {
                format!("TestResult(p_value = {})", self.0.p_value())
            }
        }

        // String representation
        pub fn __str__(&self) -> String {
            self.__repr__()
        }
    }

    /// Sets the maximum of threads to be used by the tests. These method can only be called ONCE and
    /// only BEFORE a test is started. If not used, a sane default will be chosen.
    ///
    /// If called multiple times or after the first test, an error will be raised.
    #[pyfunction]
    pub fn set_max_threads(max_threads: usize) -> PyResult<()> {
        let max_threads =
            NonZero::new(max_threads).ok_or(StsError::new_err("0 is not a valid thread count"))?;
        sts_lib::set_max_threads(max_threads)
            .map_err(|e| StsError::new_err(format!("Function was already used: {e}")))
    }

    #[pyfunction]
    pub fn get_min_length_for_test(test: Test) -> usize {
        sts_lib::get_min_length_for_test(test.into()).get()
    }

    /// List of all tests, used for the TestRunner to know which threads to run.
    #[pyclass(eq, eq_int)]
    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
    pub enum Test {
        /// See [tests::frequency_test].
        Frequency,
        /// See [tests::frequency_block_test].
        FrequencyWithinABlock,
        /// See [tests::runs_test].
        Runs,
        /// See [tests::longest_run_of_ones_test].
        LongestRunOfOnes,
        /// See [tests::binary_matrix_rank_test].
        BinaryMatrixRank,
        /// See [tests::spectral_dft_test].
        SpectralDft,
        /// See [tests::template_matching::non_overlapping_test].
        NonOverlappingTemplateMatching,
        /// See [tests::template_matching::overlapping_test].
        OverlappingTemplateMatching,
        /// See [tests::maurers_universal_statistical_test]
        MaurersUniversalStatistical,
        /// See [tests::linear_complexity_test]
        LinearComplexity,
        /// See [tests::serial_test]
        Serial,
        /// See [tests::approximate_entropy_test]
        ApproximateEntropy,
        /// See [tests::cumulative_sums_test]
        CumulativeSums,
        /// See [tests::random_excursions_test]
        RandomExcursions,
        /// See [tests::random_excursions_variant_test]
        RandomExcursionsVariant,
    }

    impl From<sts_lib::Test> for Test {
        fn from(value: sts_lib::Test) -> Self {
            match value {
                sts_lib::Test::Frequency => Test::Frequency,
                sts_lib::Test::FrequencyWithinABlock => Test::FrequencyWithinABlock,
                sts_lib::Test::Runs => Test::Runs,
                sts_lib::Test::LongestRunOfOnes => Test::LongestRunOfOnes,
                sts_lib::Test::BinaryMatrixRank => Test::BinaryMatrixRank,
                sts_lib::Test::SpectralDft => Test::SpectralDft,
                sts_lib::Test::NonOverlappingTemplateMatching => {
                    Test::NonOverlappingTemplateMatching
                }
                sts_lib::Test::OverlappingTemplateMatching => Test::OverlappingTemplateMatching,
                sts_lib::Test::MaurersUniversalStatistical => Test::MaurersUniversalStatistical,
                sts_lib::Test::LinearComplexity => Test::LinearComplexity,
                sts_lib::Test::Serial => Test::Serial,
                sts_lib::Test::ApproximateEntropy => Test::ApproximateEntropy,
                sts_lib::Test::CumulativeSums => Test::CumulativeSums,
                sts_lib::Test::RandomExcursions => Test::RandomExcursions,
                sts_lib::Test::RandomExcursionsVariant => Test::RandomExcursionsVariant,
            }
        }
    }

    impl From<Test> for sts_lib::Test {
        fn from(value: Test) -> Self {
            match value {
                Test::Frequency => sts_lib::Test::Frequency,
                Test::FrequencyWithinABlock => sts_lib::Test::FrequencyWithinABlock,
                Test::Runs => sts_lib::Test::Runs,
                Test::LongestRunOfOnes => sts_lib::Test::LongestRunOfOnes,
                Test::BinaryMatrixRank => sts_lib::Test::BinaryMatrixRank,
                Test::SpectralDft => sts_lib::Test::SpectralDft,
                Test::NonOverlappingTemplateMatching => {
                    sts_lib::Test::NonOverlappingTemplateMatching
                }
                Test::OverlappingTemplateMatching => sts_lib::Test::OverlappingTemplateMatching,
                Test::MaurersUniversalStatistical => sts_lib::Test::MaurersUniversalStatistical,
                Test::LinearComplexity => sts_lib::Test::LinearComplexity,
                Test::Serial => sts_lib::Test::Serial,
                Test::ApproximateEntropy => sts_lib::Test::ApproximateEntropy,
                Test::CumulativeSums => sts_lib::Test::CumulativeSums,
                Test::RandomExcursions => sts_lib::Test::RandomExcursions,
                Test::RandomExcursionsVariant => sts_lib::Test::RandomExcursionsVariant,
            }
        }
    }

    #[pymethods]
    impl Test {
        // String representations
        pub fn __repr__(&self) -> String {
            format!("Test.{}", sts_lib::Test::from(*self))
        }

        pub fn __str__(&self) -> String {
            self.__repr__()
        }
    }

    #[pymodule]
    pub mod tests {
        /// The functions for calling the tests directly.

        // Test 1
        #[pymodule_export]
        pub use crate::tests::frequency_test;
        // Test 2
        #[pymodule_export]
        pub use crate::tests::frequency_block_test;
        // Test 3
        #[pymodule_export]
        pub use crate::tests::runs_test;
        // Test 4
        #[pymodule_export]
        pub use crate::tests::longest_runs_of_ones_test;
        // Test 5
        #[pymodule_export]
        pub use crate::tests::binary_matrix_rank_test;
        // Test 6
        #[pymodule_export]
        pub use crate::tests::spectral_dft_test;
        // Test 7
        #[pymodule_export]
        pub use crate::tests::non_overlapping_template_matching_test;
        // Test 8
        #[pymodule_export]
        pub use crate::tests::overlapping_template_matching_test;
        // Test 9
        #[pymodule_export]
        pub use crate::tests::maurers_universal_statistical_test;
        // Test 10
        #[pymodule_export]
        pub use crate::tests::linear_complexity_test;
        // Test 11
        #[pymodule_export]
        pub use crate::tests::serial_test;
        // Test 12
        #[pymodule_export]
        pub use crate::tests::approximate_entropy_test;
        // Test 13
        #[pymodule_export]
        pub use crate::tests::cumulative_sums_test;
        // Test 14
        #[pymodule_export]
        pub use crate::tests::random_excursions_test;
        // Test 15
        #[pymodule_export]
        pub use crate::tests::random_excursions_variant_test;
    }

    #[pymodule]
    pub mod test_args {
        /// The test argument types, where necessary.

        #[pymodule_export]
        pub use crate::test_args::FrequencyBlockTestArg;

        #[pymodule_export]
        pub use crate::test_args::NonOverlappingTemplateTestArgs;

        #[pymodule_export]
        pub use crate::test_args::OverlappingTemplateTestArgs;

        #[pymodule_export]
        pub use crate::test_args::LinearComplexityTestArg;

        #[pymodule_export]
        pub use crate::test_args::SerialTestArg;

        #[pymodule_export]
        pub use crate::test_args::ApproximateEntropyTestArg;
    }
}
