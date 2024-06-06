//! Checks that a test work using the example inputs shown in the description of the tests
//! by NIST:

use crate::bitvec::BitVec;
use crate::frequency_test::frequency_test;
use crate::TestResult;

const LEVEL_VALUE: f64 = 0.01;

/// The books only give the values with 6 digits precision - rounding is always necessary
fn round_to_six_digits(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

/// Function to compare f64 values - == is not a good option
fn cmp_f64(got: f64, expected: f64) {
    assert!(
        f64::abs(got - expected) < f64::EPSILON,
        "Expected {expected}, got {got}"
    )
}

/// Test the frequency test (no. 1) - input and expected output from 2.1.4
#[test]
fn test_frequency_test_1() {
    let input = BitVec::from_ascii_str("1011010101").unwrap();

    let output = frequency_test(input);
    assert!(output.is_ok());

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    cmp_f64(round_to_six_digits(output.p_value), 0.527089);
}

/// Test the frequency test (
#[test]
fn test_frequency_test_2() {
    let input = BitVec::from_ascii_str("1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000")
        .unwrap();

    let output = frequency_test(input);
    assert!(output.is_ok());

    let output = output.unwrap();
    assert!(output.passed(LEVEL_VALUE));

    cmp_f64(round_to_six_digits(output.p_value), 0.109599);
}
