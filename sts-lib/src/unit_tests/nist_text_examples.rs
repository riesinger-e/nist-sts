//! Checks that a test work using the example inputs shown in the description of the tests
//! by NIST:

use super::assert_f64_eq;
use crate::bitvec::BitVec;
use crate::tests::binary_matrix_rank::binary_matrix_rank_test;
use crate::tests::frequency::frequency_test;
use crate::tests::frequency_block::{frequency_block_test, FrequencyBlockTestArg};
use crate::tests::longest_run_of_ones::longest_run_of_ones_test;
use crate::tests::template_matching::TemplateArg;
use crate::tests::runs::runs_test;
use crate::tests::spectral_dft::spectral_dft_test;
use crate::{BYTE_SIZE, Error};
use std::fs;
use std::num::NonZero;
use std::path::Path;
use crate::tests::approximate_entropy::{approximate_entropy_test, ApproximateEntropyTestArg};
use crate::tests::cumulative_sums::{cumulative_sums_test, cusum_test_internal};
use crate::tests::linear_complexity::{linear_complexity_test, LinearComplexityTestArg};
use crate::tests::maurers_universal_statistical::maurers_universal_statistical_test;
use crate::tests::random_excursions::random_excursions_test;
use crate::tests::random_excursions_variant::random_excursions_variant_test;
use crate::tests::serial::{serial_test, SerialTestArg};
use crate::tests::template_matching::non_overlapping::{DEFAULT_BLOCK_COUNT, non_overlapping_template_matching_test, NonOverlappingTemplateTestArgs};
use crate::tests::template_matching::overlapping::{overlapping_template_matching_test, OverlappingTemplateTestArgs};

const LEVEL_VALUE: f64 = 0.01;
// Path to the test directory
const TEST_FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test-files");

/// The book only gives the value with reduces precision - rounding is nearly always necessary
fn round(value: f64, digits: u8) -> f64 {
    let t = f64::powi(10.0, digits as i32);
    (value * t).round() / t
}

/// Check the test result: Assert that it is OK and print the error if it is not.
fn result_checker<T>(output: &Result<T, Error>) {
    if let Err(e) = output {
        println!("Error: {e}")
    }
    assert!(output.is_ok())
}

/// Test the frequency test (no. 1) - input and expected output from 2.1.4
#[test]
fn test_frequency_test_1() {
    let input = BitVec::from_ascii_str("1011010101").unwrap();

    let output = frequency_test(&input);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.527089);
}

/// Test the frequency test (no.1) - input and expected output from 2.1.8
#[test]
fn test_frequency_test_2() {
    let input = BitVec::from_ascii_str("1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000")
        .unwrap();

    let output = frequency_test(&input);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.109599);
}

/// Test the frequency within a block test (no. 2) - input and expected output from 2.2.4
#[test]
fn test_frequency_block_test_1() {
    let input = BitVec::from_ascii_str("0110011010").unwrap();
    let arg = FrequencyBlockTestArg::Bitwise(NonZero::new(3).unwrap());

    let output = frequency_block_test(&input, arg);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.801252);
}

/// Test the frequency within a block test (no. 2) - input and expected output from 2.2.8
#[test]
fn test_frequency_block_test_2() {
    let input = BitVec::from_ascii_str("1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000")
        .unwrap();
    let arg = FrequencyBlockTestArg::new(NonZero::new(10).unwrap());

    let output = frequency_block_test(&input, arg);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.706438);
}

/// Test the runs test (no. 3) - input and expected output from 2.3.4
#[test]
fn test_runs_test_1() {
    let input = BitVec::from_ascii_str("1001101011").unwrap();

    let output = runs_test(&input);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.147232);
}

/// Test the runs test (no. 3) - input and expected output from 2.3.8
#[test]
fn test_runs_test_2() {
    let input = BitVec::from_ascii_str("1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000")
        .unwrap();

    let output = runs_test(&input);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.500798);
}

/// Test the longest run of ones in a block test (no. 4) - input and expected output from 2.4.8
#[test]
fn test_longest_run_of_ones() {
    let input = BitVec::from_ascii_str("11001100000101010110110001001100111000000000001001001101010100010001001111010110100000001101011111001100111001101101100010110010")
        .unwrap();

    let output = longest_run_of_ones_test(&input);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    // the expected value differs slightly from the textbook values because some constants
    // were recalculated with higher precision.
    assert_f64_eq!(round(output.p_value, 6), 0.180609);
}

