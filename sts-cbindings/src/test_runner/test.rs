//! The test type itself.

use std::ffi::c_int;

// Type of a raw test, used for the FFI boundary (rust doesn't like it if a value is passed for an
// enum that is not in the enum).
pub type RawTest = c_int;

/// List of all tests, used for automatic running.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub enum Test {
    /// See [frequency_test](crate::tests::frequency_test).
    Frequency = 0,
    /// See [frequency_block_test](crate::tests::frequency_block_test).
    FrequencyWithinABlock = 1,
    /// See [runs_test](crate::tests::runs_test).
    Runs = 2,
    /// See [longest_run_of_ones_test](crate::tests::longest_run_of_ones_test).
    LongestRunOfOnes = 3,
    /// See [binary_matrix_rank_test](crate::tests::binary_matrix_rank_test).
    BinaryMatrixRank = 4,
    /// See [spectral_dft_test](crate::tests::spectral_dft_test).
    SpectralDft = 5,
    /// See [non_overlapping_template_matching_test](crate::tests::non_overlapping_template_matching_test).
    NonOverlappingTemplateMatching = 6,
    /// See [overlapping_template_matching_test](crate::tests::overlapping_template_matching_test).
    OverlappingTemplateMatching = 7,
    /// See [maurers_universal_statistical_test](crate::tests::maurers_universal_statistical_test).
    MaurersUniversalStatistical = 8,
    /// See [linear_complexity_test](crate::tests::linear_complexity_test).
    LinearComplexity = 9,
    /// See [serial_test](crate::tests::serial_test).
    Serial = 10,
    /// See [approximate_entropy_test](crate::tests::approximate_entropy_test).
    ApproximateEntropy = 11,
    /// See [cumulative_sums_test](crate::tests::cumulative_sums_test).
    CumulativeSums = 12,
    /// See [random_excursions_test](crate::tests::random_excursions_test).
    RandomExcursions = 13,
    /// See [random_excursions_variant_test](crate::tests::random_excursions_variant_test).
    RandomExcursionsVariant = 14,
}

// If any of these fails, you also need to adjust the TryFrom-Implementation
impl From<Test> for sts_lib::Test {
    fn from(value: Test) -> Self {
        match value {
            Test::Frequency => sts_lib::Test::Frequency,
            Test::FrequencyWithinABlock => sts_lib::Test::FrequencyWithinABlock,
            Test::Runs => sts_lib::Test::Runs,
            Test::LongestRunOfOnes => sts_lib::Test::LongestRunOfOnes,
            Test::BinaryMatrixRank => sts_lib::Test::BinaryMatrixRank,
            Test::SpectralDft => sts_lib::Test::SpectralDft,
            Test::NonOverlappingTemplateMatching => sts_lib::Test::NonOverlappingTemplateMatching,
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

impl From<sts_lib::Test> for Test {
    fn from(value: sts_lib::Test) -> Self {
        match value {
            sts_lib::Test::Frequency => Test::Frequency,
            sts_lib::Test::FrequencyWithinABlock => Test::FrequencyWithinABlock,
            sts_lib::Test::Runs => Test::Runs,
            sts_lib::Test::LongestRunOfOnes => Test::LongestRunOfOnes,
            sts_lib::Test::BinaryMatrixRank => Test::BinaryMatrixRank,
            sts_lib::Test::SpectralDft => Test::SpectralDft,
            sts_lib::Test::NonOverlappingTemplateMatching => Test::NonOverlappingTemplateMatching,
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

impl TryFrom<RawTest> for Test {
    type Error = ();

    fn try_from(value: RawTest) -> Result<Self, Self::Error> {
        let test = match value {
            0 => Test::Frequency,
            1 => Test::FrequencyWithinABlock,
            2 => Test::Runs,
            3 => Test::LongestRunOfOnes,
            4 => Test::BinaryMatrixRank,
            5 => Test::SpectralDft,
            6 => Test::NonOverlappingTemplateMatching,
            7 => Test::OverlappingTemplateMatching,
            8 => Test::MaurersUniversalStatistical,
            9 => Test::LinearComplexity,
            10 => Test::Serial,
            11 => Test::ApproximateEntropy,
            12 => Test::CumulativeSums,
            13 => Test::RandomExcursions,
            14 => Test::RandomExcursionsVariant,
            _ => return Err(()),
        };

        Ok(test)
    }
}