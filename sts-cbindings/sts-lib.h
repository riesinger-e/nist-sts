#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * The default block count to use in the Non-overlapping Template Matching Test.
 */
#define NON_OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_COUNT 8

/**
 * The default template length to use in the Non-overlapping Template Matching Test.
 */
#define NON_OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LEN 9

/**
 * The default length of each block M, in bits, for use in the Overlapping Template Matching Test.
 */
#define OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_LENGTH 1032

/**
 * The default degree of freedom K for use in the Overlapping Template Matching Test.
 */
#define OVERLAPPING_TEMPLATE_DEFAULT_FREEDOM 6

/**
 * The default template length use in the Overlapping Template Matching Test.
 */
#define OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LENGTH 9

/**
 * The default threshold for determining if a test passes its criteria.
 */
#define DEFAULT_THRESHOLD 0.01

/**
 * The error codes that are returned by some fallible functions.
 * A human-readable error message can be retrieved with [get_last_error_str].
 */
typedef enum ErrorCode {
  /**
   * No error
   */
  ErrorCode_NoError = 0,
  /**
   * A numeric overflow happened in the called test.
   */
  ErrorCode_Overflow = 1,
  /**
   * The result of a test was `NaN`
   */
  ErrorCode_NaN = 2,
  /**
   * The result of a test was (positive or negative) Infinity.
   */
  ErrorCode_Infinite = 3,
  /**
   * The gamma function used in a test failed.
   */
  ErrorCode_GammaFunctionFailed = 4,
  /**
   * A test was called with an invalid parameter (value-wise, references are not checked!).
   */
  ErrorCode_InvalidParameter = 5,
  /**
   * The function [set_max_threads] failed.
   */
  ErrorCode_SetMaxThreads = 6,
  /**
   * A test passed to the test runner is invalid (Invalid value).
   */
  ErrorCode_InvalidTest = 7,
  /**
   * A test was specified multiple times in the same call to the test runner.
   */
  ErrorCode_DuplicateTest = 8,
  /**
   * One or multiple tests that were run with the test runner failed.
   */
  ErrorCode_TestFailed = 9,
  /**
   * The test whose result was tried to be retrieved from the test runner was not run.
   */
  ErrorCode_TestWasNotRun = 10,
} ErrorCode;

/**
 * List of all tests, used for automatic running.
 */
typedef enum Test {
  /**
   * See [frequency_test](crate::tests::frequency_test).
   */
  Test_Frequency = 0,
  /**
   * See [frequency_block_test](crate::tests::frequency_block_test).
   */
  Test_FrequencyWithinABlock = 1,
  /**
   * See [runs_test](crate::tests::runs_test).
   */
  Test_Runs = 2,
  /**
   * See [longest_run_of_ones_test](crate::tests::longest_run_of_ones_test).
   */
  Test_LongestRunOfOnes = 3,
  /**
   * See [binary_matrix_rank_test](crate::tests::binary_matrix_rank_test).
   */
  Test_BinaryMatrixRank = 4,
  /**
   * See [spectral_dft_test](crate::tests::spectral_dft_test).
   */
  Test_SpectralDft = 5,
  /**
   * See [non_overlapping_template_matching_test](crate::tests::non_overlapping_template_matching_test).
   */
  Test_NonOverlappingTemplateMatching = 6,
  /**
   * See [overlapping_template_matching_test](crate::tests::overlapping_template_matching_test).
   */
  Test_OverlappingTemplateMatching = 7,
  /**
   * See [maurers_universal_statistical_test](crate::tests::maurers_universal_statistical_test).
   */
  Test_MaurersUniversalStatistical = 8,
  /**
   * See [linear_complexity_test](crate::tests::linear_complexity_test).
   */
  Test_LinearComplexity = 9,
  /**
   * See [serial_test](crate::tests::serial_test).
   */
  Test_Serial = 10,
  /**
   * See [approximate_entropy_test](crate::tests::approximate_entropy_test).
   */
  Test_ApproximateEntropy = 11,
  /**
   * See [cumulative_sums_test](crate::tests::cumulative_sums_test).
   */
  Test_CumulativeSums = 12,
  /**
   * See [random_excursions_test](crate::tests::random_excursions_test).
   */
  Test_RandomExcursions = 13,
  /**
   * See [random_excursions_variant_test](crate::tests::random_excursions_variant_test).
   */
  Test_RandomExcursionsVariant = 14,
} Test;

/**
 * BitVec: a list of bits to run statistical tests on.
 */
typedef struct BitVec BitVec;

/**
 * All test arguments for use in a *TestRunner*,
 * prefilled with sane defaults.
 *
 * To set an argument, use the appropriate `runner_test_args_set_...` function.
 */
typedef struct RunnerTestArgs RunnerTestArgs;

/**
 * The argument for the Approximate Entropy Test: the block length in bits to check.
 *
 * Argument constraints:
 * 1. the given block length must be >= 2.
 * 2. each value of with the bit length the given block length must be representable as usize,
 *     i.e. depending on the platform, 32 or 64 bits.
 * 3. the block length must be < (log2(bit_len) as int) - 5
 *
 * Constraints 1 and 2 are checked when creating the arguments.
 *
 * Constraint 3 is checked on executing the test. If the constraint is violated,
 * an error will be raised.
 *
 * The default value for this argument is 10. For this to work, the input length must be at least
 * 2^16 bit.
 */
typedef struct TestArgApproximateEntropy TestArgApproximateEntropy;

/**
 * The argument for the Frequency test within a block: the block length.
 *
 * The block length should be at least 20 bits, with the block length greater than 1% of the
 * total bit length and fewer than 100 total blocks.
 */
typedef struct TestArgFrequencyBlock TestArgFrequencyBlock;

/**
 * The argument for the Linear Complexity Test.
 * Allows to choose the block length manually or automatically.
 *
 * If the block length is chosen manually, the following equations must be true:
 * * 500 <= block length <= 5000
 * * total bit length / block length >= 200
 */
