#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "../include/decls.h"
#include "../include/stat_fncs.h"

// macro to call a test and time them - prints the result as JSON, time unit: ms.
#define TIME_TEST(call) { \
        clock_t start = clock(); \
        call; \
        clock_t end = clock(); \
        double diff = 1000.0 * (end - start) / CLOCKS_PER_SEC; \
        printf("{ \"test\": \"%s\", \"time\": %lf }\n", #call, diff); \
    }



int main(int argc, char* argv[]) {
    // check arguments
    if (argc != 3) {
        printf("Usage: %s <input_file> <input_length>\n", argv[0]);
        printf("Input file: binary file that contains the data to test.\n");
        printf("Input length: the length of the input file, in bits.\n");
        return 1;
    }

    // path to the file
    char* input_file = argv[1];

    // arguments - same as in Appendix B
    tp.n = atoi(argv[2]);
    tp.blockFrequencyBlockLength = 128;
    tp.nonOverlappingTemplateBlockLength = 9;
    tp.overlappingTemplateBlockLength = 9;
    tp.approximateEntropyBlockLength = 10;
    tp.serialBlockLength = 16;
    tp.linearComplexitySequenceLength = 500;
    tp.numOfBitStreams = 1;


    // set global output variables so that they do nothing
    for (int i = 1; i <= NUMOFTESTS; i++) {
        stats[i] = fopen("/dev/null", "w");
        results[i] = fopen("/dev/null", "w");
    }

    freqfp = fopen("/dev/null", "w");
    summary = fopen("/dev/null", "w");

    // read sequence
    epsilon = (BitSequence*) calloc(tp.n, sizeof(BitSequence));
    FILE* input = fopen(input_file, "r");
    uint8_t current_byte = 0;
    for (int read_bytes = 0; read_bytes < tp.n / 8; read_bytes++) {
        if (fread(&current_byte, sizeof(uint8_t), 1, input) != 1) {
            printf("Error reading input: too few bytes in file. Expected: %d. Got: %d\n", tp.n / 8, read_bytes);
            return 2;
        }

        // split byte into bits
        int bit_pos = read_bytes * 8;
        for (int i = 0; i < 8; i++) {
            uint8_t current_bit = (current_byte >> (7 - i)) & 0x01;
            epsilon[bit_pos + i] = current_bit;
        }
    }

    fclose(input);

    // execute tests, time them, print the execution time
    TIME_TEST(Frequency(tp.n));
    TIME_TEST(BlockFrequency(tp.blockFrequencyBlockLength, tp.n));
    TIME_TEST(CumulativeSums(tp.n));
    TIME_TEST(Runs(tp.n));
    TIME_TEST(LongestRunOfOnes(tp.n));
    TIME_TEST(Rank(tp.n));
    TIME_TEST(DiscreteFourierTransform(tp.n));
    TIME_TEST(NonOverlappingTemplateMatchings(tp.nonOverlappingTemplateBlockLength, tp.n));
    TIME_TEST(OverlappingTemplateMatchings(tp.overlappingTemplateBlockLength, tp.n));
    TIME_TEST(Universal(tp.n));
    TIME_TEST(ApproximateEntropy(tp.approximateEntropyBlockLength, tp.n));
    TIME_TEST(RandomExcursions(tp.n));
    TIME_TEST(RandomExcursionsVariant(tp.n));
    TIME_TEST(Serial(tp.serialBlockLength,tp.n));
    TIME_TEST(LinearComplexity(tp.linearComplexitySequenceLength, tp.n));

    // close all open files
    for (int i = 1; i <= NUMOFTESTS; i++) {
        fclose(stats[i]);
        fclose(results[i]);
    }

    fclose(freqfp);
    fclose(summary);

    return 0;
}
