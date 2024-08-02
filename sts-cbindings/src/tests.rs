//! The statistical tests.

use sts_lib::tests;
use sts_lib::tests::approximate_entropy::ApproximateEntropyTestArg;
use sts_lib::tests::frequency_block::FrequencyBlockTestArg;
use sts_lib::tests::linear_complexity::LinearComplexityTestArg;
use sts_lib::tests::template_matching::non_overlapping::NonOverlappingTemplateTestArgs;
use sts_lib::tests::template_matching::overlapping::OverlappingTemplateTestArgs;
use sts_lib::tests::serial::SerialTestArg;
use crate::test_result::TestResult;
use crate::bitvec::BitVec;


// TODO: wrap each argument type as an opaque struct.

/// Macro for generating a valid C call. Works for cases with only one result.
macro_rules! test_wrapper {
    ($name: ident => $call: expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec) -> *const TestResult {
            let result = $call(&data.0);

            match result {
                Ok(res) => {
                    let res = TestResult(res);
                    Box::into_raw(Box::new(res))
                },
                Err(err) => {
                    // Set error for calling later
                    crate::LAST_ERROR.with_borrow_mut(|e| *e = Some(err.to_string()));
                    std::ptr::null()
                }
            }
        }
    };
    ($name: ident(() => fixed_array) => $call: expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec) -> *const *const TestResult {
            let result = $call(&data.0);

            match result {
                Ok(res) => {
                    let vec: Box<[*const TestResult]> = Box::into_iter(Vec::from(res).into_boxed_slice())
                        .map(|res| Box::into_raw(Box::new(TestResult(res))) as *const TestResult)
                        .collect();
                    Box::into_raw(vec) as *const *const TestResult
                },
                Err(err) => {
                    // Set error for calling later
                    crate::LAST_ERROR.with_borrow_mut(|e| *e = Some(err.to_string()));
                    std::ptr::null()
                }
            }
        }
    };
    ($name: ident($argtype: ty) => $call: expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec, test_arg: &$argtype) -> *const TestResult {
            let result = $call(&data.0, *test_arg);

            match result {
                Ok(res) => {
                    let res = TestResult(res);
                    Box::into_raw(Box::new(res))
                },
                Err(err) => {
                    // Set error for calling later
                    crate::LAST_ERROR.with_borrow_mut(|e| *e = Some(err.to_string()));
                    std::ptr::null()
                }
            }
        }
    };
    ($name: ident($argtype: ty => dynamic_array) => $call: expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec, test_arg: &$argtype, length: &mut usize) -> *const *const TestResult {
            let result = $call(&data.0, *test_arg);

            match result {
                Ok(res) => {
                    *length = res.len();
                    let vec: Box<[*const TestResult]> = Box::into_iter(res.into_boxed_slice())
                        .map(|res| Box::into_raw(Box::new(TestResult(res))) as *const TestResult)
                        .collect();
                    Box::into_raw(vec) as *const *const TestResult
                },
                Err(err) => {
                    // Set error for calling later
                    crate::LAST_ERROR.with_borrow_mut(|e| *e = Some(err.to_string()));
                    std::ptr::null()
                }
            }
        }
    };
    ($name: ident($argtype: ty => fixed_array) => $call: expr) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(data: &BitVec, test_arg: &$argtype) -> *const *const TestResult {
            let result = $call(&data.0, *test_arg);

            match result {
                Ok(res) => {
                    let vec: Box<[*const TestResult]> = Box::into_iter(Vec::from(res).into_boxed_slice())
                        .map(|res| Box::into_raw(Box::new(TestResult(res))) as *const TestResult)
                        .collect();
                    Box::into_raw(vec) as *const *const TestResult
                },
                Err(err) => {
                    // Set error for calling later
                    crate::LAST_ERROR.with_borrow_mut(|e| *e = Some(err.to_string()));
                    std::ptr::null()
                }
            }
        }
    };
}

test_wrapper!(frequency_test => tests::frequency::frequency_test);

test_wrapper!(frequency_block_test(FrequencyBlockTestArg) => tests::frequency_block::frequency_block_test);

test_wrapper!(runs_test => tests::runs::runs_test);

test_wrapper!(longest_run_of_ones_test => tests::longest_run_of_ones::longest_run_of_ones_test);

test_wrapper!(binary_matrix_rank_test => tests::binary_matrix_rank::binary_matrix_rank_test);

test_wrapper!(spectral_dft_test => tests::spectral_dft::spectral_dft_test);

// TODO: documentation about dynamic length
test_wrapper!(non_overlapping_template_matching_test(NonOverlappingTemplateTestArgs<'static> => dynamic_array) => tests::template_matching::non_overlapping::non_overlapping_template_matching_test);

test_wrapper!(overlapping_template_matching_test(OverlappingTemplateTestArgs) => tests::template_matching::overlapping::overlapping_template_matching_test);

test_wrapper!(maurers_universal_statistical_test => tests::maurers_universal_statistical::maurers_universal_statistic_test);

test_wrapper!(linear_complexity_test(LinearComplexityTestArg) => tests::linear_complexity::linear_complexity_test);

// TODO: serial: documentation about length guarantee
test_wrapper!(serial_test(SerialTestArg => fixed_array) => tests::serial::serial_test);

test_wrapper!(approximate_entropy_test(ApproximateEntropyTestArg) => tests::approximate_entropy::approximate_entropy_test);

// TODO: cusum: documentation about length guarantee
test_wrapper!(cumulative_sums_test(() => fixed_array) => tests::cumulative_sums::cumulative_sums_test);

// TODO: random excursions: documentation about length guarantee
test_wrapper!(random_excursions_test(() => fixed_array) => tests::random_excursions::random_excursions_test);

// TODO: random excursions variant: documentation about length guarantee
test_wrapper!(random_excursions_variant_test(() =>fixed_array) => tests::random_excursions_variant::random_excursions_variant_test);