typedef struct TestArgLinearComplexity TestArgLinearComplexity;

/**
 * The arguments for the Non-overlapping Template Matching Test.
 *
 * 1. The templates length to use within a block: `m`.
 *    2 <= `m` <= 21 - recommended: 9.
 * 2. The number of independent blocks to test in the sequence: `N`
 *    1 <= `N` < 100 - recommended: 8
 *
 * These bounds are checked by all creation functions.
 *
 * You can also use [NON_OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_COUNT] and
 * [NON_OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LEN].
 */
typedef struct TestArgNonOverlappingTemplate TestArgNonOverlappingTemplate;

/**
 * The arguments for the Overlapping Template Matching Test.
 *
 * 1. The template length *m*. 2 <= *m* <= 21. See [OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LENGTH].
 * 2. The length of each block, *M*, in bits. See [OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_LENGTH].
 * 3. The degrees of freedom, *K*. See [OVERLAPPING_TEMPLATE_DEFAULT_FREEDOM].
 *
 * With these arguments the *pi* values are calculated according to Hamano and Kaneko.
 * These bounds are checked by all creation functions.
 *
 * The original NIST implementation has some glaring inaccuracies,
 * to replicate this exact NIST behaviour, use [test_arg_overlapping_template_new_nist_behaviour]
 */
typedef struct TestArgOverlappingTemplate TestArgOverlappingTemplate;

/**
 * The argument for the serial test: the block length in bits to check.
 *
 * Argument constraints:
 * 1. the given block length must be >= 2.
 * 2. each value of with the bit length the given block length must be representable as usize,
 *     i.e. depending on the platform, 32 or 64 bits.
 * 3. the block length must be < (log2(bit_len) as int) - 2
 *
 * Constraints 1 and 2 are checked when creating the arguments.
 *
 * Constraint 3 is checked on executing the test. If the constraint is violated,
 * an error will be raised.
 *
 * The default value for this argument is 16. For this to work, the input length must be at least
 * 2^19 bit.
 */
typedef struct TestArgSerial TestArgSerial;

/**
 * The result of a statistical test.
 */
typedef struct TestResult TestResult;

/**
 * This test runner can be used to run several / all tests on a sequence in one call.
 */
typedef struct TestRunner TestRunner;


#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Returns the last error that happened in the calling thread. This function works in 2 steps:
 * 1. the caller calls the function with `ptr` set to `NULL`. The necessary length is written to
 *    `len`.
 * 2. the caller calls the function with `ptr` set to a valid buffer, and `len` set to the length of
 *    the buffer. If the length is enough to store the error message, it is written to the buffer.
 *    The error message is written with a nul-terminating byte.
 *
 * ## Return values
 *
 * - >0: the [ErrorCode] of the last error. Everything worked.
 * - 0: there is no error in storage.
 * - -1: the passed string buffer is too small.
 *
 * ## Safety
 *
 * * `len` must not be `NULL`.
 * * `ptr` must be valid for writes of up to `len` bytes.
 * * `ptr` may not be mutated for the duration of this call.
 * * All responsibility for `ptr` and `len`, especially for its de-allocation, remains with the caller.
 */
int get_last_error_str(char *ptr,
                       size_t *len);

/**
 * Sets the maximum of threads to be used by the tests. These method can only be called ONCE and only
 * BEFORE any test is started. If not used, a sane default will be chosen.
 *
 * If called multiple times or after the first test, an error will be returned.
 *
 * Since this library uses [rayon](https://docs.rs/rayon/latest/rayon/index.html), this function
 * effectively calls
 * [ThreadPoolBuilder::num_threads](https://docs.rs/rayon/latest/rayon/struct.ThreadPoolBuilder.html#method.num_threads).
 * If you use rayon in the calling code, no rayon workload may have been run before calling this
 * function.
 *
 * ## Return values
 *
 * * 0: the call worked.
 * * 1: an error happened - use [get_last_error_str]
 */
int set_max_threads(size_t max_threads);

/**
 * Returns the minimum input length, in bits, for the specified test.
 *
 * ## Return values
 *
 * * >0: the call worked. Returned is minimum input length
 * * 0: an error happened - use [get_last_error_str]
 */
size_t get_min_length_for_test(Test test);

/**
 * Creates a Bit Vector from a string, with the ASCII char "0" mapping to 0 and "1" mapping to 1.
 * Any other character is ignored.
 *
 * ## Safety
 *
 * * The memory pointed to by `ptr` must contain a valid nul terminator at the end of the string.
 * * `ptr` must be valid, as defined by the Rust module safety documentation of `std::ptr`, for reads
 *   of bytes up to and including the nul terminator.
 *     * The entire memory range must be contained within a single allocated object!
 * * `ptr` must have at least length 1: the nul terminator.
 * * The memory referenced by `ptr` must not be mutated for the duration of this method call.
 * * `ptr`, particularly the de-allocation of it, remains in the responsibility of the caller.
 * * The de-allocation of the returned [BitVec] must be done via [bitvec_destroy].
 */
struct BitVec *bitvec_from_str(const char *ptr);

/**
 * Same as [bitvec_from_str], but allows to specify a maximum count of bits to read from the
 * string. When this limit is reached, the String will not be read any further.
 *
 * ## Safety
 *
 * The same safety considerations apply as for [bitvec_from_str]
 */
struct BitVec *bitvec_from_str_with_max_length(const char *ptr, size_t max_length);

/**
 * Creates a BitVec from a byte array, where each byte is filled with 8 bits.
 *
 * ## Safety
 *
 * * The memory pointed to by `ptr` must be valid for reads of up to `len` bytes.
 * * The memory referenced by `ptr` must not be mutated for the duration of this method call.
 * * `ptr`, particularly the de-allocation of it, remains in the responsibility of the caller.
 * * The de-allocation of the returned [BitVec] must be done via [bitvec_destroy].
 */
struct BitVec *bitvec_from_bytes(const uint8_t *ptr, size_t len);

