//! All unit tests

use crate::bitvec::BitVec;
use crate::tests::linear_complexity::berlekamp_massey;
use crate::tests::template_matching::overlapping::calculate_hamano_kaneko_pis;

mod full_examples;
mod nist_text_examples;

// Path to the test directory
const TEST_FILE_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test-files");

/// Macro to compare f64 values - == is not a good option because of floating point shenanigans.
macro_rules! assert_f64_eq {
    ($left:expr, $right:expr) => {
        let (got, expected) = ($left, $right);
        assert!(
            f64::abs(got - expected) < f64::EPSILON,
            "Expected {expected}, got {got}"
        );
    };
    ($left:expr, $right:expr, $prefix: tt) => {
        let (got, expected) = ($left, $right);
        assert!(
            f64::abs(got - expected) < f64::EPSILON,
            "{}: Expected {expected}, got {got}",
            $prefix
        );
    };
}

use assert_f64_eq;

/// The book only gives the value with reduces precision - rounding is nearly always necessary
fn round(value: f64, digits: u8) -> f64 {
    let t = f64::powi(10.0, digits as i32);
    (value * t).round() / t
}

/// Test the creation of a BitVec from a bool vec
#[test]
fn test_bitvec_from_bool() {
    let input_data = [
        true, false, true, true, false, true, false, true, false, true,
    ];

    let bitvec = BitVec::from(input_data.as_slice());

    // assert that length is the expected 10
    assert_eq!(bitvec.len_bit(), input_data.len());

    let bits = 0b10_1101_0101;
    let bits = bits << (usize::BITS as usize - input_data.len());
    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, 10);
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

    let bits = 0b10_1101_0101;
    let bits = bits << (usize::BITS as usize - input_data.len());
    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, 10);
}

/// Test the ASCII string parsing with an invalid ASCII string (should not work)
#[test]
fn test_bitvec_from_ascii_string_invalid() {
    let input_data = "10110b10101";

    let bitvec = BitVec::from_ascii_str(input_data);

    assert!(bitvec.is_none());
}

/// Test the lossy ASCII string parsing with invalid characters interspersed.
#[test]
fn test_bitvec_from_ascii_string_lossy() {
    let input_data = "101a101100b101010o100";
    let bit_len = 18;

    let bitvec = BitVec::from_ascii_str_lossy(input_data);

    // assert that length is the expected 18
    assert_eq!(bitvec.len_bit(), bit_len);

    let bits = 0b10_1101_1001_0101_0100;
    let bits = bits << (usize::BITS as usize - bit_len);
    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, bit_len as u8);
}

/// Test the lossy ASCII string parsing with a given max len
#[test]
fn test_bitvec_from_ascii_string_lossy_with_max_len() {
    let input_data = "101a101100b101010o100";

    for length in [14, 22] {
        let bitvec = BitVec::from_ascii_str_lossy_with_max_length(input_data, length);

        // assert that length is the expected 18
        assert_eq!(bitvec.len_bit(), usize::min(length, 18));
        if length == 14 {
            let bits = 0b10_1101_1001_0101;
            let bits = bits << (usize::BITS as usize - length);
            assert_eq!(&*bitvec.words, &[bits]);
        } else {
            let bits = 0b10_1101_1001_0101_0100;
            let bits = bits << (usize::BITS as usize - 18);
            assert_eq!(&*bitvec.words, &[bits]);
        }
    }
}

/// Test the creation of a BitVec from a Pointer.
#[test]
fn test_bitvec_from_c_str() {
    let input_data = c"1011010101";
    let input_len = 10; //count of characters

    // SAFETY: input_data is a valid CStr
    let bitvec = unsafe { BitVec::from_c_str(input_data.as_ptr()) };

    // assert that length is the expected 10
    assert_eq!(bitvec.len_bit(), input_len);

    let bits = 0b10_1101_0101;
    let bits = bits << (usize::BITS as usize - input_len);
    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, input_len as u8);
}

