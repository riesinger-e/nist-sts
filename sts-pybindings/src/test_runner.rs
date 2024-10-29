use crate::nist_sts::{BitVec, Test, TestResult};
use crate::test_args::*;
use crate::{RunnerError, TestError};
use pyo3::prelude::*;
use sts_lib::{test_runner, Error, TestArgs};

type TestResultIteratorItem = (sts_lib::Test, Result<Vec<sts_lib::TestResult>, Error>);

/// Iterator for the result of the [run_tests] function.
#[pyclass]
pub struct TestResultIterator {
    iter: Box<dyn Iterator<Item = TestResultIteratorItem> + Send + 'static>,
}

#[pymethods]
impl TestResultIterator {
    pub fn __iter__(this: PyRef<'_, Self>) -> PyRef<'_, Self> {
        this
    }

    pub fn __next__(mut this: PyRefMut<'_, Self>) -> PyResult<Option<(Test, PyObject)>> {
        if let Some((test, res)) = this.iter.next() {
            let res = match res {
                Ok(res) => {
                    if res.len() == 1 {
                        TestResult(res[0]).into_py(this.py())
                    } else {
                        res.into_iter()
                            .map(TestResult)
                            .collect::<Vec<_>>()
                            .into_py(this.py())
                    }
                }
                Err(e) => return Err(TestError::new_err(e.to_string())),
            };

            Ok(Some((test.into(), res)))
        } else {
            Ok(None)
        }
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
///
/// ## Return value
///
/// An iterator of tuples. Each tuple contains the `Test` that was run as the first element, and
/// the second element is either of:
/// * 1 TestResult
/// * a list of TestResults
///
/// ## Errors
///
/// RunnerError if a test is specified more than 1 time.
///
/// If an error occurs while evaluating a test, TestError is thrown.
#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (data, tests=None, frequency_block_arg=None, non_overlapping_template_args=None, overlapping_template_args=None, linear_complexity_arg=None, serial_arg=None, approximate_entropy_arg=None))]
pub fn run_tests(
    data: &BitVec,
    tests: Option<Vec<Test>>,
    frequency_block_arg: Option<FrequencyBlockTestArg>,
    non_overlapping_template_args: Option<NonOverlappingTemplateTestArgs>,
    overlapping_template_args: Option<OverlappingTemplateTestArgs>,
    linear_complexity_arg: Option<LinearComplexityTestArg>,
    serial_arg: Option<SerialTestArg>,
    approximate_entropy_arg: Option<ApproximateEntropyTestArg>,
) -> PyResult<TestResultIterator> {
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
            let tests = tests.into_iter().map(|t| t.into());

            let iter = test_runner::run_tests(data.0.clone(), tests, args)
                .map_err(|e| RunnerError::new_err(format!("Duplicate test: {}", e.0)))?;
            Ok(TestResultIterator {
                iter: Box::new(iter),
            })
        }
        None => {
            let iter = test_runner::run_all_tests(data.0.clone(), args)
                .map_err(|e| RunnerError::new_err(format!("Duplicate test: {}", e.0)))?;
            Ok(TestResultIterator {
                iter: Box::new(iter),
            })
        }
    }
}
