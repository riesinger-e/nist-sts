# Benchmarking

The purpose of this command line application is to benchmark this implementation against the NIST reference implementation.
This works by executing all tests 100 times each for 5 sample files and calculating the average. 

**Warning**: The NIST reference implementation only works on UNIX systems - this application will refuse to work on non-UNIX systems.

## How to build the NIST reference implementation for benchmarking

Since the NIST reference implementation only provides a TUI and no time values, some modifications are needed:

1. Download the NIST reference implementation and extract it. You should have a directory `sts-2.1.2`.
2. Copy the provided `assess.c` to `sts-2.1.2/src/`, overwriting the original file.
3. Build the NIST reference implementation by executing `make` in the directory `sts-2.1.2`. You should now have
   an executable `assess` in the directory.
4. Do NOT move the `assess` executable! The executable needs the resource folder `templates` to be in the same directory as itself.

## Executing the benchmark

```sh
cargo run --release -p benchmarking -- \
  <PATH_TO_BUILT_ASSESS_BINARY> <PATH_TO_TEST_FILES_DIRECTORY>
```

Replace `<PATH_TO_BUILT_ASSESS_BINARY>` with the path to the built `assess` binary of the NIST reference implementation.

Replace `<PATH_TO_TEST_FILES_DIRECTORY>` with the path to the test files directory. The test files are contained within this 
repository, from the repository root: `sts-lib/test-files`.

You absolutely MUST use the release flag when using `cargo run`, otherwise the results will not be accurate.

The application will print the average execution time per implementation and test and the difference in percent.

## Output

The output contains per-file per-test comparisons, and a per-test comparison over all used files. 

`benchmark.txt` contains the result of the benchmark, executed on the developers machine. As can be seen, this implementation is faster 
in every tested case.