/// Test the c string pointer parsing with invalid characters interspersed.
#[test]
fn test_bitvec_from_c_str_lossy() {
    let input_data = c"101a101100b101010o100";
    let input_len = 18;

    let bitvec = unsafe { BitVec::from_c_str(input_data.as_ptr()) };

    // assert that length is the expected 18
    assert_eq!(bitvec.len_bit(), input_len);

    let bits = 0b10_1101_1001_0101_0100;
    let bits = bits << (usize::BITS as usize - input_len);

    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, input_len as u8);
}

/// Test the c string pointer parsing with invalid characters interspersed and a given max length.
#[test]
fn test_bitvec_from_c_str_with_max_len() {
    let input_data = c"101a101100b101010o100";

    for length in [13, 22] {
        let bitvec = unsafe { BitVec::from_c_str_with_max_length(input_data.as_ptr(), length) };

        // assert that length is the expected 18
        assert_eq!(bitvec.len_bit(), usize::min(length, 18));
        if length == 13 {
            let bits = 0b1_0110_1100_1010;
            let bits = bits << (usize::BITS as usize - length);
            assert_eq!(&*bitvec.words, &[bits]);
        } else {
            let bits = 0b10_1101_1001_0101_0100;
            let bits = bits << (usize::BITS as usize - 18);
            assert_eq!(&*bitvec.words, &[bits]);
        }
    }
}

/// Test bitvec cropping for more than 1 byte.
#[test]
fn test_bitvec_crop_more_than_1_byte() {
    let input_data = "10110101101101011011010101";
    let length = 26;

    let bitvec = BitVec::from_ascii_str(input_data);

    assert!(bitvec.is_some());

    let mut bitvec = bitvec.unwrap();

    // assert that length is the expected one
    assert_eq!(bitvec.len_bit(), length);

    let bits = 0b10_1101_0110_1101_0110_1101_0101;
    let bits = bits << (usize::BITS as usize - length);

    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, length as u8);

    let length = 11;
    bitvec.crop(length);

    assert_eq!(bitvec.len_bit(), length);

    let bits = 0b101_1010_1101;
    let bits = bits << (usize::BITS as usize - length);

    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, length as u8);
}

/// Test bitvec cropping for less than 1 byte.
#[test]
fn test_bitvec_crop_less_than_1_byte() {
    let input_data = "1011010101";
    let length = 10;

    let bitvec = BitVec::from_ascii_str(input_data);

    assert!(bitvec.is_some());

    let mut bitvec = bitvec.unwrap();

    assert_eq!(bitvec.len_bit(), length);

    let bits = 0b10_1101_0101;
    let bits = bits << (usize::BITS as usize - length);
    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, length as u8);

    let length = 9;
    bitvec.crop(length);

    assert_eq!(bitvec.len_bit(), length);

    let bits = 0b1_0110_1010;
    let bits = bits << (usize::BITS as usize - length);
    assert_eq!(&*bitvec.words, &[bits]);
    assert_eq!(bitvec.bit_count_last_word, length as u8);
}

/// Test the pi calculation according to Hamano and Kaneko. Used in the overlapping template matching
/// test.
#[test]
fn test_pi_calculation() {
    let block_length = 1032;
    let template_length = 9;
    let freedom = 6;

    let pis = calculate_hamano_kaneko_pis(block_length, template_length, freedom);
    let expected = [0.364091, 0.185659, 0.139381, 0.100571, 0.070432];

    for i in 0..5 {
        // round to six digits
        let pi = (pis[i] * 1_000_000.0).round() / 1_000_000.0;
        assert_f64_eq!(pi, expected[i]);
    }
}

/// Test the Berlekamp-Massey algorithm used in the linear complexity test.
#[test]
fn test_berlekamp_massey() {
    // start_bit is 0, everything in the sequence
    let value = 0b1_1010_1111_0001;
    let bit_len = 13;
    let start_bit = 0;
    let sequence = [value << (usize::BITS as usize - bit_len - start_bit)];

    assert_eq!(berlekamp_massey(&sequence, bit_len, start_bit), 4);

    // start bit is != 0
    let value = 0b01_1010_1111_0001;
    let bit_len = 13;
    let start_bit = 1;
    let sequence = [value << (usize::BITS as usize - bit_len - start_bit)];

    assert_eq!(berlekamp_massey(&sequence, bit_len, start_bit), 4);
}
