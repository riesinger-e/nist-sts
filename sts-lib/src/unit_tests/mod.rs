//! All unit tests

use crate::bitvec::BitVec;

mod nist_text_examples;

/// Test the creation of a bitvec from a bool vec
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