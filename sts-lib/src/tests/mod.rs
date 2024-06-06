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