/**
 * Creates a BitVec from a bool array, with each bool representing one bit.
 *
 * ## Safety
 *
 * * The memory pointed to by `ptr` must be valid for reads of up to `len` elements.
 * * The memory referenced by `ptr` must not be mutated for the duration of this method call.
 * * `ptr`, particularly the de-allocation of it, remains in the responsibility of the caller.
 * * The de-allocation of the returned [BitVec] must be done via [bitvec_destroy].
 */
struct BitVec *bitvec_from_bits(const bool *ptr, size_t len);

/**
 * Destroys a created BitVec.
 *
 * ## Safety
 *
 * * `bitvec` must have been created by either [bitvec_from_str], [bitvec_from_str_with_max_length],
 *   [bitvec_from_bytes] or [bitvec_from_bits].
 * * `bitvec` must be a valid pointer.
 * * `bitvec` may not be mutated for the duration of this call..
 */
struct BitVec *bitvec_clone(const struct BitVec *bitvec);

/**
 * Destroys a created BitVec.
 *
 * ## Safety
 *
 * * `bitvec` must have been created by either [bitvec_from_str], [bitvec_from_str_with_max_length],
 *   [bitvec_from_bytes], [bitvec_from_bits] or [bitvec_clone].
 * * `bitvec` must be a valid pointer.
 * * There must be no other references to `bitvec`.
 * * After this call, the memory referenced by `bitvec` is freed. Trying to access this memory
 *   will lead to undefined behaviour.
 */
void bitvec_destroy(struct BitVec *bitvec);

/**
 * Returns the count of bits in the BitVec.
 *
 * ## Safety
 *
 * * `bitvec` must have been created by either [bitvec_from_str], [bitvec_from_str_with_max_length],
 *   [bitvec_from_bytes], [bitvec_from_bits] or [bitvec_clone].
 * * `bitvec` must be a valid pointer.
 * * `bitvec` may not be mutated for the duration of this call.
 */
size_t bitvec_len_bit(const struct BitVec *bitvec);

/**
 * Crops the BitVec to the given count of bits. Values for `new_bit_len` that are larger than the
 * current bit length will do nothing.
 *
 * ## Safety
 *
 * * `bitvec` must have been created by either [bitvec_from_str], [bitvec_from_str_with_max_length],
 *   [bitvec_from_bytes], [bitvec_from_bits] or [bitvec_clone].
 * * `bitvec` must be a valid pointer.
 * * `bitvec` may not be mutated by other functions for the duration of this call.
 */
void bitvec_crop(struct BitVec *bitvec, size_t new_bit_len);

/**
 * Creates a default new argument for the Frequency test within a block that chooses a suitable
 * block length automatically.
 * This function never returns `NULL`.
 */
struct TestArgFrequencyBlock *test_arg_frequency_block_default(void);

/**
 * Destroys the given argument for the Frequency test within a block.
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the construction methods provided by this library.
 * * `ptr` must be valid for reads and writes and non-null.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 * * `ptr` may not be mutated for the duration of this call.
 */
void test_arg_frequency_block_destroy(struct TestArgFrequencyBlock *ptr);

/**
 * Creates a new argument for the Frequency test within a block, specifying the block length in bits.
 *
 * ## Return values
 * - if the given `block_length == 0`, `NULL` is returned.
 * - if the given `block_length != 0`, a pointer to the argument is returned.
 */
struct TestArgFrequencyBlock *test_arg_frequency_block_new(size_t block_length);

/**
 * Creates a default new non-overlapping template test argument that chooses its template length
 * and block count according to the values recommended by NIST.
 * This function never returns `NULL`.
 */
struct TestArgNonOverlappingTemplate *test_arg_non_overlapping_template_default(void);

/**
 * Destroys the given argument for Non-overlapping Template Matching Test.
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the construction methods provided by this library.
 * * `ptr` must be valid for reads and writes and non-null.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 * * `ptr` may not be mutated for the duration of this call.
 */
void test_arg_non_overlapping_template_destroy(struct TestArgNonOverlappingTemplate *ptr);

/**
 * Creates a new non-overlapping template test argument with the specified template length and block
 * count.
 *
 * ## Return values.
 * * If both arguments are within the bounds specified in [TestArgNonOverlappingTemplate]: the new argument.
 * * Otherwise: `NULL`
 */
struct TestArgNonOverlappingTemplate *test_arg_non_overlapping_template_new(size_t template_len,
                                                                            size_t count_blocks);

/**
 * Creates a new argument for the Overlapping Template Matching Test, using the default values
 * [OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LENGTH], [OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_LENGTH]
 * and [OVERLAPPING_TEMPLATE_DEFAULT_FREEDOM].
 * This function never returns `NULL`.
 */
struct TestArgOverlappingTemplate *test_arg_overlapping_template_default(void);

/**
 * Destroys the given argument for the Overlapping Template Matching Test.
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the construction methods provided by this library.
 * * `ptr` must be valid for reads and writes and non-null.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 * * `ptr` may not be mutated for the duration of this call.
 */
void test_arg_overlapping_template_destroy(struct TestArgOverlappingTemplate *ptr);

/**
 * Creates a new Overlapping Template Matching Test argument with the specified template length, block
 * count and degrees of freedom.
 *
 * ## Return values.
 * * If all arguments are within the bounds specified in [TestArgOverlappingTemplate]: the new argument.
 * * Otherwise: `NULL`
 */
struct TestArgOverlappingTemplate *test_arg_overlapping_template_new(size_t template_length,
                                                                     size_t block_length,
                                                                     size_t freedom);

/**
 * Creates a new Overlapping Template Matching Test argument with the specified template length,
 * forcing the test to use the inaccurate behaviour of the NIST STS reference implementation.
 *
 * The template length may be either 9 or 10.
 *
 * ## Return values.
 * * If the argument is within the specified bounds: the new argument.
 * * Otherwise: `NULL`
 */