/// Test the binary matrix rank test (no. 5) - input and expected output from 2.5.8.
/// The values from 2.5.4 cannot be used here, because they would necessitate bitwise-matrices,
/// while the implementation only supports byte-wise matrices (because the paper itself says that
/// the implementation only gives usable values for 32x32 matrices)
#[test]
fn test_binary_matrix_rank_test() {
    let file_path = Path::new(TEST_FILE_PATH).join("e.1e6.bin");
    let length = 100_000;

    // read in the test data
    let data = fs::read(file_path).unwrap();
    let mut bitvec = BitVec::from(data);
    bitvec.crop(length);
    assert_eq!(bitvec.len_bit(), length);
    assert_eq!(bitvec.data.len(), length / BYTE_SIZE);
    assert!(bitvec.remainder.is_empty());

    // run the test
    let output = binary_matrix_rank_test(&bitvec);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    // the expected value differs slightly from the values from the paper because
    // 1. some constants were recalculated with higher precision.
    // 2. the values in the text book are just (slightly) WRONG! - try calculating chi^2 yourself with
    //    the F_M, F_{M-1} and (N - F_M - F_{M-1}) according to the paper, it does not match!
    assert_f64_eq!(round(output.p_value, 6), 0.503604);
}

/// Test the spectral dft test (no 6.) - input and output taken from 2.6.4
#[test]
fn test_spectral_dft_1() {
    let input = BitVec::from_ascii_str("1001010011").unwrap();

    let output = spectral_dft_test(&input);

    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    // This result is not taken from the paper itself, instead, the original NIST STS was run.
    // The value in the paper is completely wrong!
    assert_f64_eq!(round(output.p_value, 6), 0.468160);
}

/// Test the spectral dft test (no 6.) - input and output taken from 2.6.8
#[test]
fn test_spectral_dft_2() {
    let input = BitVec::from_ascii_str("1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000")
        .unwrap();

    let output = spectral_dft_test(&input);

    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    // Again: calculated with the original NIST STS, value in the paper is wrong!
    assert_f64_eq!(round(output.p_value, 6), 0.646355);
}

/// Test the Non-Overlapping Template Matching test (no. 7) - input and output taken from 2.7.4
#[test]
fn test_non_overlapping_template_matching_1() {
    let input = BitVec::from_ascii_str("10100100101110010110").unwrap();

    let template = [0b0010_0000_u8];
    let templates = [template.as_slice()];
    let template_len = 3;
    let count_blocks = 2;
    let template_arg = TemplateArg::new_with_custom_templates(
        templates.as_slice(),
        template_len,
    )
    .unwrap();
    let test_arg = NonOverlappingTemplateTestArgs::new_with_custom_template(template_arg, count_blocks)
        .unwrap();

    let output = non_overlapping_template_matching_test(&input, test_arg);

    result_checker(&output);

    let output = output.unwrap()[0];
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.344154);
}

/// Test the Non-Overlapping Template Matching test (no. 7) - input and output taken from 2.7.8
///
/// # Problems with this test
///
/// The test from the paper is not reproducible - the output that the reference implementation
/// gives for 2^20 bits from the G-SHA-1 generator does not agree with the paper.
///
/// Because of this, the input pattern and the result value were chosen from the output of
/// the NIST reference implementation.
#[test]
fn test_non_overlapping_template_matching_2() {
    // This test depends on a test file: the first 2^20 bits of the G-SHA-1 generator
    let file_path = Path::new(TEST_FILE_PATH).join("sha1.1Mi.bin");
    let length = 1<<20;
    let data = fs::read(file_path).unwrap();
    let input = BitVec::from(data);
    assert_eq!(input.len_bit(), length);

    let template: [u8; 2] = [0b0010_1111, 0b1000_0000];
    let templates = [template.as_slice()];
    let template_len = 9;
    let template_arg = TemplateArg::new_with_custom_templates(
        templates.as_slice(),
        template_len,
    )
        .unwrap();
    let test_arg = NonOverlappingTemplateTestArgs::new_with_custom_template(template_arg, DEFAULT_BLOCK_COUNT)
        .unwrap();
    
    let output = non_overlapping_template_matching_test(&input, test_arg);

    result_checker(&output);

    let output = output.unwrap()[0];
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.015021);
}

/// Test the overlapping template matching test (no. 8) - input and expected output from 2.8.8.
/// The values from 2.8.4 cannot be used here, because they would necessitate supporting different
/// freedom degrees, which the reference implementation itself does not.
///
/// # Problems with this test
///
/// The results shown in the paper use inaccurate values for the pis. This is mitigated by using
/// the pi values from the reference implementation, which is entirely inaccurate for most 
/// bigger sequences (> 1e6).
#[test]
fn test_overlapping_template_matching_test() {
    let file_path = Path::new(TEST_FILE_PATH).join("e.1e6.bin");
    let length = 1_000_000;

    // read in the test data
    let data = fs::read(file_path).unwrap();
    let bitvec = BitVec::from(data);
    assert_eq!(bitvec.len_bit(), length);
    assert_eq!(bitvec.data.len(), length / BYTE_SIZE);
    assert!(bitvec.remainder.is_empty());

    // create the custom template
    let arg = OverlappingTemplateTestArgs::new_nist_behaviour(9)
        .unwrap();

    // run the test
    let output = overlapping_template_matching_test(&bitvec, arg);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    // This value is taken from the NIST reference implementation since the paper is (yet again) wrong.
    assert_f64_eq!(round(output.p_value, 6), 0.110434);
}

