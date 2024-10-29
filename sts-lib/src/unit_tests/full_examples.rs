//! All tests in this module run the same statistical tests as specified in Appendix B of the
//! NIST SP 800-22r1a, with all expected results verified to be either the same as the ones
//! by NIST or an expected deviation because of recalculated constants.

use super::{assert_f64_eq, round, TEST_FILE_PATH};
use crate::bitvec::BitVec;
use crate::test_runner;
use crate::tests::approximate_entropy::ApproximateEntropyTestArg;
use crate::tests::frequency_block::FrequencyBlockTestArg;
use crate::tests::linear_complexity::LinearComplexityTestArg;
use crate::tests::serial::SerialTestArg;
use crate::tests::template_matching::non_overlapping::NonOverlappingTemplateTestArgs;
use crate::tests::template_matching::overlapping::OverlappingTemplateTestArgs;
use crate::{Test, TestArgs};
use std::collections::HashMap;
use std::fs;
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Test args. These are used by all tests in this module.
static TEST_ARGS: LazyLock<TestArgs> = LazyLock::new(|| TestArgs {
    frequency_block: FrequencyBlockTestArg::Bytewise(NonZero::new(128 / 8).unwrap()),
    non_overlapping_template: NonOverlappingTemplateTestArgs::new_const::<9, 8>(),
    overlapping_template: OverlappingTemplateTestArgs::new_nist_behaviour(9).unwrap(),
    linear_complexity: LinearComplexityTestArg::ManualBlockLength(NonZero::new(500).unwrap()),
    serial: SerialTestArg::new(16).unwrap(),
    approximate_entropy: ApproximateEntropyTestArg::new(10).unwrap(),
});

#[test]
fn pi_1e6() {
    let expected: HashMap<Test, Vec<(usize, f64)>> = [
        (Test::Frequency, vec![(0, 0.578211)]),
        (Test::OverlappingTemplateMatching, vec![(0, 0.296897)]),
        (Test::SpectralDft, vec![(0, 0.010186)]),
        (Test::LongestRunOfOnes, vec![(0, 0.027295)]),
        (Test::FrequencyWithinABlock, vec![(0, 0.380615)]),
        (Test::Runs, vec![(0, 0.419268)]),
        (Test::CumulativeSums, vec![(0, 0.628308), (1, 0.663369)]),
        (Test::Serial, vec![(0, 0.143005)]),
        (Test::RandomExcursionsVariant, vec![(8, 0.760966)]),
        (Test::NonOverlappingTemplateMatching, vec![(0, 0.165757)]),
        (Test::MaurersUniversalStatistical, vec![(0, 0.669012)]),
        (Test::BinaryMatrixRank, vec![(0, 0.122325)]),
        (Test::LinearComplexity, vec![(0, 0.246801)]),
        (Test::ApproximateEntropy, vec![(0, 0.361595)]),
        (Test::RandomExcursions, vec![(4, 0.844143)]),
    ]
    .into();

    let test_file = Path::new(TEST_FILE_PATH).join("pi.1e6.bin");
    common_parts(test_file, expected);
}
#[test]
fn e_1e6() {
    let expected: HashMap<Test, Vec<(usize, f64)>> = [
        (Test::CumulativeSums, vec![(0, 0.669886), (1, 0.724265)]),
        (Test::Runs, vec![(0, 0.561917)]),
        (Test::Frequency, vec![(0, 0.953749)]),
        (Test::RandomExcursionsVariant, vec![(8, 0.826009)]),
        (Test::FrequencyWithinABlock, vec![(0, 0.211072)]),
        (Test::LongestRunOfOnes, vec![(0, 0.718366)]),
        (Test::OverlappingTemplateMatching, vec![(0, 0.110434)]),
        (Test::LinearComplexity, vec![(0, 0.826202)]),
        (Test::MaurersUniversalStatistical, vec![(0, 0.282568)]),
        (Test::Serial, vec![(0, 0.766182)]),
        (Test::SpectralDft, vec![(0, 0.847187)]),
        (Test::NonOverlappingTemplateMatching, vec![(0, 0.07879)]),
        (Test::BinaryMatrixRank, vec![(0, 0.500518)]),
        (Test::ApproximateEntropy, vec![(0, 0.700073)]),
        (Test::RandomExcursions, vec![(4, 0.786868)]),
    ]
    .into();

    let test_file = Path::new(TEST_FILE_PATH).join("e.1e6.bin");
    common_parts(test_file, expected);
}

