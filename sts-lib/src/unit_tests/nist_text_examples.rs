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
use crate::tests::template_matching::non_overlapping::{DEFAULT_BLOCK_COUNT, non_overlapping_template_matching_test, NonOverlappingTemplateTestArgs};

const LEVEL_VALUE: f64 = 0.01;
// Path to the test directory
const TEST_FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test-files");

/// The books only give the values with 6 digits precision - rounding is always necessary
fn round_to_six_digits(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
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

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.527089);
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

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.109599);
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

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.801252);
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

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.706438);
}

/// Test the runs test (no. 3) - input and expected output from 2.3.4
#[test]
fn test_runs_test_1() {
    let input = BitVec::from_ascii_str("1001101011").unwrap();

    let output = runs_test(&input);
    result_checker(&output);

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.147232);
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

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.500798);
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
    assert_f64_eq!(round_to_six_digits(output.p_value), 0.180609);
}

/// Test the binary matrix rank test (no. 5) - input and expected output from 2.5.8.
/// The values from 2.5.4 cannot be used here, because they would necessitate bitwise-matrices,
/// while the implementation only supports byte-wise matrices (because the paper itself says that
/// the implementation only gives usable values for 32x32 matrices)
#[test]
fn test_binary_matrix_rank_test() {
    let file_path = Path::new(TEST_FILE_PATH).join("e.1e5.bin");
    let length = 100_000;

    // create the file from the original nist sample data
    // let data = fs::read_to_string(&file_path).unwrap();
    //
    // let bitvec = BitVec::from_ascii_str_lossy_with_max_length(&data, 100_000);
    // assert_eq!(bitvec.len_bit(), length);
    // assert_eq!(bitvec.data.len(), length / BYTE_SIZE);
    // assert!(bitvec.remainder.is_empty());
    //
    // fs::write(file_path, bitvec.data).unwrap();

    // read in the converted data
    let data = fs::read(file_path).unwrap();
    let bitvec = BitVec::from(data);
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
    assert_f64_eq!(round_to_six_digits(output.p_value), 0.503604);
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
    assert_f64_eq!(round_to_six_digits(output.p_value), 0.468160);
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
    assert_f64_eq!(round_to_six_digits(output.p_value), 0.646355);
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

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.344154);
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

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.015021);
}