/// Test Maurer's "Universal Statistical" Test (no. 9).
///
/// Input values are similar to 2.9.8, output values are taken from the NIST reference implementation.
///
/// # Problems with the examples from the paper
///
/// 1. The values form 2.9.4 cannot be used here, because the parameters used in the example
///    are not valid for real tests.
/// 2. Because the parameters for the G-SHA-1 generator used in 2.9.8 are unknown, the exact sequence
///    used in the example is unknown and the values from 2.9.8 can also not be used.
#[test]
fn test_maurers_universal_statistical_test() {
    let file_path = Path::new(TEST_FILE_PATH).join("e.1e6.bin");
    let length = 1_000_000;

    // read in the test data
    let data = fs::read(file_path).unwrap();
    let bitvec = BitVec::from(data);
    assert_eq!(bitvec.len_bit(), length);

    // run the test
    let output = maurers_universal_statistical_test(&bitvec);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.282568);
}


/// Test the linear complexity test (no. 10).
/// Input an output values are taken from 2.10.8. 2.10.4. has no complete example given.
///
/// # Problems with the test
///
/// The test result is directly from the NIST reference implementation, which has one
/// critical error: pi\[0] is given as 0.01047 instead of the 0.010417 it should be. This leads
/// to a wrong result, it was manually verified that this implementation is correct (by temporarily
/// changing the constant to the wrong value).
///
/// Because of this, the output value is not exactly the one given in 2.10.8, but instead the one
/// it would be if the correct constant was used.
#[test]
fn test_linear_complexity_test() {
    let file_path = Path::new(TEST_FILE_PATH).join("e.1e6.bin");
    let length = 1_000_000;

    // read in the test data
    let data = fs::read(file_path).unwrap();
    let bitvec = BitVec::from(data);
    assert_eq!(bitvec.len_bit(), length);

    // construct the argument
    let arg = LinearComplexityTestArg::ManualBlockLength(NonZero::new(1000).unwrap());

    // run the test
    let output = linear_complexity_test(&bitvec, arg);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round(output.p_value, 6), 0.844738);
}

/// Test the serial test (no. 11) - input from 2.11.4, output from 2.11.6
///
/// # Problems with the test
///
/// The output described in 2.11.5 step 5 is just wrong, 2.11.6 contains the correct output.
/// Also, as described in the [test module documentation](crate::tests::serial), the description
/// in 2.11.5 step 5 is wrong.
#[test]
fn test_serial_test_1() {
    let data = BitVec::from_ascii_str("0011011101").unwrap();
    let test_arg = SerialTestArg::new(3).unwrap();

    let output = serial_test(&data, test_arg);

    result_checker(&output);

    let output = output.unwrap();
    assert!(output[0].passed(LEVEL_VALUE));
    assert_f64_eq!(round(output[0].p_value, 6), 0.808792);
    assert!(output[1].passed(LEVEL_VALUE));
    assert_f64_eq!(round(output[1].p_value, 6), 0.670320);
}

/// Test the serial test (no. 11) - input and output from 2.11.8
#[test]
fn test_serial_test_2() {
    let file_path = Path::new(TEST_FILE_PATH).join("e.1e6.bin");
    let length = 1_000_000;

    // read in the test data
    let data = fs::read(file_path).unwrap();
    let data = BitVec::from(data);
    assert_eq!(data.len_bit(), length);

    let test_arg = SerialTestArg::new(2).unwrap();

    let output = serial_test(&data, test_arg);

    result_checker(&output);

    let output = output.unwrap();
    assert!(output[0].passed(LEVEL_VALUE));
    assert_f64_eq!(round(output[0].p_value, 6), 0.843764);
    assert!(output[1].passed(LEVEL_VALUE));
    assert_f64_eq!(round(output[1].p_value, 6), 0.561915);
}

/// Test the approximate entropy test (no. 12) - input and output from 2.12.4
#[test]
fn test_approximate_entropy_test_1() {
    let data = BitVec::from_ascii_str("0100110101").unwrap();

    let test_arg = ApproximateEntropyTestArg::new(3)
        .unwrap();

    let output = approximate_entropy_test(&data, test_arg);

    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));
    assert_f64_eq!(round(output.p_value, 6), 0.261961);
}

