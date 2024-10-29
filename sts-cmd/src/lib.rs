//! The command line arguments for this program.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use sts_lib::Test;

pub mod cmd_args;
pub mod csv;
pub mod toml_config;
pub mod valid_arg;

/// The tests that can be specified. Used both for command line arguments and TOML.
#[derive(Copy, Clone, Debug, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArgTest {
    /// Frequency Test
    Frequency,
    /// Frequency Test within a Block
    FrequencyWithinABlock,
    /// Runs Test
    Runs,
    /// Test for the Longest Run of Ones in a Block
    LongestRunOfOnes,
    /// Binary Matrix Rank Test
    BinaryMatrixRank,
    /// Discrete Fourier Transform (Spectral) Test
    SpectralDft,
    /// Non-overlapping Template Matching Test
    NonOverlappingTemplateMatching,
    /// Overlapping Template Matching Test
    OverlappingTemplateMatching,
    /// Maurers Universal Statistical Test
    MaurersUniversalStatistical,
    /// Linear Complexity Test
    LinearComplexity,
    /// Serial Test
    Serial,
    /// Approximate Entropy Test
    ApproximateEntropy,
    /// Cumulative Sums Test
    CumulativeSums,
    /// Random Excursions Test
    RandomExcursions,
    /// Random Excursions Variant Test
    RandomExcursionsVariant,
}

// this implementation is only there to break if a test is added into sts_lib.
impl From<Test> for ArgTest {
    fn from(value: Test) -> Self {
        match value {
            Test::Frequency => ArgTest::Frequency,
            Test::FrequencyWithinABlock => ArgTest::FrequencyWithinABlock,
            Test::Runs => ArgTest::Runs,
            Test::LongestRunOfOnes => ArgTest::LongestRunOfOnes,
            Test::BinaryMatrixRank => ArgTest::BinaryMatrixRank,
            Test::SpectralDft => ArgTest::SpectralDft,
            Test::NonOverlappingTemplateMatching => ArgTest::NonOverlappingTemplateMatching,
            Test::OverlappingTemplateMatching => ArgTest::OverlappingTemplateMatching,
            Test::MaurersUniversalStatistical => ArgTest::MaurersUniversalStatistical,
            Test::LinearComplexity => ArgTest::LinearComplexity,
            Test::Serial => ArgTest::Serial,
            Test::ApproximateEntropy => ArgTest::ApproximateEntropy,
            Test::CumulativeSums => ArgTest::CumulativeSums,
            Test::RandomExcursions => ArgTest::RandomExcursions,
            Test::RandomExcursionsVariant => ArgTest::RandomExcursionsVariant,
        }
    }
}

impl From<ArgTest> for Test {
    fn from(value: ArgTest) -> Self {
        match value {
            ArgTest::Frequency => Test::Frequency,
            ArgTest::FrequencyWithinABlock => Test::FrequencyWithinABlock,
            ArgTest::Runs => Test::Runs,
            ArgTest::LongestRunOfOnes => Test::LongestRunOfOnes,
            ArgTest::BinaryMatrixRank => Test::BinaryMatrixRank,
            ArgTest::SpectralDft => Test::SpectralDft,
            ArgTest::NonOverlappingTemplateMatching => Test::NonOverlappingTemplateMatching,
            ArgTest::OverlappingTemplateMatching => Test::OverlappingTemplateMatching,
            ArgTest::MaurersUniversalStatistical => Test::MaurersUniversalStatistical,
            ArgTest::LinearComplexity => Test::LinearComplexity,
            ArgTest::Serial => Test::Serial,
            ArgTest::ApproximateEntropy => Test::ApproximateEntropy,
            ArgTest::CumulativeSums => Test::CumulativeSums,
            ArgTest::RandomExcursions => Test::RandomExcursions,
            ArgTest::RandomExcursionsVariant => Test::RandomExcursionsVariant,
        }
    }
}

/// The input file formats that can be specified. Used both for command line arguments and TOML.
#[derive(Copy, Clone, Debug, PartialEq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputFormat {
    /// Binary input.
    Binary,
    /// Input is an ASCII text file consisting of only '0' or '1'.
    Ascii,
    /// Input is an ASCII text file consisting of any character. Characters other than '0' or '1'
    /// are skipped.
    AsciiLossy,
}
