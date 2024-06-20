//! All unit tests

use crate::bitvec::BitVec;
use crate::tests::frequency_block_test::{frequency_block_test, FrequencyBlockTestArg};

mod nist_text_examples;

/// Macro to compare f64 values - == is not a good option because of floating point shenanigans.
macro_rules! assert_f64_eq {
    ($left:expr, $right:expr) => {
        let (got, expected) = ($left, $right);
        assert!(
            f64::abs(got - expected) < f64::EPSILON,
            "Expected {expected}, got {got}"
        );
    }
}

use assert_f64_eq;

/// Test the creation of a BitVec from a bool vec
#[test]
fn test_bitvec_from_bool() {
    let input_data = [true, false, true, true, false, true, false, true, false, true];

    let bitvec = BitVec::from(input_data.as_slice());

    // assert that length is the expected 10
    assert_eq!(bitvec.len_bit(), input_data.len());
    // the first 8 bits should be packed into 1 byte
    assert_eq!(&*bitvec.data, &[0b10110101]);
    // the remaining two bits should be here
    assert_eq!(&*bitvec.remainder, &[false, true])
}

/// Test the creation of a BitVec from an ASCII string
#[test]
fn test_bitvec_from_ascii_string() {
    let input_data = "1011010101";

    let bitvec = BitVec::from_ascii_str(input_data);

    assert!(bitvec.is_some());

    let bitvec = bitvec.unwrap();

    // assert that length is the expected 10
    assert_eq!(bitvec.len_bit(), input_data.len());
    // the first 8 bits should be packed into 1 byte
    assert_eq!(&*bitvec.data, &[0b10110101]);
    // the remaining two bits should be here
    assert_eq!(&*bitvec.remainder, &[false, true])
}

/// Test the creation of a BitVec from a Pointer.
#[test]
fn test_bitvec_from_c_str() {
    let input_data = c"1011010101";
    let input_len = 10; //count of characters

    // SAFETY: input_data is a valid CStr
    let bitvec = unsafe { BitVec::from_c_str(input_data.as_ptr()) };

    assert!(bitvec.is_some());

    let bitvec = bitvec.unwrap();

    // assert that length is the expected 10
    assert_eq!(bitvec.len_bit(), input_len);
    // the first 8 bits should be packed into 1 byte
    assert_eq!(&*bitvec.data, &[0b10110101]);
    // the remaining two bits should be here
    assert_eq!(&*bitvec.remainder, &[false, true])
}

/// Assert that the bitwise and byte-wise version of the frequency block test (No.2) do the same thing
#[test]
fn test_frequency_block_bytewise_vs_bitwise() {
    let input = BitVec::from_ascii_str("1100100100001111110110101010001000100001011010001100001000110100110001001100011001100010100010111000")
        .unwrap();

    // Same argument, but differently expressed
    let arg1 = FrequencyBlockTestArg::Bitwise(16);
    let arg2 = FrequencyBlockTestArg::Bytewise(2);

    let res1 = frequency_block_test(&input, arg1)
        .unwrap();
    let res2 = frequency_block_test(&input, arg2)
        .unwrap();
    assert_f64_eq!(res1.p_value, res2.p_value);
}