struct TestArgOverlappingTemplate *test_arg_overlapping_template_new_nist_behaviour(size_t template_length);

/**
 * Creates a default argument for the Linear Complexity Test, choosing the block length
 * automatically on runtime.
 * This function never returns `NULL`.
 */
struct TestArgLinearComplexity *test_arg_linear_complexity_default(void);

/**
 * Destroys the given argument for the Linear Complexity Test.
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the construction methods provided by this library.
 * * `ptr` must be valid for reads and writes and non-null.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 * * `ptr` may not be mutated for the duration of this call.
 */
void test_arg_linear_complexity_destroy(struct TestArgLinearComplexity *ptr);

/**
 * Creates a new argument for the linear Complexity Test, choosing the block length manually.
 *
 * ## Return values
 *
 * * If the block length is within 500 <= block_length <= 5000: the new argument.
 * * Otherwise: `NULL`
 */
struct TestArgLinearComplexity *test_arg_linear_complexity_new(size_t block_length);

/**
 * Creates a default argument for the Serial Test, with the block length set to the one
 * recommended by NIST.
 * This function never returns `NULL`.
 */
struct TestArgSerial *test_arg_serial_default(void);

/**
 * Destroys the given argument for the Serial Test.
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the construction methods provided by this library.
 * * `ptr` must be valid for reads and writes and non-null.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 * * `ptr` may not be mutated for the duration of this call.
 */
void test_arg_serial_destroy(struct TestArgSerial *ptr);

/**
 * Creates a new argument for the Serial Test. The block length is checked to fulfill the constraints
 * defined in the struct declaration.
 *
 * ## Return value
 *
 * * if the given block length satisfies the constraints: the new argument.
 * * otherwise: `NULL`
 */
struct TestArgSerial *test_arg_serial_new(uint8_t block_length);

/**
 * Creates a default argument for the Approximate Entropy Test, with the block length set to the one
 * recommended by NIST.
 * This function never returns `NULL`.
 */
struct TestArgApproximateEntropy *test_arg_approximate_entropy_default(void);

/**
 * Destroys the given argument for the Approximate Entropy Test.
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the construction methods provided by this library.
 * * `ptr` must be valid for reads and writes and non-null.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 * * `ptr` may not be mutated for the duration of this call.
 */
void test_arg_approximate_entropy_destroy(struct TestArgApproximateEntropy *ptr);

/**
 * Creates a new argument for the Approximate Entropy Test. The block length is checked to fulfill
 * the constraints defined in the struct declaration.
 *
 * ## Return value
 *
 * * if the given block length satisfies the constraints: the new argument.
 * * otherwise: `NULL`
 */
struct TestArgApproximateEntropy *test_arg_approximate_entropy_new(uint8_t block_length);

/**
 * Destroys the given test result. If you want to destroy a whole list, use [test_result_list_destroy].
 * You cannot destroy only a part of a list with this function.
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the tests and must have been returned as a single pointer.
 * * `ptr` must be a valid, non-null element.
 * * `ptr` must not be mutated for the duration of this call.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 */
void test_result_destroy(struct TestResult *ptr);

/**
 * Destroys the given list of test results. If you want to destroy only a single test result,
 * use [test_result_destroy].
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the tests or by the test runner, and must have been
 *   returned by the creating function as a list.
 * * `ptr` must be valid allocation with `count` elements.
 * * `ptr` must not be mutated for the duration of this call.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 */
void test_result_list_destroy(struct TestResult **ptr, size_t count);

/**
 * Returns the p_value of the test result.
 *
 * ## Safety
 *
 * * `result` must have been created by one of the tests.
 * * `result` must be a valid pointer.
 * * `result` may not be mutated for the duration of this call.
 */
double test_result_get_p_value(const struct TestResult *result);

/**
 * Checks if the contained p_value passed the given threshold (i.e. if test passed).
 *
 * ## Safety
 *
 * * `result` must have been created by one of the tests.
 * * `result` must be a valid pointer.
 * * `result` may not be mutated for the duration of this call.
 */
bool test_result_passed(const struct TestResult *result, double threshold);

/**
 * Extracts the (maybe existing) comment contained in the test result.
 * This function works in 2 steps:
 * 1. the caller calls the function with `ptr` set to `NULL`. The necessary length is written to
 *    `len`.
 * 2. the caller calls the function with `ptr` set to a valid buffer, and `len` set to the length of
 *    the buffer. If the length is enough to store the error message, it is written to the buffer.
 *    The error message is written with a nul-terminating byte.
 *
 * # Return values
 *
 * - 0: everything's OK.
 * - 1: there is no comment to store.
 * - 2: the passed string buffer is too small.
 *
 * ## Safety
 *
 * * `result` must have been created by one of the tests.
 * * `result` must be a valid pointer.
 * * `result` may not be mutated for the duration of this call.
 * * `len` must not be `NULL`.
 * * `ptr` must be valid for writes of up to `len` bytes.
 * * `ptr` may not be mutated for the duration of this call.
 * * All responsibility for `ptr` and `len`, especially for its de-allocation, remains with the caller.
 */
int test_result_get_comment(const struct TestResult *result,
                            char *ptr,
                            size_t *len);

/**
 * Creates a new test runner. This test runner can be used to run multiple tests on 1 sequence in
 * 1 function call.
 *
 * The result pointer must be freed with [test_runner_destroy].
 */
struct TestRunner *test_runner_new(void);

/**
 * Destroys the given test runner.
 *
 * ## Safety
 *
 * * `runner` must have been created by [test_runner_new()]
 * * `runner` must be valid for reads and writes and non-null.
 * * `runner` may not be mutated for the duration of this call.
 * * `runner` will be an invalid pointer after this call, trying to access its memory will lead to
 *   undefined behaviour.
 */
void test_runner_destroy(struct TestRunner *runner);

