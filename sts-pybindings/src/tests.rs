use crate::bitvec::BitVec;
use crate::nist_sts::TestResult;
use crate::test_args::*;
use crate::TestError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use sts_lib::tests::*;

/// Frequency (mono bit) test - No. 1
///
/// This test focuses on the numbers of ones and zeros in the sequence - the proportion should
/// be roughly 50:50.
///
/// ## Arguments
///
/// - data: `BitVec` to test.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen.
#[pyfunction]
pub fn frequency_test(data: &BitVec) -> PyResult<TestResult> {
    frequency::frequency_test(&data.0)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// Frequency Test within a block - No. 2
///
/// This tests for the same property as `frequency_test`, but within M-bit blocks.
/// It is recommended that each block has a length of at least 100 bits.
///
/// ## Arguments
///
/// - data: `BitVec` to test.
/// - test_arg: `FrequencyBlockTestArg` - the block length, may be left unspecified.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen.
#[pyfunction]
#[pyo3(signature = (data, test_arg=None))]
pub fn frequency_block_test(
    data: &BitVec,
    test_arg: Option<FrequencyBlockTestArg>,
) -> PyResult<TestResult> {
    let arg = test_arg.map(|a| a.0).unwrap_or_default();

    frequency_block::frequency_block_test(&data.0, arg)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// Runs test - No. 3
///
/// This tests focuses on the number of runs in the sequence. A run is an uninterrupted sequence of
/// identical bits.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Should have at least 100 bits length.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
pub fn runs_test(data: &BitVec) -> PyResult<TestResult> {
    runs::runs_test(&data.0)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// Test for the Longest Run of Ones in a Block - No. 4
///
/// This test determines whether the longest run (See `runs_test`) of ones
/// in a block is consistent with the expected value for a random sequence.
///
/// An irregularity in the length of longest run of ones also implies an irregularity in the length
/// of the longest runs of zeroes, meaning that only this test is necessary. See the NIST publication.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Has to be at least 128 bits in length.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
pub fn longest_runs_of_ones_test(data: &BitVec) -> PyResult<TestResult> {
    longest_run_of_ones::longest_run_of_ones_test(&data.0)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// Binary Matrix Rank Test -  No. 5
///
/// This test checks for linear dependence among fixed length substrings of the sequence.
/// These substrings are interpreted as matrices of size 32x32.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Has to consist of at least 38 912 bits = 4864 bytes
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
pub fn binary_matrix_rank_test(data: &BitVec) -> PyResult<TestResult> {
    binary_matrix_rank::binary_matrix_rank_test(&data.0)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// The Spectral Discrete Fourier Transform test - No. 6
///
/// This test focuses on the peak heights in the DFT of the input sequence. This is used to detect
/// periodic features that indicate a deviation from a random sequence.
///
/// ## Arguments
///
/// - data: `BitVec` to test. It is recommended (but not required) for the input to be of at least 1000 bits.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
pub fn spectral_dft_test(data: &BitVec) -> PyResult<TestResult> {
    spectral_dft::spectral_dft_test(&data.0)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// Non-overlapping Template Matching test - No. 7
///
/// This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
/// This test uses an m-bit window to search for an m-bit pattern.
///
/// ## Arguments
///
/// - data: `BitVec` to test.
/// - test_arg: `NonOverlappingTemplateTestArgs`. May be left unspecified.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
#[pyo3(signature = (data, test_arg=None))]
pub fn non_overlapping_template_matching_test(
    data: &BitVec,
    test_arg: Option<NonOverlappingTemplateTestArgs>,
) -> PyResult<Vec<TestResult>> {
    let arg = test_arg.map(|a| a.0).unwrap_or_default();

    template_matching::non_overlapping::non_overlapping_template_matching_test(&data.0, arg)
        .map(|results| results.into_iter().map(TestResult).collect())
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// Overlapping Template Matching test - No. 8
///
/// This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
/// This test uses an m-bit window to search for an m-bit pattern.
/// The big difference to the `non_overlapping_template_matching_test` test is that template matches
/// may overlap.
///
/// The default arguments for this test derivate significantly from the NIST reference implementation,
/// since the NIST reference implementation for this test is known bad.
/// The problem is that the PI values from NIST are wrong - the correction from Hamano and Kaneko is used.
///
/// Details about the problems:
/// * Even though the pi values should be revised according to the paper, both the example and
///   the implementation still use the old, inaccurate calculation.
/// * The (not working) fixed values according to Hamano and Kaneko only work for very specific cases.
/// * The value *K*, as given in the paper, ist just wrong. You don't need a statistics degree to see
///   that it is 6 and not 5.
///
/// # About performance
///
/// This test is quite slow in debug mode when using the more precise pi values (non-NIST behaviour),
/// taking several seconds - it runs good when using release mode.
/// For better performance, values that are calculated once are cached.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Length must be at least 10^6 bits.
/// - test_arg: `OverlappingTemplateTestArgs`. May be left unspecified.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
#[pyo3(signature = (data, test_arg=None))]
pub fn overlapping_template_matching_test(
    data: &BitVec,
    test_arg: Option<OverlappingTemplateTestArgs>,
) -> PyResult<TestResult> {
    let arg = test_arg.map(|a| a.0).unwrap_or_default();

    template_matching::overlapping::overlapping_template_matching_test(&data.0, arg)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// Maurer's "Universal Statistical" Test - No. 9
///
/// This test detects if the given sequence if significantly compressible without information loss.
/// If it is, it is considered non-random.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Recommended minimum length of 387 840 bits. Absolute minimum length
///   of 2020 bits, smaller inputs will raise an error.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
pub fn maurers_universal_statistical_test(data: &BitVec) -> PyResult<TestResult> {
    maurers_universal_statistical::maurers_universal_statistical_test(&data.0)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// The linear complexity test - No. 10
///
/// This test determines the randomness of a sequence by calculating the minimum length of a linear
/// feedback shift register that can create the sequence. Random sequences need longer LSFRs.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Minimum length of 10^6 bits.
/// - test_arg: `LinearComplexityTestArg`. May be left unspecified.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
#[pyo3(signature = (data, test_arg=None))]
pub fn linear_complexity_test(
    data: &BitVec,
    test_arg: Option<LinearComplexityTestArg>,
) -> PyResult<TestResult> {
    let arg = test_arg.map(|a| a.0).unwrap_or_default();

    linear_complexity::linear_complexity_test(&data.0, arg)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// The serial test - No. 11
///
/// This test checks the frequency of all 2^m overlapping m-bit patterns in the sequence. Random
/// sequences should be uniform. For *m = 1*, this would be the same as the
/// `frequency_test()`.
///
/// The paper describes the test slightly wrong: in 2.11.5 step 5, the second argument need to be
/// halved in both *igamc* calculations. Only then are the calculated P-values equal to the P-values
/// described in 2.11.6 and the reference implementation.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Minimum recommended length of 2^19 bits.
/// - test_arg: `LinearComplexityTestArg`. May be left unspecified.
///
/// If the combination of the given data and `test_arg` is invalid,
/// an error is raised. For the exact constraints, see `SerialTestArg`.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
#[pyo3(signature = (data, test_arg=None))]
pub fn serial_test(
    data: &BitVec,
    test_arg: Option<SerialTestArg>,
) -> PyResult<(TestResult, TestResult)> {
    let arg = test_arg.map(|a| a.0).unwrap_or_default();

    serial::serial_test(&data.0, arg)
        .map(|[res1, res2]| (TestResult(res1), TestResult(res2)))
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// The approximate entropy test - No. 12
///
/// This test is similar to the Serial Test. It compares the frequency
/// of overlapping blocks with the two block lengths *m* and *m + 1* against the expected result
/// of a random sequence.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Minimum recommended length of 2^16 bits.
/// - test_arg: `ApproximateEntropyTestArg`. May be left unspecified.
///
/// If the combination of the given data and `test_arg` is invalid,
/// an error is raised. For the exact constraints, see `ApproximateEntropyTestArg`.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen
#[pyfunction]
#[pyo3(signature = (data, test_arg=None))]
pub fn approximate_entropy_test(
    data: &BitVec,
    test_arg: Option<ApproximateEntropyTestArg>,
) -> PyResult<TestResult> {
    let arg = test_arg.map(|a| a.0).unwrap_or_default();

    approximate_entropy::approximate_entropy_test(&data.0, arg)
        .map(TestResult)
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// The cumulative sums test - No. 13
///
/// This test calculates cumulative partial sums of the bit sequence, once starting from the
/// first bit and once starting from the last bit, adjusting the digits to -1 and +1 and calculating
/// the maximum absolute partial sum. The test checks if this maximum is within the expected bounds
/// for random sequences.
///
/// This test returns 2 `TestResult`s.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Minimum length of 100 bits.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen.
#[pyfunction]
pub fn cumulative_sums_test(data: &BitVec) -> PyResult<(TestResult, TestResult)> {
    cumulative_sums::cumulative_sums_test(&data.0)
        .map(|[res1, res2]| (TestResult(res1), TestResult(res2)))
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// The random excursions test - No. 14.
///
/// This test, similarly to the Cumulative sums test, calculates
/// cumulative sums of a digit-adjusted (-1, +1) bit sequence, but only from the beginning to the end.
/// This test checks if the frequency of cumulative sums values per cycle is as expected for
/// a random sequence. A cycle consists of all cumulative sums between 2 "0"-values.
///
/// Since the test needs at least 500 cycles to occur, bit sequences with fewer cycles will not
/// raise and error, but all values will be filled with "0.0".
///
/// If the computation finishes successfully, 8 `TestResult`s are returned: one for each tested state,
/// `x`. The results will contain a comment about the state they are calculated from (e.g. "x = 3"),
/// the order is: `[-4, -3, -2, -1, +1, +2, +3, +4]`.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Minimum length of 10^6 bits.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen.
#[pyfunction]
pub fn random_excursions_test(data: &BitVec) -> PyResult<TestResultLen8> {
    random_excursions::random_excursions_test(&data.0)
        .map(|res| TestResultLen8 { data: res })
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// The random excursions variant test.
///
/// This test is quite similar to the Random Excursion Test,
/// with the key difference being that the frequencies are calculated over all cycles, instead of per
/// cycle.
///
/// This test does not require a minimum number of cycles.
///
/// If the computation finishes successfully, 18 `TestResult`s are returned: one for each tested state,
/// `x`. The results will contain a comment about the state they are calculated from (e.g. "x = 3"),
/// the order is: `[-9, -8, -7, -6, -5, -4, -3, -2, -1, +1, +2, +3, +4, +5, +6, +7, +8, +9]`.
///
/// ## Arguments
///
/// - data: `BitVec` to test. Minimum length of 10^6 bits.
///
/// ## Exceptions
///
/// Exceptions of type `TestError` may happen.
#[pyfunction]
pub fn random_excursions_variant_test(data: &BitVec) -> PyResult<TestResultLen18> {
    random_excursions_variant::random_excursions_variant_test(&data.0)
        .map(|res| TestResultLen18 { data: res })
        .map_err(|e| TestError::new_err(e.to_string()))
}

/// Struct to convert a test result with length 8 into a tuple
pub struct TestResultLen8 {
    data: [sts_lib::TestResult; 8],
}

impl<'py> IntoPyObject<'py> for TestResultLen8 {
    type Target = PyTuple;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let data: Vec<_> = self
            .data
            .into_iter()
            .map(|res| TestResult(res).into_pyobject(py))
            .collect::<Result<_, _>>()?;
        PyTuple::new(py, data)
    }
}

/// Struct to convert a test result array with length 18 into a tuple
pub struct TestResultLen18 {
    data: [sts_lib::TestResult; 18],
}

impl<'py> IntoPyObject<'py> for TestResultLen18 {
    type Target = PyTuple;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let data: Vec<_> = self
            .data
            .into_iter()
            .map(|res| TestResult(res).into_pyobject(py))
            .collect::<Result<_, _>>()?;
        PyTuple::new(py, data)
    }
}