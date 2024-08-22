# sts-lib

This library implements the statistical tests for RNGs and PRNGs as defined by the NIST SP 800-22r1a.

## Usage

To run a test, you need to load the bit sequence to be tested into the `BitVec` structure. This structure supports data 
loading from bytes (as `[u8]`), bits (as `[bool]`), or ASCII strings, where the ASCII character "0" denotes bit `0` and "1" 
denotes bit `1`. ASCII strings can be loaded fallible, meaning the occurrence of any other character causes the sequence to
not load, or lossy, meaning any other character is ignored.

### Run a single test

To run a single test, just call the test function, which is defined in its own submodule in the module `test`.
If the test needs an argument, the argument type is defined in the same module as the test function. Each argument
type implements `Default::default()`, meaning a default value is available.

Each test returns a `Result` type. An error here means that something serious, such as a arithmetic overflow, 
invalid floating point values, etc. happened. The `Ok` value of the result is either one `TestResult` or 
multiple `[TestResult]`. Each `TestResult` contains a P-Value, which is >= 0.0 and <= 1.0. The smaller the 
P-Value is, the less likely the sequence is random. Each `TestResult` may also optionally contain a comment,
e.g. denoting the exact origin of the result in tests that contain multiple results.

### Use the test runner to run multiple tests at once

To run multiple tests in one go, use the functions defined in the module `test_runner`.
If invalid parameters are specified, a `RunnerError` is returned immediately, else an iterator over the test results, linked
with the test name, is returned. The iterator works lazily, meaning each test is only run when its result is queried.

To use custom test arguments, use the struct `TestArgs`.

## Verify that the tests work

This library implements unit tests for every single statistical test, some more complex methods, and, for the 
inputs defined in NIST SP 800-22r1a, for all statistical tests. To run all unit tests, use `cargo test`. To run
a specific unit test, check the `unit_tests` subdirectory for the name of the test method.

The tests are sorted into three modules:
* `unit_tests/mod.rs` defines some helper methods and tests for functions that are not statistical tests
* `unit_tests/nist_text_examples.rs` defines at least 1 test for each statistical test. The inputs and outputs are
  taken from the examples in NIST SP 800-22r1a, section 2.
* `unit_tests/full_examples.rs` defines tests for the inputs defined in NIST SP 800-22r1a, appendix B.