/**
 * Returns the result of the given test, if it was run. Since some tests return multiple results,
 * the returned pointer is an array, the count of elements will be stored into `length`.
 *
 * After this call, the result is no longer stored inside the runner.
 *
 * The resulting list of test results must be destroyed with
 * [test_result_list_destroy](crate::test_result::test_result_list_destroy).
 *
 * ## Safety
 *
 * * `runner` must have been created by [test_runner_new()]
 * * `runner` must be valid for reads and writes and non-null.
 * * `runner` may not be mutated for the duration of this call.
 * * `length` must be a non-null pointer valid for writes.
 * * `length` may not be mutated for the duration of this call.
 */
struct TestResult **test_runner_get_result(struct TestRunner *runner, Test test, size_t *length);

/**
 * Runs all tests on the given bit sequence with the default test arguments.
 *
 * ## Return value
 *
 * * If all tests ran successfully, `0` is returned.
 * * If an error occurred when running one test, but without aborting the tests, `2` is returned.
 *   The good test results can be retrieved with [test_runner_get_result], the exact error can
 *   be retrieved with [get_last_error_str](crate::get_last_error_str).
 *
 * ## Safety
 *
 * * `runner` must have been created by [test_runner_new()]
 * * `runner` must be valid for reads and writes and non-null.
 * * `runner` may not be mutated for the duration of this call.
 * * `bitvec` must have been created by either [bitvec_from_str](crate::bitvec::bitvec_from_str),
 *   [bitvec_from_str_with_max_length](crate::bitvec::bitvec_from_str_with_max_length),
 *   [bitvec_from_bytes](crate::bitvec::bitvec_from_bytes),
 *   [bitvec_from_bits](crate::bitvec::bitvec_from_bits) or
 *   [bitvec_clone](crate::bitvec::bitvec_clone).
 * * `bitvec` must be a non-null pointer valid for reads.
 * * `bitvec` may not be mutated for the duration of this call.
 */
int test_runner_run_all_automatic(struct TestRunner *runner, const struct BitVec *data);

/**
 * Runs all chosen tests on the given bit sequence with the default test arguments.
 *
 * ## Return value
 *
 * * If all tests ran successfully, `0` is returned.
 * * If one of the tests specified was a duplicate of a previous test, `1` is returned.
 * * If one of the tests specified was not a valid test as per the enum [Test], `1` is returned.
 * * If an error occurred while running the tests, `2` is returned. All other tests are still done.
 *   The good test results can be retrieved with [test_runner_get_result], the exact error can
 *   be retrieved.
 *
 * In each error case, the error message and code can be found out with
 * [get_last_error_str](crate::get_last_error_str).
 *
 * ## Safety
 *
 * * `runner` must have been created by [test_runner_new()]
 * * `runner` must be valid for reads and writes and non-null.
 * * `runner` may not be mutated for the duration of this call.
 * * `bitvec` must have been created by either [bitvec_from_str](crate::bitvec::bitvec_from_str),
 *   [bitvec_from_str_with_max_length](crate::bitvec::bitvec_from_str_with_max_length),
 *   [bitvec_from_bytes](crate::bitvec::bitvec_from_bytes),
 *   [bitvec_from_bits](crate::bitvec::bitvec_from_bits) or
 *   [bitvec_clone](crate::bitvec::bitvec_clone).
 * * `bitvec` must be a non-null pointer valid for reads.
 * * `bitvec` may not be mutated for the duration of this call.
 * * `tests` must be a valid, non-null pointer readable for up to `tests_len` elements.
 * * `tests` may not be mutated for the duration of this call.
 */
int test_runner_run_automatic(struct TestRunner *runner,
                              const struct BitVec *data,
                              const Test *tests,
                              size_t tests_len);

/**
 * Runs all tests on the given bit sequence with the given test arguments.
 *
 * ## Return value
 *
 * * If all tests ran successfully, `0` is returned.
 * * If an error occurred while running the tests, `2` is returned. All other tests are still done.
 *   The good test results can be retrieved with [test_runner_get_result], the exact error can
 *   be retrieved.
 *
 * ## Safety
 *
 * * `runner` must have been created by [test_runner_new()]
 * * `runner` must be valid for reads and writes and non-null.
 * * `runner` may not be mutated for the duration of this call.
 * * `bitvec` must have been created by either [bitvec_from_str](crate::bitvec::bitvec_from_str),
 *   [bitvec_from_str_with_max_length](crate::bitvec::bitvec_from_str_with_max_length),
 *   [bitvec_from_bytes](crate::bitvec::bitvec_from_bytes),
 *   [bitvec_from_bits](crate::bitvec::bitvec_from_bits) or
 *   [bitvec_clone](crate::bitvec::bitvec_clone).
 * * `bitvec` must be a non-null pointer valid for reads.
 * * `bitvec` may not be mutated for the duration of this call.
 * * `test_args` must have been created by [runner_test_args_new](test_args::runner_test_args_new).
 * * `test_args` must be a non-null pointer valid for reads.
 */
int test_runner_run_all_tests(struct TestRunner *runner,
                              const struct BitVec *data,
                              const struct RunnerTestArgs *test_args);

/**
 * Runs all chosen tests on the given bit sequence with the given test arguments.
 *
 * ## Return value
 *
 * * If all tests ran successfully, `0` is returned.
 * * If one of the tests specified was a duplicate of a previous test, `1` is returned.
 * * If one of the tests specified was not a valid test as per the enum [Test], `1` is returned.
 * * If an error occurred while running the tests, `2` is returned. All other tests are still done.
 *   The good test results can be retrieved with [test_runner_get_result], the exact error can
 *   be retrieved.
 *
 * In each error case, the error message and code can be found out with
 * [get_last_error_str](crate::get_last_error_str).
 *
 * ## Safety
 *
 * * `runner` must have been created by [test_runner_new()]
 * * `runner` must be valid for reads and writes and non-null.
 * * `runner` may not be mutated for the duration of this call.
 * * `bitvec` must have been created by either [bitvec_from_str](crate::bitvec::bitvec_from_str),
 *   [bitvec_from_str_with_max_length](crate::bitvec::bitvec_from_str_with_max_length),
 *   [bitvec_from_bytes](crate::bitvec::bitvec_from_bytes),
 *   [bitvec_from_bits](crate::bitvec::bitvec_from_bits) or
 *   [bitvec_clone](crate::bitvec::bitvec_clone).
 * * `bitvec` must be a non-null pointer valid for reads.
 * * `bitvec` may not be mutated for the duration of this call.
 * * `tests` must be a valid, non-null pointer readable for up to `tests_len` elements.
 * * `tests` may not be mutated for the duration of this call.
 * * `test_args` must have been created by [runner_test_args_new](test_args::runner_test_args_new).
 * * `test_args` must be a non-null pointer valid for reads.
 */
