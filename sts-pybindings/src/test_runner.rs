use crate::nist_sts::{BitVec, Test, TestResult};
use crate::test_args::*;
use crate::{RunnerError};
use pyo3::prelude::*;
use sts_lib::{Error, test_runner, TestArgs};

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
///
/// ## Return value
///
/// A list of tuples. For each run test, either contains 1 TestResult, a list of TestResults, or a
/// string with the description of the error that happened when the test ran.
///
/// ## Errors
///
/// RunnerError if a test is specified more than 1 time.
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (data, tests=None, frequency_block_arg=None, non_overlapping_template_args=None, overlapping_template_args=None, linear_complexity_arg=None, serial_arg=None, approximate_entropy_arg=None))]
pub fn run_tests(
    python: Python<'_>,
    data: &BitVec,
    tests: Option<Vec<Test>>,
    frequency_block_arg: Option<FrequencyBlockTestArg>,
    non_overlapping_template_args: Option<NonOverlappingTemplateTestArgs>,
    overlapping_template_args: Option<OverlappingTemplateTestArgs>,
    linear_complexity_arg: Option<LinearComplexityTestArg>,
    serial_arg: Option<SerialTestArg>,
    approximate_entropy_arg: Option<ApproximateEntropyTestArg>,
) -> PyResult<Vec<(Test, PyObject)>> {
    // assemble args (or use defaults if not there)
    let args = TestArgs {
        frequency_block: frequency_block_arg.unwrap_or_default().0,
        non_overlapping_template: non_overlapping_template_args.unwrap_or_default().0,
        overlapping_template: overlapping_template_args.unwrap_or_default().0,
        linear_complexity: linear_complexity_arg.unwrap_or_default().0,
        serial: serial_arg.unwrap_or_default().0,
        approximate_entropy: approximate_entropy_arg.unwrap_or_default().0,
    };

    match tests {
        Some(tests) => {
            let tests = tests.into_iter()
                .map(|t| t.into());

            let iter = test_runner::run_tests(tests, &data.0, args)
                .map_err(|e| RunnerError::new_err(format!("Duplicate test: {}", e.0)))?;
            Ok(handle_result_iter(python, iter))
        }
        None => {
            let iter = test_runner::run_all_tests(&data.0, args)
                .map_err(|e| RunnerError::new_err(format!("Duplicate test: {}", e.0)))?;
            Ok(handle_result_iter(python, iter))
        }
    }
}

/// Creates the python result iterator for [run_tests].
fn handle_result_iter(python: Python<'_>, iter: impl Iterator<Item=(sts_lib::Test, Result<Vec<sts_lib::TestResult>, Error>)>) -> Vec<(Test, PyObject)> {
    iter
        .map(|(test, res)| {
            let res = match res{
                Ok(res) => {
                    if res.len() == 1 {
                        TestResult(res[0]).into_py(python)
                    } else {
                        res.into_iter()
                            .map(TestResult)
                            .collect::<Vec<_>>()
                            .into_py(python)
                    }
                }
                Err(e) => {
                    e.to_string().into_py(python)
                }
            };

            (test.into(), res)
        })
        .collect()
}
