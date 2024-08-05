//! The statistical tests.

use sts_lib::tests;
use crate::test_result::TestResult;
use crate::bitvec::BitVec;
use crate::test_args::{
    TestArgFrequencyBlock,
    TestArgNonOverlappingTemplate,
    TestArgOverlappingTemplate,
    TestArgLinearComplexity,
    TestArgSerial,
    TestArgApproximateEntropy
};


/// Macro for generating a valid C function that calls the rust test internally.
macro_rules! test_wrapper {
    (
        $(#[$comment: meta])*
        fn $name: ident => $call: expr;
    ) => {
        $(#[$comment])*
        #[doc = ""]
        #[doc = " ## Return value"]
        #[doc = ""]
        #[doc = " If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`."]
        #[doc = " If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`."]
        #[doc = ""]
        #[doc = " ## Safety"]
        #[doc = ""]
        #[doc = " * `data` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `data` must be valid for reads and non-null."]
        #[doc = " * `data` may not be mutated for the duration of this call."]
        #[doc = " * All responsibility for `data`, particularly for its destruction, remains with the caller."]
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec) -> Option<Box<TestResult>> {
            let result = $call(&data.0);

            match result {
                Ok(res) => {
                    Some(Box::new(TestResult(res)))
                },
                Err(err) => {
                    // Set error for calling later
                    crate::set_last_from_error(err);
                    None
                }
            }
        }
    };
    (
        $(#[$comment: meta])*
        fn $name: ident(() => fixed_array($length: literal)) => $call: expr;
    ) => {
        $(#[$comment])*
        #[doc = ""]
        #[doc = " ## Return value"]
        #[doc = ""]
        #[doc = " If the test ran without errors, a list of `TestResult` is returned. This result can be deallocated with `test_result_list_destroy`."]
        #[doc = concat!(" The returned array always has length ", stringify!($length), ".")]
        #[doc = " If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`."]
        #[doc = ""]
        #[doc = " ## Safety"]
        #[doc = ""]
        #[doc = " * `data` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `data` must be valid for reads and non-null."]
        #[doc = " * `data` may not be mutated for the duration of this call."]
        #[doc = " * All responsibility for `data`, particularly for its destruction, remains with the caller."]
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec) -> *mut Box<TestResult> {
            let result = $call(&data.0);

            match result {
                Ok(res) => {
                    let vec: Box<[Box<TestResult>]> = Box::into_iter(Vec::from(res).into_boxed_slice())
                        .map(|res| Box::new(TestResult(res)))
                        .collect();
                    // strip away the length information
                    Box::into_raw(vec) as *mut Box<TestResult>
                },
                Err(err) => {
                    // Set error for calling later
                    crate::set_last_from_error(err);
                    std::ptr::null_mut()
                }
            }
        }
    };
    (
        $(#[$comment: meta])*
        fn $name: ident($argtype: ty) => $call: expr;
    ) => {
        $(#[$comment])*
        #[doc = ""]
        #[doc = " ## Return value"]
        #[doc = ""]
        #[doc = " If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`."]
        #[doc = " If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`."]
        #[doc = ""]
        #[doc = " ## Safety"]
        #[doc = ""]
        #[doc = " * `data` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `data` must be valid for reads and non-null."]
        #[doc = " * `data` may not be mutated for the duration of this call."]
        #[doc = " * `test_arg` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `test_arg` must be valid for reads and non-null."]
        #[doc = " * `test_arg` may not be mutated for the duration of this call."]
        #[doc = " * All responsibility for `data` and `test_arg`, particularly for their destruction, remains with the caller."]
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec, test_arg: &$argtype) -> Option<Box<TestResult>> {
            let result = $call(&data.0, test_arg.into());

            match result {
                Ok(res) => {
                    Some(Box::new(TestResult(res)))
                },
                Err(err) => {
                    // Set error for calling later
                    crate::set_last_from_error(err);
                    None
                }
            }
        }
    };
    (
        $(#[$comment: meta])*
        fn $name: ident($argtype: ty => dynamic_array) => $call: expr;
    ) => {
        $(#[$comment])*
        #[doc = ""]
        #[doc = " ## Return value"]
        #[doc = ""]
        #[doc = " If the test ran without errors, a list of `TestResult` is returned. This list can be deallocated with `test_result_list_destroy`."]
        #[doc = " The length of the returned list will be stored into `length`."]
        #[doc = " If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`."]
        #[doc = ""]
        #[doc = " ## Safety"]
        #[doc = ""]
        #[doc = " * `data` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `data` must be valid for reads and non-null."]
        #[doc = " * `data` may not be mutated for the duration of this call."]
        #[doc = " * `test_arg` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `test_arg` must be valid for reads and non-null."]
        #[doc = " * `test_arg` may not be mutated for the duration of this call."]
        #[doc = " * `length` must be valid for writes and non-null."]
        #[doc = " * `length` may not be mutated for the duration of this call."]
        #[doc = " * All responsibility for `data`, `test_arg` and `length`, particularly for their destruction, remains with the caller."]
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec, test_arg: &$argtype, length: &mut usize) -> *mut Box<TestResult> {
            let result = $call(&data.0, test_arg.into());

            match result {
                Ok(res) => {
                    *length = res.len();
                    let vec: Box<[Box<TestResult>]> = Box::into_iter(res.into_boxed_slice())
                        .map(|res| Box::new(TestResult(res)))
                        .collect();
                    // strip away the length information
                    Box::into_raw(vec) as *mut Box<TestResult>
                },
                Err(err) => {
                    // Set error for calling later
                    crate::set_last_from_error(err);
                    std::ptr::null_mut()
                }
            }
        }
    };
    (
        $(#[$comment: meta])*
        fn $name: ident($argtype: ty => fixed_array($length: literal)) => $call: expr;
    ) => {
        $(#[$comment])*
        #[doc = ""]
        #[doc = " ## Return value"]
        #[doc = ""]
        #[doc = " If the test ran without errors, a list of `TestResult` is returned. This list can be deallocated with `test_result_list_destroy`."]
        #[doc = concat!(" The returned array always has length ", stringify!($length), ".")]
        #[doc = " If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`."]
        #[doc = ""]
        #[doc = " ## Safety"]
        #[doc = ""]
        #[doc = " * `data` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `data` must be valid for reads and non-null."]
        #[doc = " * `data` may not be mutated for the duration of this call."]
        #[doc = " * `test_arg` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `test_arg` must be valid for reads and non-null."]
        #[doc = " * `test_arg` may not be mutated for the duration of this call."]
        #[doc = " * All responsibility for `data` and `test_arg`, particularly for their destruction, remains with the caller."]
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec, test_arg: &$argtype) -> *mut Box<TestResult> {
            let result = $call(&data.0, test_arg.into());

            match result {
                Ok(res) => {
                    let vec: Box<[Box<TestResult>]> = Box::into_iter(Vec::from(res).into_boxed_slice())
                        .map(|res| Box::new(TestResult(res)))
                        .collect();
                    Box::into_raw(vec) as *mut Box<TestResult>
                },
                Err(err) => {
                    // Set error for calling later
                    crate::set_last_from_error(err);
                    std::ptr::null_mut()
                }
            }
        }
    };
}

test_wrapper! {
    /// Frequency (mono bit) test - No. 1
    ///
    /// This test focuses on the numbers of ones and zeros in the sequence - the proportion should
    /// be roughly 50:50.
    fn frequency_test => tests::frequency::frequency_test;
}

test_wrapper! {
    /// Frequency Test within a block - No. 2
    ///
    /// This tests for the same property as [frequency_test], but within M-bit blocks.
    /// It is recommended that each block has a length of at least 100 bits.
    fn frequency_block_test(TestArgFrequencyBlock) => tests::frequency_block::frequency_block_test;
}

test_wrapper! {
    /// Runs test - No. 3
    ///
    /// This tests focuses on the number of runs in the sequence. A run is an uninterrupted sequence of
    /// identical bits.
    /// Each tested [BitVec] should have at least 100 bits length.
    fn runs_test => tests::runs::runs_test;
}

test_wrapper! {
    /// Test for the Longest Run of Ones in a Block - No. 4
    ///
    /// This test determines whether the longest run (See [runs_test]) of ones
    /// in a block is consistent with the expected value for a random sequence.
    ///
    /// An irregularity in the length of longest run of ones also implies an irregularity in the length
    /// of the longest runs of zeroes, meaning that only this test is necessary. See the NIST publication.
    ///
    /// The data has to be at least 128 bits in length.
    fn longest_run_of_ones_test => tests::longest_run_of_ones::longest_run_of_ones_test;
}

test_wrapper! {
    /// Binary Matrix Rank Test -  No. 5
    ///
    /// This test checks for linear dependence among fixed length substrings of the sequence.
    /// These substrings are interpreted as matrices of size 32x32.
    ///
    /// The sequence must consist of at least 38 912 bits = 4864 bytes.
    fn binary_matrix_rank_test => tests::binary_matrix_rank::binary_matrix_rank_test;
}

test_wrapper! {
    /// The Spectral Discrete Fourier Transform test - No. 6
    ///
    /// This test focuses on the peak heights in the DFT of the input sequence. This is used to detect
    /// periodic features that indicate a deviation from a random sequence.
    ///
    /// It is recommended (but not required) for the input to be of at least 1000 bits.
    fn spectral_dft_test => tests::spectral_dft::spectral_dft_test;
}

test_wrapper! {
    /// Non-overlapping Template Matching test - No. 7
    ///
    /// This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
    /// This test uses an m-bit window to search for an m-bit pattern.
    ///
    /// This test allows for parameters, see [TestArgNonOverlappingTemplate].
    fn non_overlapping_template_matching_test(TestArgNonOverlappingTemplate => dynamic_array) => tests::template_matching::non_overlapping::non_overlapping_template_matching_test;
}

test_wrapper! {
    /// Overlapping Template Matching test - No. 8
    ///
    /// This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
    /// This test uses an m-bit window to search for an m-bit pattern.
    /// The big difference to the [non_overlapping_template_matching_test] test is that template matches
    /// may overlap.
    ///
    /// The default arguments for this test derivate significantly from the NIST reference implementation,
    /// since the NIST reference implementation for this test is known bad.
    /// The problem is that the PI values from NIST are wrong - the correction from Hamano and Kaneko is used.
    ///
    /// Details about the problems:
    /// * Even though the pi values should be revised according to the paper, both the example and
    ///   the implementation still use the old, inaccurate calculation.
    /// * The (not working) fixed values according to Hamano and Kaneko only work for very specific cases.
    /// * The value *K*, as given in the paper, ist just wrong. You don't need a statistics degree to see
    ///   that it is 6 and not 5.
    ///
    /// This test needs arguments, see [TestArgOverlappingTemplate].
    ///
    /// This test enforces that the input length must be >= 10^6 bits. Smaller values will lead to
    /// an error!
    ///
    /// # About performance
    ///
    /// This test is quite slow in debug mode when using the more precise pi values (non-NIST behaviour),
    /// taking several seconds - it runs good when using release mode.
    /// For better performance, values that are calculated once are cached.
    fn overlapping_template_matching_test(TestArgOverlappingTemplate) => tests::template_matching::overlapping::overlapping_template_matching_test;
}

test_wrapper! {
    /// Maurer's "Universal Statistical" Test - No. 9
    ///
    /// This test detects if the given sequence if significantly compressible without information loss.
    /// If it is, it is considered non-random.
    ///
    /// The recommended minimum length of the sequence is 387 840 bits. The absolute minimum length to
    /// be used is 2020 bits, smaller inputs will raise an error.
    fn maurers_universal_statistical_test => tests::maurers_universal_statistical::maurers_universal_statistical_test;
}

test_wrapper! {
    /// The linear complexity test - No. 10
    ///
    /// This test determines the randomness of a sequence by calculating the minimum length of a linear
    /// feedback shift register that can create the sequence. Random sequences need longer LSFRs.
    ///
    /// This test needs a parameter, [TestArgLinearComplexity]. Additionally, the input sequence
    /// must have a minimum length of 10^6 bits. Smaller lengths will raise an error.
    fn linear_complexity_test(TestArgLinearComplexity) => tests::linear_complexity::linear_complexity_test;
}

test_wrapper! {
    /// The serial test - No. 11
    ///
    /// This test checks the frequency of all 2^m overlapping m-bit patterns in the sequence. Random
    /// sequences should be uniform. For *m = 1*, this would be the same as the
    /// [Frequency Test](frequency_test).
    ///
    /// This test needs a parameter [TestArgSerial]. Check the described constraints there.
    ///
    /// The paper describes the test slightly wrong: in 2.11.5 step 5, the second argument need to be
    /// halved in both *igamc* calculations. Only then are the calculated P-values equal to the P-values
    /// described in 2.11.6 and the reference implementation.
    ///
    /// The input length should be at least 2^19 bit, although this is not enforced. If the default
    /// value for [TestArgSerial] is used, a smaller input length will lead to an Error because
    /// of constraint no. 3!
    ///
    /// If the combination of the given data ([BitVec]) and [TestArgSerial] is invalid,
    /// an error is raised. For the exact constraints, see [TestArgSerial].
    fn serial_test(TestArgSerial => fixed_array(2)) => tests::serial::serial_test;
}

test_wrapper! {
    /// The approximate entropy test - No. 12
    ///
    /// This test is similar to the [serial test](serial_test). It compares the frequency
    /// of overlapping blocks with the two block lengths *m* and *m + 1* against the expected result
    /// of a random sequence.
    ///
    /// This test needs a parameter [TestArgApproximateEntropy]. Check the described constraints there.
    ///
    /// The input length should be at least 2^16 bit, although this is not enforced. If the default
    /// value for [TestArgApproximateEntropy] is used, a smaller input length will lead to an Error because
    /// of constraint no. 3!
    ///
    /// If the combination of the given data ([BitVec]) and [TestArgApproximateEntropy] is invalid,
    /// an error is raised. For the exact constraints, see [TestArgApproximateEntropy].
    fn approximate_entropy_test(TestArgApproximateEntropy) => tests::approximate_entropy::approximate_entropy_test;
}

test_wrapper! {
    /// The cumulative sums test - No. 13
    ///
    /// This test calculates cumulative partial sums of the bit sequence, once starting from the
    /// first bit and once starting from the last bit, adjusting the digits to -1 and +1 and calculating
    /// the maximum absolute partial sum. The test checks if this maximum is within the expected bounds
    /// for random sequences.
    ///
    /// The input sequence should be at least 100 bits in length, smaller sequences will raise
    /// an error.
    fn cumulative_sums_test(() => fixed_array(2)) => tests::cumulative_sums::cumulative_sums_test;
}

test_wrapper! {
    /// The random excursions test - No. 14.
    ///
    /// This test, similarly to the [cumulative sums test](cumulative_sums_test), calculates
    /// cumulative sums of a digit-adjusted (-1, +1) bit sequence, but only from the beginning to the end.
    /// This test checks if the frequency of cumulative sums values per cycle is as expected for
    /// a random sequence. A cycle consists of all cumulative sums between 2 "0"-values.
    ///
    /// Since the test needs at least 500 cycles to occur, bit sequences with fewer cycles will lead to an
    /// `Ok()` result, but with the values filled with "0.0".
    ///
    /// If the computation finishes successfully, 8 [TestResult] are returned: one for each tested state,
    /// `x`. The results will contain a comment about the state they are calculated from (e.g. "x = 3"),
    /// the order is: `[-4, -3, -2, -1, +1, +2, +3, +4]`.
    ///
    /// The input length must be at least 10^6 bits, otherwise, an error is raised.
    fn random_excursions_test(() => fixed_array(8)) => tests::random_excursions::random_excursions_test;
}

test_wrapper! {
    /// The random excursions variant test.
    ///
    /// This test is quite similar to the [random excursions test](random_excursions_test),
    /// with the key difference being that the frequencies are calculated over all cycles, instead of per
    /// cycle.
    ///
    /// This test does not require a minimum number of cycles.
    ///
    /// If the computation finishes successfully, 18 [TestResult] are returned: one for each tested state,
    /// `x`. The results will contain a comment about the state they are calculated from (e.g. "x = 3"),
    /// the order is: `[-9, -8, -7, -6, -5, -4, -3, -2, -1, +1, +2, +3, +4, +5, +6, +7, +8, +9]`.
    ///
    /// The input length must be at least 10^6 bits, otherwise, an error is returned.
    fn random_excursions_variant_test(() => fixed_array(18)) => tests::random_excursions_variant::random_excursions_variant_test;
}