int test_runner_run_tests(struct TestRunner *runner,
                          const struct BitVec *data,
                          const Test *tests,
                          size_t tests_len,
                          const struct RunnerTestArgs *test_args);

/**
 * Create new [RunnerTestArgs], prefilled with sane defaults.
 *
 * To set an argument, use the appropriate `runner_test_args_set_...` function.
 *
 * The resulting pointer must be freed via [runner_test_args_destroy].
 */
struct RunnerTestArgs *runner_test_args_new(void);

/**
 * Destroy the given [RunnerTestArgs].
 *
 * ## Safety
 *
 * * `args` must have been created by [runner_test_args_new()]
 * * `args` must be valid for reads and writes and non-null.
 * * `args` may not be mutated for the duration of this call.
 * * `args` will be an invalid pointer after this call, trying to access its memory will lead to
 *   undefined behaviour.
 */
void runner_test_args_destroy(struct RunnerTestArgs *args);

/**
 * Set the argument for the Frequency Block Test to the given value.
 *
 * ## Safety
 *
 * * `args` must have been created by [runner_test_args_new()]
 * * `args` must be valid for reads and writes and non-null.
 * * `args` may not be mutated for the duration of this call.
 * * `arg` must have been created by one of the construction methods provided by this library.
 * * `arg` must be valid for reads and non-null.
 * * `arg` may not be mutated for the duration of this call.
 * * All responsibility for `arg`, particularly its deallocation, remains with the caller.
 *   This function copies the content of `arg`.
 */
void runner_test_args_set_frequency_block(struct RunnerTestArgs *args,
                                          const struct TestArgFrequencyBlock *arg);

/**
 * Set the argument for the Non-Overlapping Template Matching Test to the given value.
 *
 * ## Safety
 *
 * * `args` must have been created by [runner_test_args_new()]
 * * `args` must be valid for reads and writes and non-null.
 * * `args` may not be mutated for the duration of this call.
 * * `arg` must have been created by one of the construction methods provided by this library.
 * * `arg` must be valid for reads and non-null.
 * * `arg` may not be mutated for the duration of this call.
 * * All responsibility for `arg`, particularly its deallocation, remains with the caller.
 *   This function copies the content of `arg`.
 */
void runner_test_args_set_non_overlapping_template(struct RunnerTestArgs *args,
                                                   const struct TestArgNonOverlappingTemplate *arg);

/**
 * Set the argument for the Overlapping Template Matching Test to the given value.
 *
 * ## Safety
 *
 * * `args` must have been created by [runner_test_args_new()]
 * * `args` must be valid for reads and writes and non-null.
 * * `args` may not be mutated for the duration of this call.
 * * `arg` must have been created by one of the construction methods provided by this library.
 * * `arg` must be valid for reads and non-null.
 * * `arg` may not be mutated for the duration of this call.
 * * All responsibility for `arg`, particularly its deallocation, remains with the caller.
 *   This function copies the content of `arg`.
 */
void runner_test_args_set_overlapping_template(struct RunnerTestArgs *args,
                                               const struct TestArgOverlappingTemplate *arg);

/**
 * Set the argument for the Linear Complexity Test to the given value.
 *
 * ## Safety
 *
 * * `args` must have been created by [runner_test_args_new()]
 * * `args` must be valid for reads and writes and non-null.
 * * `args` may not be mutated for the duration of this call.
 * * `arg` must have been created by one of the construction methods provided by this library.
 * * `arg` must be valid for reads and non-null.
 * * `arg` may not be mutated for the duration of this call.
 * * All responsibility for `arg`, particularly its deallocation, remains with the caller.
 *   This function copies the content of `arg`.
 */
void runner_test_args_set_linear_complexity(struct RunnerTestArgs *args,
                                            const struct TestArgLinearComplexity *arg);

/**
 * Set the argument for the Serial Test to the given value.
 *
 * ## Safety
 *
 * * `args` must have been created by [runner_test_args_new()]
 * * `args` must be valid for reads and writes and non-null.
 * * `args` may not be mutated for the duration of this call.
 * * `arg` must have been created by one of the construction methods provided by this library.
 * * `arg` must be valid for reads and non-null.
 * * `arg` may not be mutated for the duration of this call.
 * * All responsibility for `arg`, particularly its deallocation, remains with the caller.
 *   This function copies the content of `arg`.
 */
void runner_test_args_set_serial(struct RunnerTestArgs *args, const struct TestArgSerial *arg);

/**
 * Set the argument for the Approximate Entropy Test to the given value.
 *
 * ## Safety
 *
 * * `args` must have been created by [runner_test_args_new()]
 * * `args` must be valid for reads and writes and non-null.
 * * `args` may not be mutated for the duration of this call.
 * * `arg` must have been created by one of the construction methods provided by this library.
 * * `arg` must be valid for reads and non-null.
 * * `arg` may not be mutated for the duration of this call.
 * * All responsibility for `arg`, particularly its deallocation, remains with the caller.
 *   This function copies the content of `arg`.
 */
void runner_test_args_set_approximate_entropy(struct RunnerTestArgs *args,
                                              const struct TestArgApproximateEntropy *arg);

