//! Opaque struct for the test arguments.

use crate::test_args::{TestArgApproximateEntropy, TestArgFrequencyBlock, TestArgLinearComplexity, TestArgNonOverlappingTemplate, TestArgOverlappingTemplate, TestArgSerial};

/// All test arguments for use in a *TestRunner*,
/// prefilled with sane defaults.
///
/// To set an argument, use the appropriate `runner_test_args_set_...` function.
#[derive(Copy, Clone, Debug)]
pub struct RunnerTestArgs(pub(super) sts_lib::TestArgs);

/// Create new [RunnerTestArgs], prefilled with sane defaults.
///
/// To set an argument, use the appropriate `runner_test_args_set_...` function.
///
/// The resulting pointer must be freed via [runner_test_args_destroy].
#[no_mangle]
pub extern "C" fn runner_test_args_new() -> &'static mut RunnerTestArgs {
    let args = sts_lib::TestArgs::default();
    let args = Box::new(RunnerTestArgs(args));

    Box::leak(args)
}

/// Destroy the given [RunnerTestArgs].
///
/// ## Safety
///
/// * `args` must have been created by [runner_test_args_new()]
/// * `args` must be valid for reads and writes and non-null.
/// * `args` may not be mutated for the duration of this call.
/// * `args` will be an invalid pointer after this call, trying to access its memory will lead to
///   undefined behaviour.
#[no_mangle]
pub unsafe extern "C" fn runner_test_args_destroy(args: &'static mut RunnerTestArgs) {
    // SAFETY: caller has to ensure that the pointer is valid.
    let _ = unsafe { Box::from_raw(args) };
}

macro_rules! setter {
    (
        $(#[$setter_comment: meta])*
        fn $name: ident($field_name: ident: $arg_type: ty);
    ) => {
        $(#[$setter_comment])*
        #[doc = ""]
        #[doc = " ## Safety"]
        #[doc = ""]
        #[doc = " * `args` must have been created by [runner_test_args_new()]"]
        #[doc = " * `args` must be valid for reads and writes and non-null."]
        #[doc = " * `args` may not be mutated for the duration of this call."]
        #[doc = " * `arg` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `arg` must be valid for reads and non-null."]
        #[doc = " * `arg` may not be mutated for the duration of this call."]
        #[doc = " * All responsibility for `arg`, particularly its deallocation, remains with the caller."]
        #[doc = "   This function copies the content of `arg`."]
        #[no_mangle]
        pub unsafe extern "C" fn $name(args: &mut RunnerTestArgs, arg: &$arg_type) {
            args.0.$field_name = arg.0;
        }
    };
}

setter! {
    /// Set the argument for the Frequency Block Test to the given value.
    fn runner_test_args_set_frequency_block(frequency_block: TestArgFrequencyBlock);
}

setter! {
    /// Set the argument for the Non-Overlapping Template Matching Test to the given value.
    fn runner_test_args_set_non_overlapping_template(non_overlapping_template: TestArgNonOverlappingTemplate);
}

setter! {
    /// Set the argument for the Overlapping Template Matching Test to the given value.
    fn runner_test_args_set_overlapping_template(overlapping_template: TestArgOverlappingTemplate);
}

setter! {
    /// Set the argument for the Linear Complexity Test to the given value.
    fn runner_test_args_set_linear_complexity(linear_complexity: TestArgLinearComplexity);
}

setter! {
    /// Set the argument for the Serial Test to the given value.
    fn runner_test_args_set_serial(serial: TestArgSerial);
}

setter! {
    /// Set the argument for the Approximate Entropy Test to the given value.
    fn runner_test_args_set_approximate_entropy(approximate_entropy: TestArgApproximateEntropy);
}