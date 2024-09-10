//! Wrapper types for the test arguments.

use std::num::NonZero;
use sts_lib::tests::{
    approximate_entropy, frequency_block, linear_complexity, serial,
    template_matching::{non_overlapping, overlapping},
};

/// Macro for automatically creating the necessary struct for a test argument.
macro_rules! test_arg {
    (
        $(#[$struct_comment: meta])*
        struct $name: ident ($inner: ty);

        $(#[$default_comment: meta])*
        fn $default_name: ident() -> Self;

        $(#[$destructor_comment: meta])*
        fn $destructor_name: ident(self);
    ) => {
        $(#[$struct_comment])*
        #[derive(Copy, Clone)]
        pub struct $name(pub(crate) $inner);

        $(#[$default_comment])*
        #[doc = " This function never returns `NULL`."]
        #[no_mangle]
        pub extern "C" fn $default_name() -> Box<$name> {
            let arg: $inner = Default::default();
            Box::new($name(arg))
        }

        $(#[$destructor_comment])*
        #[doc = ""]
        #[doc = " ## Safety"]
        #[doc = ""]
        #[doc = " * `ptr` must have been created by one of the construction methods provided by this library."]
        #[doc = " * `ptr` must be valid for reads and writes and non-null."]
        #[doc = " * `ptr` will be invalid after this call, access will lead to undefined behaviour."]
        #[doc = " * `ptr` may not be mutated for the duration of this call."]
        #[no_mangle]
        pub unsafe extern "C" fn $destructor_name(ptr: Box<$name>) {
            // drop the pointer
            _ = ptr;
        }

        impl From<&$name> for $inner {
            fn from(value: &$name) -> Self {
                value.0
            }
        }
    }
}

// frequency test within a block
test_arg! {
    /// The argument for the Frequency test within a block: the block length.
    ///
    /// The block length should be at least 20 bits, with the block length greater than 1% of the
    /// total bit length and fewer than 100 total blocks.
    struct TestArgFrequencyBlock(frequency_block::FrequencyBlockTestArg);

    /// Creates a default new argument for the Frequency test within a block that chooses a suitable
    /// block length automatically.
    fn test_arg_frequency_block_default() -> Self;

    /// Destroys the given argument for the Frequency test within a block.
    fn test_arg_frequency_block_destroy(self);
}

/// Creates a new argument for the Frequency test within a block, specifying the block length in bits.
///
/// ## Return values
/// - if the given `block_length == 0`, `NULL` is returned.
/// - if the given `block_length != 0`, a pointer to the argument is returned.
#[no_mangle]
pub extern "C" fn test_arg_frequency_block_new(
    block_length: usize,
) -> Option<Box<TestArgFrequencyBlock>> {
    NonZero::new(block_length).map(|block_length| {
        let arg = frequency_block::FrequencyBlockTestArg::new(block_length);
        Box::new(TestArgFrequencyBlock(arg))
    })
}

// non-overlapping template matching
test_arg! {
    /// The arguments for the Non-overlapping Template Matching Test.
    ///
    /// 1. The templates length to use within a block: `m`.
    ///    2 <= `m` <= 21 - recommended: 9.
    /// 2. The number of independent blocks to test in the sequence: `N`
    ///    1 <= `N` < 100 - recommended: 8
    ///
    /// These bounds are checked by all creation functions.
    ///
    /// You can also use [NON_OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_COUNT] and
    /// [NON_OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LEN].
    struct TestArgNonOverlappingTemplate(non_overlapping::NonOverlappingTemplateTestArgs<'static>);

    /// Creates a default new non-overlapping template test argument that chooses its template length
    /// and block count according to the values recommended by NIST.
    fn test_arg_non_overlapping_template_default() -> Self;

    /// Destroys the given argument for Non-overlapping Template Matching Test.
    fn test_arg_non_overlapping_template_destroy(self);
}

/// Creates a new non-overlapping template test argument with the specified template length and block
/// count.
///
/// ## Return values.
/// * If both arguments are within the bounds specified in [TestArgNonOverlappingTemplate]: the new argument.
/// * Otherwise: `NULL`
#[no_mangle]
pub extern "C" fn test_arg_non_overlapping_template_new(
    template_len: usize,
    count_blocks: usize,
) -> Option<Box<TestArgNonOverlappingTemplate>> {
    non_overlapping::NonOverlappingTemplateTestArgs::new(template_len, count_blocks)
        .map(|arg| Box::new(TestArgNonOverlappingTemplate(arg)))
}

// overlapping template matching
test_arg! {
    /// The arguments for the Overlapping Template Matching Test.
    ///
    /// 1. The template length *m*. 2 <= *m* <= 21. See [OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LENGTH].
    /// 2. The length of each block, *M*, in bits. See [OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_LENGTH].
    /// 3. The degrees of freedom, *K*. See [OVERLAPPING_TEMPLATE_DEFAULT_FREEDOM].
    ///
    /// With these arguments the *pi* values are calculated according to Hamano and Kaneko.
    /// These bounds are checked by all creation functions.
    ///
    /// The original NIST implementation has some glaring inaccuracies,
    /// to replicate this exact NIST behaviour, use [test_arg_overlapping_template_new_nist_behaviour]
    struct TestArgOverlappingTemplate(overlapping::OverlappingTemplateTestArgs);

    /// Creates a new argument for the Overlapping Template Matching Test, using the default values
    /// [OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LENGTH], [OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_LENGTH]
    /// and [OVERLAPPING_TEMPLATE_DEFAULT_FREEDOM].
    fn test_arg_overlapping_template_default() -> Self;

    /// Destroys the given argument for the Overlapping Template Matching Test.
    fn test_arg_overlapping_template_destroy(self);
}

/// Creates a new Overlapping Template Matching Test argument with the specified template length, block
/// count and degrees of freedom.
///
/// ## Return values.
/// * If all arguments are within the bounds specified in [TestArgOverlappingTemplate]: the new argument.
/// * Otherwise: `NULL`
#[no_mangle]
pub extern "C" fn test_arg_overlapping_template_new(
    template_length: usize,
    block_length: usize,
    freedom: usize,
) -> Option<Box<TestArgOverlappingTemplate>> {
    overlapping::OverlappingTemplateTestArgs::new(template_length, block_length, freedom)
        .map(|arg| Box::new(TestArgOverlappingTemplate(arg)))
}

/// Creates a new Overlapping Template Matching Test argument with the specified template length,
/// forcing the test to use the inaccurate behaviour of the NIST STS reference implementation.
///
/// The template length may be either 9 or 10.
///
/// ## Return values.
/// * If the argument is within the specified bounds: the new argument.
/// * Otherwise: `NULL`
#[no_mangle]
pub extern "C" fn test_arg_overlapping_template_new_nist_behaviour(
    template_length: usize,
) -> Option<Box<TestArgOverlappingTemplate>> {
    overlapping::OverlappingTemplateTestArgs::new_nist_behaviour(template_length)
        .map(|arg| Box::new(TestArgOverlappingTemplate(arg)))
}

// linear complexity test
test_arg! {
    /// The argument for the Linear Complexity Test.
    /// Allows to choose the block length manually or automatically.
    ///
    /// If the block length is chosen manually, the following equations must be true:
    /// * 500 <= block length <= 5000
    /// * total bit length / block length >= 200
    struct TestArgLinearComplexity(linear_complexity::LinearComplexityTestArg);

    /// Creates a default argument for the Linear Complexity Test, choosing the block length
    /// automatically on runtime.
    fn test_arg_linear_complexity_default() -> Self;

    /// Destroys the given argument for the Linear Complexity Test.
    fn test_arg_linear_complexity_destroy(self);
}

/// Creates a new argument for the linear Complexity Test, choosing the block length manually.
///
/// ## Return values
///
/// * If the block length is within 500 <= block_length <= 5000: the new argument.
/// * Otherwise: `NULL`
#[no_mangle]
pub extern "C" fn test_arg_linear_complexity_new(
    block_length: usize,
) -> Option<Box<TestArgLinearComplexity>> {
    if (500..=5000).contains(&block_length) {
        // non-zero was just checked
        let arg = NonZero::new(block_length)?;
        let arg = linear_complexity::LinearComplexityTestArg::ManualBlockLength(arg);
        Some(Box::new(TestArgLinearComplexity(arg)))
    } else {
        None
    }
}

// serial test
test_arg! {
    /// The argument for the serial test: the block length in bits to check.
    ///
    /// Argument constraints:
    /// 1. the given block length must be >= 2.
    /// 2. each value of with the bit length the given block length must be representable as usize,
    ///     i.e. depending on the platform, 32 or 64 bits.
    /// 3. the block length must be < (log2(bit_len) as int) - 2
    ///
    /// Constraints 1 and 2 are checked when creating the arguments.
    ///
    /// Constraint 3 is checked on executing the test. If the constraint is violated,
    /// an error will be raised.
    ///
    /// The default value for this argument is 16. For this to work, the input length must be at least
    /// 2^19 bit.
    struct TestArgSerial(serial::SerialTestArg);

    /// Creates a default argument for the Serial Test, with the block length set to the one
    /// recommended by NIST.
    fn test_arg_serial_default() -> Self;

    /// Destroys the given argument for the Serial Test.
    fn test_arg_serial_destroy(self);
}

/// Creates a new argument for the Serial Test. The block length is checked to fulfill the constraints
/// defined in the struct declaration.
///
/// ## Return value
///
/// * if the given block length satisfies the constraints: the new argument.
/// * otherwise: `NULL`
#[no_mangle]
pub extern "C" fn test_arg_serial_new(block_length: u8) -> Option<Box<TestArgSerial>> {
    serial::SerialTestArg::new(block_length)
        .map(|arg| Box::new(TestArgSerial(arg)))
}

// approximate entropy test
test_arg! {
    /// The argument for the Approximate Entropy Test: the block length in bits to check.
    ///
    /// Argument constraints:
    /// 1. the given block length must be >= 2.
    /// 2. each value of with the bit length the given block length must be representable as usize,
    ///     i.e. depending on the platform, 32 or 64 bits.
    /// 3. the block length must be < (log2(bit_len) as int) - 5
    ///
    /// Constraints 1 and 2 are checked when creating the arguments.
    ///
    /// Constraint 3 is checked on executing the test. If the constraint is violated,
    /// an error will be raised.
    ///
    /// The default value for this argument is 10. For this to work, the input length must be at least
    /// 2^16 bit.
    struct TestArgApproximateEntropy(approximate_entropy::ApproximateEntropyTestArg);

    /// Creates a default argument for the Approximate Entropy Test, with the block length set to the one
    /// recommended by NIST.
    fn test_arg_approximate_entropy_default() -> Self;

    /// Destroys the given argument for the Approximate Entropy Test.
    fn test_arg_approximate_entropy_destroy(self);
}

/// Creates a new argument for the Approximate Entropy Test. The block length is checked to fulfill
/// the constraints defined in the struct declaration.
///
/// ## Return value
///
/// * if the given block length satisfies the constraints: the new argument.
/// * otherwise: `NULL`
#[no_mangle]
pub extern "C" fn test_arg_approximate_entropy_new(
    block_length: u8,
) -> Option<Box<TestArgApproximateEntropy>> {
    approximate_entropy::ApproximateEntropyTestArg::new(block_length)
        .map(|arg| Box::new(TestArgApproximateEntropy(arg)))
}
