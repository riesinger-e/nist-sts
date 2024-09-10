#include <stdio.h>
#include "lib/sts-lib.h"

/**
* Prints the last error on the current thread that was produced by the sts library.
*/
void print_last_error(void) {
    size_t length = 0;
    int error_code = get_last_error(NULL, &length);
    if (error_code == 0 ) {
        printf("No error!\n");
    }

    char* buffer = malloc(sizeof(char) * length);
    error_code = get_last_error(buffer, &length);

    printf("Error (Code %d): %s\n", error_code, buffer);
    free(buffer);
}


int main(int argc, char **argv) {
    if (argc != 3) {
        printf("Usage: %s <filename> <bit_count>\n", argv[0]);
        return 1;
    }

    // read the file
    FILE* input = fopen(argv[1], "rb");

    if (input == NULL) {
        printf("Error opening input file\n");
        return 1;
    }

    size_t bit_size = atoi(argv[2]);
    // byte_size: if bytes are left over, 1 additional byte needs to be read
    size_t byte_size = bit_size / 8 + (bit_size % 8 == 0 ? 0 : 1);
    uint8_t *input_data = malloc(sizeof(uint8_t) * byte_size);
    if (input_data == NULL) {
        printf("Not enough memory\n");
        return 1;
    }

    size_t read_bytes = fread(input_data, sizeof(uint8_t), byte_size, input);
    if (read_bytes != byte_size) {
      printf("Failed to read input file, read only %zu bytes\n", read_bytes);
      return 1;
    }

    // create a BitVec from the buffer
    BitVec *data = bitvec_from_bytes(input_data, byte_size);
    bitvec_crop(data, bit_size);
    free(input_data);
    fclose(input);

    if (data == NULL) {
        print_last_error();
        return 1;
    }

    // Create all test args manually (mainly to show the usage)
    RunnerTestArgs *test_args = runner_test_args_new();

    TestArgFrequencyBlock *test_arg_frequency_block = test_arg_frequency_block_new(128);
    if (test_arg_frequency_block == NULL) {
        print_last_error();
        return 1;
    }
    runner_test_args_set_frequency_block(test_args, test_arg_frequency_block);
    free(test_arg_frequency_block);

    TestArgNonOverlappingTemplate *test_arg_non_overlapping_template = test_arg_non_overlapping_template_new(9, 8);
    if (test_arg_non_overlapping_template == NULL) {
        print_last_error();
        return 1;
    }
    runner_test_args_set_non_overlapping_template(test_args, test_arg_non_overlapping_template);
    free(test_arg_non_overlapping_template);

    TestArgOverlappingTemplate *test_arg_overlapping_template = test_arg_overlapping_template_new_nist_behaviour(9);
    if (test_arg_overlapping_template == NULL) {
        print_last_error();
        return 1;
    }
    runner_test_args_set_overlapping_template(test_args, test_arg_overlapping_template);
    free(test_arg_overlapping_template);

    TestArgLinearComplexity *test_arg_linear_complexity = test_arg_linear_complexity_new(500);
    if (test_arg_linear_complexity == NULL) {
        print_last_error();
        return 1;
    }
    runner_test_args_set_linear_complexity(test_args, test_arg_linear_complexity);
    free(test_arg_linear_complexity);

    TestArgSerial *test_arg_serial = test_arg_serial_new(16);
    if (test_arg_serial == NULL) {
        print_last_error();
        return 1;
    }
    runner_test_args_set_serial(test_args, test_arg_serial);
    free(test_arg_serial);

    TestArgApproximateEntropy *test_arg_approximate_entropy = test_arg_approximate_entropy_new(10);
    if (test_arg_approximate_entropy == NULL) {
        print_last_error();
        return 1;
    }
    runner_test_args_set_approximate_entropy(test_args, test_arg_approximate_entropy);
    free(test_arg_approximate_entropy);

    // Create a test runner and run all tests.
    TestRunner *runner = test_runner_new();
    if (test_runner_run_all_tests(runner, data, test_args) == 2) {
        print_last_error();
        // no return - no hard error
    }

    // Print the test results for each test.
    for (int i = 0; i < TEST_COUNT; i++) {
        size_t length = 0;
        TestResult **results = test_runner_get_result(runner, i, &length);

        for (int j = 0; j < length; j++) {
            printf("Test: %d: TestResult %d: P-Value: %lf", i + 1, j, test_result_get_p_value(results[j]));

            size_t comment_length = 0;
            if (test_result_get_comment(results[j], NULL, &comment_length) == 0) {
                char* comment = malloc(sizeof(char) * comment_length);
                test_result_get_comment(results[j], comment, &comment_length);
                printf("; Comment: %s\n", comment);
                free(comment);
            } else {
                printf("\n");
            }
        }

        test_result_list_destroy(results, length);
    }

    test_runner_destroy(runner);
    bitvec_destroy(data);

    return 0;
}