#[test]
fn sha1_1e6() {
    let expected = [
        (Test::OverlappingTemplateMatching, vec![(0, 0.339426)]),
        (Test::MaurersUniversalStatistical, vec![(0, 0.411079)]),
        (Test::FrequencyWithinABlock, vec![(0, 0.091517)]),
        (Test::ApproximateEntropy, vec![(0, 0.982885)]),
        (Test::BinaryMatrixRank, vec![(0, 0.345311)]),
        (Test::LongestRunOfOnes, vec![(0, 0.670504)]),
        (Test::CumulativeSums, vec![(0, 0.451231), (1, 0.550134)]),
        // returns 0.00 in release config - this bound check is turned off when testing, so this makes
        // no sense.
        (Test::RandomExcursions, vec![]),
        (Test::Runs, vec![(0, 0.309757)]),
        (Test::SpectralDft, vec![(0, 0.163062)]),
        (Test::NonOverlappingTemplateMatching, vec![(0, 0.496601)]),
        (Test::LinearComplexity, vec![(0, 0.304166)]),
        (Test::Frequency, vec![(0, 0.604458)]),
        // returns 0.00 in release config - this bound check is turned off when testing, so this makes
        // no sense.
        (Test::RandomExcursionsVariant, vec![]),
        (Test::Serial, vec![(0, 0.760793)]),
    ]
    .into();

    let test_file = Path::new(TEST_FILE_PATH).join("sha1.1e6.bin");
    common_parts(test_file, expected);
}

#[test]
fn sqrt2_1e6() {
    let expected = [
        (Test::RandomExcursionsVariant, vec![(8, 0.566118)]),
        (Test::LinearComplexity, vec![(0, 0.321866)]),
        (Test::NonOverlappingTemplateMatching, vec![(0, 0.569461)]),
        (Test::RandomExcursions, vec![(4, 0.216235)]),
        (Test::CumulativeSums, vec![(0, 0.879009), (1, 0.957206)]),
        (Test::MaurersUniversalStatistical, vec![(0, 0.130805)]),
        (Test::SpectralDft, vec![(0, 0.581909)]),
        (Test::BinaryMatrixRank, vec![(0, 0.785426)]),
        (Test::Frequency, vec![(0, 0.811881)]),
        (Test::FrequencyWithinABlock, vec![(0, 0.833222)]),
        (Test::Serial, vec![(0, 0.861925)]),
        (Test::ApproximateEntropy, vec![(0, 0.88474)]),
        (Test::Runs, vec![(0, 0.313427)]),
        (Test::LongestRunOfOnes, vec![(0, 0.013472)]),
        (Test::OverlappingTemplateMatching, vec![(0, 0.791982)]),
    ]
    .into();

    let test_file = Path::new(TEST_FILE_PATH).join("sqrt2.1e6.bin");
    common_parts(test_file, expected);
}

#[test]
fn sqrt3_1e6() {
    let expected = [
        (Test::Frequency, vec![(0, 0.610051)]),
        (Test::Serial, vec![(0, 0.1575)]),
        (Test::LinearComplexity, vec![(0, 0.338199)]),
        (Test::NonOverlappingTemplateMatching, vec![(0, 0.532235)]),
        (Test::FrequencyWithinABlock, vec![(0, 0.473961)]),
        (Test::ApproximateEntropy, vec![(0, 0.180481)]),
        (Test::RandomExcursions, vec![(4, 0.783283)]),
        (Test::CumulativeSums, vec![(0, 0.917121), (1, 0.689519)]),
        (Test::OverlappingTemplateMatching, vec![(0, 0.082716)]),
        (Test::BinaryMatrixRank, vec![(0, 0.157089)]),
        (Test::LongestRunOfOnes, vec![(0, 0.464612)]),
        (Test::SpectralDft, vec![(0, 0.776046)]),
        (Test::Runs, vec![(0, 0.261123)]),
        (Test::MaurersUniversalStatistical, vec![(0, 0.165981)]),
        (Test::RandomExcursionsVariant, vec![(8, 0.155066)]),
    ]
    .into();

    let test_file = Path::new(TEST_FILE_PATH).join("sqrt3.1e6.bin");
    common_parts(test_file, expected);
}

/// Common parts of all tests
fn common_parts(test_file: PathBuf, expected: HashMap<Test, Vec<(usize, f64)>>) {
    let data = fs::read(test_file).unwrap();
    let data = BitVec::from(data);
    assert_eq!(data.len_bit(), 1_000_000);

    for (test, result) in test_runner::run_all_tests(data, *TEST_ARGS).unwrap() {
        let result = result.unwrap();

        for &(idx, expected) in &expected[&test] {
            assert_f64_eq!(round(result[idx].p_value, 6), expected, test);
        }
    }
}
