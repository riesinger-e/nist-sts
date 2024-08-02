#![doc = include_str!("../README.md")]

use crate::tests::frequency_block::FrequencyBlockTestArg;
use crate::tests::linear_complexity::LinearComplexityTestArg;
use crate::tests::serial::SerialTestArg;
use crate::tests::template_matching::non_overlapping::NonOverlappingTemplateTestArgs;
use crate::tests::template_matching::overlapping::OverlappingTemplateTestArgs;
use strum::{Display, EnumIter};
use thiserror::Error;
use rayon::ThreadPoolBuilder;
use crate::tests::approximate_entropy::ApproximateEntropyTestArg;

// internal usage only
pub(crate) mod internals;
#[cfg(test)]
mod unit_tests;

// public exports
pub mod bitvec;
pub mod test_runner;
pub mod tests;

// shared data structures

/// How many bits a byte has
const BYTE_SIZE: usize = 8;

/// List of all tests, used e.g. for automatic running.
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, EnumIter, Display)]
pub enum Test {
    Frequency,
    FrequencyWithinABlock,
    Runs,
    LongestRunOfOnes,
    BinaryMatrixRank,
    SpectralDft,
    NonOverlappingTemplateMatching,
    OverlappingTemplateMatching,
    MaurersUniversalStatistical,
    LinearComplexity,
    Serial,
    ApproximateEntropy,
    CumulativeSums,
    RandomExcursions,
    RandomExcursionsVariant,
}

/// All test arguments for use in a [TestRunner](test_runner::TestRunner),
/// prefilled with sane defaults.
///
/// You can construct an instance, leaving all other arguments as the default, like this:
/// ```
/// use std::num::NonZeroUsize;
/// use sts_lib::TestArgs;
/// use sts_lib::tests::frequency_block::FrequencyBlockTestArg;
/// let args = TestArgs {
///     frequency_block: FrequencyBlockTestArg::Bitwise(NonZeroUsize::new(23).unwrap()),
///     ..Default::default()
/// };
/// ```
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
pub struct TestArgs {
    pub frequency_block: FrequencyBlockTestArg,
    pub non_overlapping_template: NonOverlappingTemplateTestArgs<'static>,
    pub overlapping_template: OverlappingTemplateTestArgs,
    pub linear_complexity: LinearComplexityTestArg,
    pub serial: SerialTestArg,
    pub approximate_entropy: ApproximateEntropyTestArg,
}

/// The common test result type, as used by all tests.
#[derive(Copy, Clone)]
pub struct TestResult {
    p_value: f64,
    comment: Option<&'static str>,
}

// private methods
impl TestResult {
    /// A new test result without comment.
    fn new(p_value: f64) -> Self {
        Self {
            p_value,
            comment: None,
        }
    }

    /// A new test result with a comment.
    fn new_with_comment(p_value: f64, comment: &'static str) -> Self {
        Self {
            p_value,
            comment: Some(comment),
        }
    }
}

// public methods
impl TestResult {
    /// The p_value (result of the test)
    pub fn p_value(&self) -> f64 {
        self.p_value
    }

    /// To determine if the test passed, based on the given threshold:
    /// The test passes if the [p_value](Self::p_value) is greater or equal to the given
    /// threshold.
    pub fn passed(&self, threshold: f64) -> bool {
        self.p_value >= threshold
    }

    /// Some tests leave a comment about the outcome.
    pub fn comment(&self) -> Option<&'static str> {
        self.comment
    }
}

/// The error type for all tests
#[derive(Error, Debug)]
pub enum Error {
    /// A numeric overflow happened. The String gives further information on where exactly.
    #[error("Overflow in {0}.")]
    Overflow(String),
    #[error("Result is not a number.")]
    NaN,
    #[error("Result is infinite.")]
    Infinite,
    #[error(transparent)]
    GammaFunctionFailed(#[from] statrs::StatsError),
    #[error("Invalid Parameter: {0}")]
    InvalidParameter(String),
}

/// Sets the maximum of threads to be used by the tests. These method can only be called ONCE and only
/// BEFORE a test is started. If not used, a sane default will be chosen.
///
/// If called multiple times or after the first test, an error will be returned.
///
/// Since this library uses [rayon](https://docs.rs/rayon/latest/rayon/index.html), this function
/// effectively calls
/// [ThreadPoolBuilder::num_threads](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html#method.num_threads).
/// If you use rayon in the calling code, no rayon workload may have been run before calling this
/// function.
pub fn set_max_threads(max_threads: usize) -> Result<(), Box<impl std::error::Error + Send + Sync + 'static>> {
    ThreadPoolBuilder::new()
        .num_threads(max_threads)
        .build_global()
        .map_err(Box::new)
}