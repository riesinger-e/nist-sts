//! Checks that a test work using the example inputs shown in the description of the tests
//! by NIST:

use crate::bitvec::BitVec;
use crate::tests::frequency_block_test::{frequency_block_test, FrequencyBlockTestArg};
use crate::tests::frequency_test::frequency_test;
use crate::tests::runs_test::runs_test;
use super::assert_f64_eq;

const LEVEL_VALUE: f64 = 0.01;

/// The books only give the values with 6 digits precision - rounding is always necessary
fn round_to_six_digits(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

/// Test the frequency test (no. 1) - input and expected output from 2.1.4
#[test]
fn test_frequency_test_1() {
    let input = BitVec::from_ascii_str("1011010101").unwrap();

    let output = frequency_test(&input);
    assert!(output.is_ok());

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
    assert!(output.is_ok());

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.109599);
}

/// Test the frequency within a block test (no. 2) - input and expected output from 2.2.4
#[test]
fn test_frequency_block_test_1() {
    let input = BitVec::from_ascii_str("0110011010").unwrap();
    let arg = FrequencyBlockTestArg::Bitwise(3);

    let output = frequency_block_test(&input, arg);
    assert!(output.is_ok());

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.801252);
}

/// Test the frequency within a block test (no. 2) - input and expected output from 2.2.8
#[test]
fn test_frequency_block_test_2() {
    let input = BitVec::from_ascii_str("1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000")
        .unwrap();
    let arg = FrequencyBlockTestArg::new(10);

    let output = frequency_block_test(&input, arg);
    assert!(output.is_ok());

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.706438);
}

/// Test the runs test (no. 3) - input and expected output from 2.3.4
#[test]
fn test_runs_test_1() {
    let input = BitVec::from_ascii_str("1001101011")
        .unwrap();

    let output = runs_test(&input);
    assert!(output.is_ok());

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
    assert!(output.is_ok());

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    assert_f64_eq!(round_to_six_digits(output.p_value), 0.500798);
}