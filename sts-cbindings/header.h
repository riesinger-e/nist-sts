#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * The default threshold for determining if a test passes its criteria.
 */
#define DEFAULT_THRESHOLD 0.01

/**
 * BitVec: a list of bits to run statistical tests on.
 */
typedef struct BitVec BitVec;

/**
 * The result of a statistical test.
 */
typedef struct TestResult TestResult;

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
 * # Return values
 *
 * - 0: everything's OK.
 * - 1: there is no error to store.
 * - 2: the passed string buffer is too small.
 *
 * # Safety
 *
 * * `len` must not be `NULL`.
 * * `ptr` must be valid for writes of up to `len` bytes.
 * * `ptr` may not be mutated for the duration of this call.
 * * All responsibility for `ptr` and `len`, especially for its de-allocation, remains with the caller.
 */
int get_last_error_str(char *ptr,
                       size_t *len);

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
 *   [bitvec_from_bytes] or [bitvec_from_bits].
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
 *   [bitvec_from_bytes] or [bitvec_from_bits].
 * * `bitvec` must be a valid pointer.
 * * `bitvec` may not be mutated by other functions for the duration of this call.
 */
void bitvec_crop(struct BitVec *bitvec, size_t new_bit_len);

const struct TestResult *frequency_test(const struct BitVec *data);

const struct TestResult *frequency_block_test(const struct BitVec *data,
                                              const FrequencyBlockTestArg *test_arg);

const struct TestResult *runs_test(const struct BitVec *data);

const struct TestResult *longest_run_of_ones_test(const struct BitVec *data);

const struct TestResult *binary_matrix_rank_test(const struct BitVec *data);

const struct TestResult *spectral_dft_test(const struct BitVec *data);

const struct TestResult *const *non_overlapping_template_matching_test(const struct BitVec *data,
                                                                       const NonOverlappingTemplateTestArgs *test_arg,
                                                                       size_t *length);

const struct TestResult *overlapping_template_matching_test(const struct BitVec *data,
                                                            const OverlappingTemplateTestArgs *test_arg);

const struct TestResult *maurers_universal_statistical_test(const struct BitVec *data);

const struct TestResult *linear_complexity_test(const struct BitVec *data,
                                                const LinearComplexityTestArg *test_arg);

const struct TestResult *const *serial_test(const struct BitVec *data,
                                            const SerialTestArg *test_arg);

const struct TestResult *approximate_entropy_test(const struct BitVec *data,
                                                  const ApproximateEntropyTestArg *test_arg);

const struct TestResult *const *cumulative_sums_test(const struct BitVec *data);

const struct TestResult *const *random_excursions_test(const struct BitVec *data);

const struct TestResult *const *random_excursions_variant_test(const struct BitVec *data);

/**
 * Destroys the given test results.
 *
 * ## Safety
 *
 * * `ptr` must have been created by one of the tests.
 * * `ptr` must be a valid array with `count` elements.
 * * `ptr` will be invalid after this call, access will lead to undefined behaviour.
 */
void test_results_destroy(struct TestResult *ptr, size_t count);

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

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