/**
 * Frequency (mono bit) test - No. 1
 *
 * This test focuses on the numbers of ones and zeros in the sequence - the proportion should
 * be roughly 50:50.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult *frequency_test(const struct BitVec *data);

/**
 * Frequency Test within a block - No. 2
 *
 * This tests for the same property as [frequency_test], but within M-bit blocks.
 * It is recommended that each block has a length of at least 100 bits.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * `test_arg` must have been created by one of the construction methods provided by this library.
 * * `test_arg` must be valid for reads and non-null.
 * * `test_arg` may not be mutated for the duration of this call.
 * * All responsibility for `data` and `test_arg`, particularly for their destruction, remains with the caller.
 */
struct TestResult *frequency_block_test(const struct BitVec *data,
                                        const struct TestArgFrequencyBlock *test_arg);

/**
 * Runs test - No. 3
 *
 * This tests focuses on the number of runs in the sequence. A run is an uninterrupted sequence of
 * identical bits.
 * Each tested [BitVec] should have at least 100 bits length.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult *runs_test(const struct BitVec *data);

/**
 * Test for the Longest Run of Ones in a Block - No. 4
 *
 * This test determines whether the longest run (See [runs_test]) of ones
 * in a block is consistent with the expected value for a random sequence.
 *
 * An irregularity in the length of longest run of ones also implies an irregularity in the length
 * of the longest runs of zeroes, meaning that only this test is necessary. See the NIST publication.
 *
 * The data has to be at least 128 bits in length.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult *longest_run_of_ones_test(const struct BitVec *data);

/**
 * Binary Matrix Rank Test -  No. 5
 *
 * This test checks for linear dependence among fixed length substrings of the sequence.
 * These substrings are interpreted as matrices of size 32x32.
 *
 * The sequence must consist of at least 38 912 bits = 4864 bytes.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult *binary_matrix_rank_test(const struct BitVec *data);

/**
 * The Spectral Discrete Fourier Transform test - No. 6
 *
 * This test focuses on the peak heights in the DFT of the input sequence. This is used to detect
 * periodic features that indicate a deviation from a random sequence.
 *
 * It is recommended (but not required) for the input to be of at least 1000 bits.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult *spectral_dft_test(const struct BitVec *data);

/**
 * Non-overlapping Template Matching test - No. 7
 *
 * This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
 * This test uses an m-bit window to search for an m-bit pattern.
 *
 * This test allows for parameters, see [TestArgNonOverlappingTemplate].
 *
 * ## Return value
 *
 * If the test ran without errors, a list of `TestResult` is returned. This list can be deallocated with `test_result_list_destroy`.
 * The length of the returned list will be stored into `length`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * `test_arg` must have been created by one of the construction methods provided by this library.
 * * `test_arg` must be valid for reads and non-null.
 * * `test_arg` may not be mutated for the duration of this call.
 * * `length` must be valid for writes and non-null.
 * * `length` may not be mutated for the duration of this call.
 * * All responsibility for `data`, `test_arg` and `length`, particularly for their destruction, remains with the caller.
 */
struct TestResult **non_overlapping_template_matching_test(const struct BitVec *data,
                                                           const struct TestArgNonOverlappingTemplate *test_arg,
                                                           size_t *length);

/**
 * Overlapping Template Matching test - No. 8
 *
 * This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
 * This test uses an m-bit window to search for an m-bit pattern.
 * The big difference to the [non_overlapping_template_matching_test] test is that template matches
 * may overlap.
 *
 * The default arguments for this test derivate significantly from the NIST reference implementation,
 * since the NIST reference implementation for this test is known bad.
 * The problem is that the PI values from NIST are wrong - the correction from Hamano and Kaneko is used.
 *
 * Details about the problems:
 * * Even though the pi values should be revised according to the paper, both the example and
 *   the implementation still use the old, inaccurate calculation.
 * * The (not working) fixed values according to Hamano and Kaneko only work for very specific cases.
 * * The value *K*, as given in the paper, ist just wrong. You don't need a statistics degree to see
 *   that it is 6 and not 5.
 *
 * This test needs arguments, see [TestArgOverlappingTemplate].
 *
 * This test enforces that the input length must be >= 10^6 bits. Smaller values will lead to
 * an error!
 *
 * # About performance
 *
 * This test is quite slow in debug mode when using the more precise pi values (non-NIST behaviour),
 * taking several seconds - it runs good when using release mode.
 * For better performance, values that are calculated once are cached.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * `test_arg` must have been created by one of the construction methods provided by this library.
 * * `test_arg` must be valid for reads and non-null.
 * * `test_arg` may not be mutated for the duration of this call.
 * * All responsibility for `data` and `test_arg`, particularly for their destruction, remains with the caller.
 */
struct TestResult *overlapping_template_matching_test(const struct BitVec *data,
                                                      const struct TestArgOverlappingTemplate *test_arg);

/**
 * Maurer's "Universal Statistical" Test - No. 9
 *
 * This test detects if the given sequence if significantly compressible without information loss.
 * If it is, it is considered non-random.
 *
 * The recommended minimum length of the sequence is 387 840 bits. The absolute minimum length to
 * be used is 2020 bits, smaller inputs will raise an error.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult *maurers_universal_statistical_test(const struct BitVec *data);

/**
 * The linear complexity test - No. 10
 *
 * This test determines the randomness of a sequence by calculating the minimum length of a linear
 * feedback shift register that can create the sequence. Random sequences need longer LSFRs.
 *
 * This test needs a parameter, [TestArgLinearComplexity]. Additionally, the input sequence
 * must have a minimum length of 10^6 bits. Smaller lengths will raise an error.
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * `test_arg` must have been created by one of the construction methods provided by this library.
 * * `test_arg` must be valid for reads and non-null.
 * * `test_arg` may not be mutated for the duration of this call.
 * * All responsibility for `data` and `test_arg`, particularly for their destruction, remains with the caller.
 */
