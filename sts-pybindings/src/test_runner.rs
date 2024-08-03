use crate::nist_sts::{BitVec, Test, TestResult};
use crate::test_args::*;
use crate::{RunnerError, TestError};
use pyo3::prelude::*;
use sts_lib::{test_runner, TestArgs};

/// The test runner. Can be used to run multiple tests with one call.
#[pyclass]
#[derive(Default)]
pub struct TestRunner(test_runner::TestRunner);

#[pymethods]
impl TestRunner {
    /// Creates a new instance of the runner. No arguments.
    #[new]
    pub fn new() -> Self {
        Self(Default::default())
    }

    /// Returns the test result for the given test. Because some tests return multiple results,
    /// a list may be returned.
    ///
    /// ## Arguments
    ///
    /// - test: must be of type `Test`. May not be missing.
    ///
    /// ## Exceptions
    ///
    /// This function raises an exception if no test result is stored for this test.
    pub fn get_test_result(&mut self, python: Python<'_>, test: Test) -> PyResult<PyObject> {
        let test = test.into();
        let results = self.0.get_test_result(test);

        let results = match results {
            Some(res) => res,
            None => {
                return Err(RunnerError::new_err(format!(
                    "Test {test} was not run or result was already retrieved."
                )))
            }
        };

        let results: Vec<TestResult> = results.into_iter().map(TestResult).collect();

        if results.len() == 1 {
            Ok(results[0].into_py(python))
        } else {
            Ok(results.into_py(python))
        }
    }

    /// Runs the tests.
    ///
    /// ## Arguments
    ///
    /// Main arguments:
    /// - data: `BitVec` - the test data to run the tests on.
    /// - tests: `[Test]` - the tests to run. If unspecified, runs all tests.
    ///
    /// Test arguments: optionally, arguments for tests that need them can be specified. If
    /// left unspecified, default values will be used.
    /// - frequency_block_arg: `FrequencyBlockTestArg`
    /// - non_overlapping_template_args: `NonOverlappingTemplateTestArgs`
    /// - overlapping_template_args: `OverlappingTemplateTestArgs`
    /// - linear_complexity_arg: `LinearComplexityTestArg`
    /// - serial_arg: `SerialTestArg`
    /// - approximate_entropy_arg: `ApproximateEntropyTestArg`
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (data, tests=None, frequency_block_arg=None, non_overlapping_template_args=None, overlapping_template_args=None, linear_complexity_arg=None, serial_arg=None, approximate_entropy_arg=None))]
    pub fn run_tests(
        &mut self,
        data: &BitVec,
        tests: Option<Vec<Test>>,
        frequency_block_arg: Option<FrequencyBlockTestArg>,
        non_overlapping_template_args: Option<NonOverlappingTemplateTestArgs>,
        overlapping_template_args: Option<OverlappingTemplateTestArgs>,
        linear_complexity_arg: Option<LinearComplexityTestArg>,
        serial_arg: Option<SerialTestArg>,
        approximate_entropy_arg: Option<ApproximateEntropyTestArg>,
    ) -> PyResult<()> {
        // assemble args (or use defaults if not there)
        let args = TestArgs {
            frequency_block: frequency_block_arg.unwrap_or_default().0,
            non_overlapping_template: non_overlapping_template_args.unwrap_or_default().0,
            overlapping_template: overlapping_template_args.unwrap_or_default().0,
            linear_complexity: linear_complexity_arg.unwrap_or_default().0,
            serial: serial_arg.unwrap_or_default().0,
            approximate_entropy: approximate_entropy_arg.unwrap_or_default().0,
        };

        let res = match tests {
            Some(tests) => {
                let tests = tests.into_iter()
                    .map(|t| t.into());
                self.0.run_tests(tests, &data.0, args)
            }
            None => self.0.run_all_tests(&data.0, args)
        };

        match res {
            Ok(()) => Ok(()),
            Err(e @ test_runner::RunnerError::Test(_)) => Err(TestError::new_err(e.to_string())),
            Err(e @ test_runner::RunnerError::Duplicate(_)) => Err(RunnerError::new_err(e.to_string())),
        }
    }
}