/// Test the approximate entropy test (no. 12) - input and output from 2.12.8
#[test]
fn test_approximate_entropy_test_2() {
    let data = BitVec::from_ascii_str(
        "1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000"
    ).unwrap();
    assert_eq!(data.len_bit(), 100);

    let test_arg = ApproximateEntropyTestArg::new(2)
        .unwrap();

    let output = approximate_entropy_test(&data, test_arg);

    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));
    assert_f64_eq!(round(output.p_value, 6), 0.235301);
}

/// Test the cumulative sum test (no. 13) - input and output taken from 2.13.4
#[test]
fn test_cumulative_sums_test_1() {
    let data = BitVec::from_ascii_str("1011010111")
        .unwrap();

    let output = cusum_test_internal(&data, false);

    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));
    // Expected value is slightly different from the paper because of a different method to
    // calculate the standard normal cumulative distribution function. Diff: 1e-6
    assert_f64_eq!(round(output.p_value, 6), 0.411659);
}

/// Test the cumulative sum test (no. 13) - input and output taken from 2.13.8
#[test]
fn test_cumulative_sums_test_2() {
    let data = BitVec::from_ascii_str(
        "1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000"
    ).unwrap();

    let output = cumulative_sums_test(&data);

    result_checker(&output);

    let output = output.unwrap();
    assert!(output[0].passed(LEVEL_VALUE));
    assert_f64_eq!(round(output[0].p_value, 6), 0.219194);
    assert!(output[1].passed(LEVEL_VALUE));
    assert_f64_eq!(round(output[1].p_value, 6), 0.114866);
}

/// Test the random excursions test (no. 14) - input and output taken from 2.14.4.
///
/// ## Problems with this test
///
/// Since the constants used in the paper are not really precise, with
/// the recalculated constants used in this test being much better, the calculated result deviates
/// from the result in the paper (beginning with step 7). This test passes because the result was
/// manually recalculated with the new constants.
#[test]
fn test_random_excursions_test_1() {
    let data = BitVec::from_ascii_str("0110110101")
        .unwrap();

    let output = random_excursions_test(&data);

    result_checker(&output);

    let output = output.unwrap();

    assert!(output[4].passed(LEVEL_VALUE));
    assert_f64_eq!(round(output[4].p_value, 6), 0.502488);
}

/// Test the random excursions test (no. 14) - input and output taken from 2.14.8.
///
/// ## Problems with this test
///
/// See [test_random_excursions_test_1]. Once again, the difference was verified manually to be a
/// result of the changed constants.
#[test]
fn test_random_excursions_test_2() {
    let file_path = Path::new(TEST_FILE_PATH).join("e.1e6.bin");
    let length = 1_000_000;

    // read in the test data
    let data = fs::read(file_path).unwrap();
    let data = BitVec::from(data);
    assert_eq!(data.len_bit(), length);

    let output = random_excursions_test(&data);

    result_checker(&output);

    let output = output.unwrap();
    let expected_values = [0.573306, 0.197996, 0.164011, 0.007779, 0.786868, 0.440912, 0.797854, 0.778186];

    for (result, expected) in output.into_iter().zip(expected_values) {
        assert_f64_eq!(round(result.p_value, 6), expected);
    }
}

/// Test the random excursions variant test (no. 15) - input and output taken from 2.15.4.
#[test]
fn test_random_excursions_variant_test_1() {
    let data = BitVec::from_ascii_str("0110110101")
        .unwrap();

    let output = random_excursions_variant_test(&data);

    result_checker(&output);

    let output = output.unwrap();

    assert!(output[9].passed(LEVEL_VALUE));
    assert_f64_eq!(round(output[9].p_value, 6), 0.683091);
}

/// Test the random excursions variant test (no. 15) - input and output taken from 2.15.8
#[test]
fn test_random_excursions_variant_test_2() {
    let file_path = Path::new(TEST_FILE_PATH).join("e.1e6.bin");
    let length = 1_000_000;

    // read in the test data
    let data = fs::read(file_path).unwrap();
    let data = BitVec::from(data);
    assert_eq!(data.len_bit(), length);

    let output = random_excursions_variant_test(&data);

    result_checker(&output);

    let output = output.unwrap();
    let expected_values = [
        0.858946, 0.794755, 0.576249, 0.493417, 0.633873, 0.917283, 0.934708, 0.816012, 0.826009,
        0.137861, 0.200642, 0.441254, 0.939291, 0.505683, 0.445935, 0.512207, 0.538635, 0.593930,
    ];

    for (result, expected) in output.into_iter().zip(expected_values) {
        assert_f64_eq!(round(result.p_value, 6), expected);
    }
}