struct TestResult *linear_complexity_test(const struct BitVec *data,
                                          const struct TestArgLinearComplexity *test_arg);

/**
 * The serial test - No. 11
 *
 * This test checks the frequency of all 2^m overlapping m-bit patterns in the sequence. Random
 * sequences should be uniform. For *m = 1*, this would be the same as the
 * [Frequency Test](frequency_test).
 *
 * This test needs a parameter [TestArgSerial]. Check the described constraints there.
 *
 * The paper describes the test slightly wrong: in 2.11.5 step 5, the second argument need to be
 * halved in both *igamc* calculations. Only then are the calculated P-values equal to the P-values
 * described in 2.11.6 and the reference implementation.
 *
 * The input length should be at least 2^19 bit, although this is not enforced. If the default
 * value for [TestArgSerial] is used, a smaller input length will lead to an Error because
 * of constraint no. 3!
 *
 * If the combination of the given data ([BitVec]) and [TestArgSerial] is invalid,
 * an error is raised. For the exact constraints, see [TestArgSerial].
 *
 * ## Return value
 *
 * If the test ran without errors, a list of `TestResult` is returned. This list can be deallocated with `test_result_list_destroy`.
 *The returned array always has length 2.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * `test_arg` must have been created by one of the construction methods provided by this library.
 * * `test_arg` must be valid for reads and non-null.
 * * `test_arg` may not be mutated for the duration of this call.
 * * All responsibility for `data` and `test_arg`, particularly for their destruction, remains with the caller.
 */
struct TestResult **serial_test(const struct BitVec *data,
                                const struct TestArgSerial *test_arg);

/**
 * The approximate entropy test - No. 12
 *
 * This test is similar to the [serial test](serial_test). It compares the frequency
 * of overlapping blocks with the two block lengths *m* and *m + 1* against the expected result
 * of a random sequence.
 *
 * This test needs a parameter [TestArgApproximateEntropy]. Check the described constraints there.
 *
 * The input length should be at least 2^16 bit, although this is not enforced. If the default
 * value for [TestArgApproximateEntropy] is used, a smaller input length will lead to an Error because
 * of constraint no. 3!
 *
 * If the combination of the given data ([BitVec]) and [TestArgApproximateEntropy] is invalid,
 * an error is raised. For the exact constraints, see [TestArgApproximateEntropy].
 *
 * ## Return value
 *
 * If the test ran without errors, a single `TestResult` is returned. This result can be deallocated with `test_result_destroy`.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * `test_arg` must have been created by one of the construction methods provided by this library.
 * * `test_arg` must be valid for reads and non-null.
 * * `test_arg` may not be mutated for the duration of this call.
 * * All responsibility for `data` and `test_arg`, particularly for their destruction, remains with the caller.
 */
struct TestResult *approximate_entropy_test(const struct BitVec *data,
                                            const struct TestArgApproximateEntropy *test_arg);

/**
 * The cumulative sums test - No. 13
 *
 * This test calculates cumulative partial sums of the bit sequence, once starting from the
 * first bit and once starting from the last bit, adjusting the digits to -1 and +1 and calculating
 * the maximum absolute partial sum. The test checks if this maximum is within the expected bounds
 * for random sequences.
 *
 * The input sequence should be at least 100 bits in length, smaller sequences will raise
 * an error.
 *
 * ## Return value
 *
 * If the test ran without errors, a list of `TestResult` is returned. This result can be deallocated with `test_result_list_destroy`.
 *The returned array always has length 2.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult **cumulative_sums_test(const struct BitVec *data);

/**
 * The random excursions test - No. 14.
 *
 * This test, similarly to the [cumulative sums test](cumulative_sums_test), calculates
 * cumulative sums of a digit-adjusted (-1, +1) bit sequence, but only from the beginning to the end.
 * This test checks if the frequency of cumulative sums values per cycle is as expected for
 * a random sequence. A cycle consists of all cumulative sums between 2 "0"-values.
 *
 * Since the test needs at least 500 cycles to occur, bit sequences with fewer cycles will lead to an
 * `Ok()` result, but with the values filled with "0.0".
 *
 * If the computation finishes successfully, 8 [TestResult] are returned: one for each tested state,
 * `x`. The results will contain a comment about the state they are calculated from (e.g. "x = 3"),
 * the order is: `[-4, -3, -2, -1, +1, +2, +3, +4]`.
 *
 * The input length must be at least 10^6 bits, otherwise, an error is raised.
 *
 * ## Return value
 *
 * If the test ran without errors, a list of `TestResult` is returned. This result can be deallocated with `test_result_list_destroy`.
 *The returned array always has length 8.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult **random_excursions_test(const struct BitVec *data);

/**
 * The random excursions variant test.
 *
 * This test is quite similar to the [random excursions test](random_excursions_test),
 * with the key difference being that the frequencies are calculated over all cycles, instead of per
 * cycle.
 *
 * This test does not require a minimum number of cycles.
 *
 * If the computation finishes successfully, 18 [TestResult] are returned: one for each tested state,
 * `x`. The results will contain a comment about the state they are calculated from (e.g. "x = 3"),
 * the order is: `[-9, -8, -7, -6, -5, -4, -3, -2, -1, +1, +2, +3, +4, +5, +6, +7, +8, +9]`.
 *
 * The input length must be at least 10^6 bits, otherwise, an error is returned.
 *
 * ## Return value
 *
 * If the test ran without errors, a list of `TestResult` is returned. This result can be deallocated with `test_result_list_destroy`.
 *The returned array always has length 18.
 * If an error occurred, `NULL` is returned, and the error code and message can be retrieved with `get_last_error_str`.
 *
 * ## Safety
 *
 * * `data` must have been created by one of the construction methods provided by this library.
 * * `data` must be valid for reads and non-null.
 * * `data` may not be mutated for the duration of this call.
 * * All responsibility for `data`, particularly for its destruction, remains with the caller.
 */
struct TestResult **random_excursions_variant_test(const struct BitVec *data);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
