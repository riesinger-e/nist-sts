use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::num::NonZero;
use sts_lib::tests::*;

/// The argument for the Frequency test within a block: the block length.
///
/// The block length should be at least 20 bits, with the block length greater than 1% of the
/// total bit length and fewer than 100 total blocks.
#[pyclass(frozen)]
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct FrequencyBlockTestArg(pub(crate) frequency_block::FrequencyBlockTestArg);

#[pymethods]
impl FrequencyBlockTestArg {
    /// The argument for the Frequency test within a block: the block length.
    ///
    /// The block length should be at least 20 bits, with the block length greater than 1% of the
    /// total bit length and fewer than 100 total blocks.
    ///
    /// If no block length is given, a suitable block length will be chosen when the test is run.
    #[new]
    #[pyo3(signature = (block_length=None))]
    pub fn new(block_length: Option<usize>) -> Self {
        let arg = match block_length {
            Some(0) | None => frequency_block::FrequencyBlockTestArg::ChooseAutomatically,
            Some(block_length) => {
                // just checked: is not 0
                let block_length = NonZero::new(block_length).unwrap();
                frequency_block::FrequencyBlockTestArg::new(block_length)
            }
        };
        Self(arg)
    }

    pub fn __repr__(&self) -> String {
        let len = match self.0 {
            frequency_block::FrequencyBlockTestArg::Bytewise(len) => len.get() * 8,
            frequency_block::FrequencyBlockTestArg::Bitwise(len) => len.get(),
            frequency_block::FrequencyBlockTestArg::ChooseAutomatically => {
                return String::from("FrequencyBlockTestArg()");
            }
        };
        format!("FrequencyBlockTestArg({})", len)
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// The arguments for the Non-overlapping Template Matching Test.
///
/// 1. The template length `m` to use, in bits.
///    2 <= `m` <= 21 - recommended: 9.
/// 2. The number of independent blocks to test in the sequence: `N`
///    1 <= `N` < 100 - recommended: 8
///
/// These bounds are checked on creation.
#[pyclass(frozen)]
#[derive(Copy, Clone, Default)]
pub struct NonOverlappingTemplateTestArgs(
    pub(crate) template_matching::non_overlapping::NonOverlappingTemplateTestArgs<'static>,
    Option<(usize, usize)>,
);

#[pymethods]
impl NonOverlappingTemplateTestArgs {
    /// The arguments for the Non-overlapping Template Matching Test.
    ///
    /// 1. The template length `m` to use, in bits.
    ///    2 <= `m` <= 21 - recommended: 9.
    /// 2. The number of independent blocks to test in the sequence: `N`
    ///    1 <= `N` < 100 - recommended: 8
    ///
    /// These bounds are checked on creation.
    ///
    /// ## Arguments
    ///
    /// * template_len: template length in bits
    /// * count_blocks: count of blocks
    ///
    /// Both arguments may be left undefined, a default value will be used instead.
    #[new]
    #[pyo3(signature = (template_len=None, count_blocks=None))]
    pub fn new(template_len: Option<usize>, count_blocks: Option<usize>) -> PyResult<Self> {
        match (template_len, count_blocks) {
            (None, None) => Ok(Self(Default::default(), None)),
            (Some(template_len), Some(count_blocks)) => {
                let arg = template_matching::non_overlapping::NonOverlappingTemplateTestArgs::new(
                    template_len,
                    count_blocks,
                );
                match arg {
                    Some(arg) => Ok(Self(arg, Some((template_len, count_blocks)))),
                    None => Err(PyValueError::new_err(
                        "One or both arguments are out of range",
                    )),
                }
            }
            // use default value for missing argument
            (template_len, count_blocks) => {
                let template_len = template_len.or(Some(template_matching::DEFAULT_TEMPLATE_LENGTH));
                let count_blocks = count_blocks.or(Some(
                    template_matching::non_overlapping::DEFAULT_BLOCK_COUNT,
                ));
                Self::new(template_len, count_blocks)
            }
        }
    }

    pub fn __repr__(&self) -> String {
        match self.1 {
            Some((template_len, count_blocks)) => format!(
                "NonOverlappingTemplateTestArgs(template_len={}, count_blocks={})",
                template_len, count_blocks
            ),
            None => String::from("NonOverlappingTemplateTestArgs"),
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// The arguments for the Overlapping Template Matching Test.
///
/// 1. The template length *m*. 2 <= *m* <= 21. 9 per default.
/// 2. The length of each block, *M*, in bits. 1032 per default.
/// 3. The degrees of freedom, *K*. 6 per default.
///
/// These bounds are checked by all creation functions.
///
/// With all of these arguments, the *pi* values are calculated according to Hamano and Kaneko.
/// If you want to replicate the exact (inaccurate) NIST behaviour, you can set `nist_behaviour=True`.
#[pyclass(frozen)]
#[derive(Copy, Clone, Default)]
pub struct OverlappingTemplateTestArgs(
    pub(crate) template_matching::overlapping::OverlappingTemplateTestArgs,
    Option<(usize, Option<(usize, usize)>)>,
);

#[pymethods]
impl OverlappingTemplateTestArgs {
    /// The arguments for the Overlapping Template Matching Test.
    ///
    /// 1. The template length *m*. 2 <= *m* <= 21. 9 per default.
    /// 2. The length of each block, *M*, in bits. 1032 per default.
    /// 3. The degrees of freedom, *K*. 6 per default.
    ///
    /// These bounds are checked by all creation functions.
    ///
    /// With all of these arguments, the *pi* values are calculated according to Hamano and Kaneko.
    /// If you want to replicate the exact (inaccurate) NIST behaviour, you can set `nist_behaviour=True`.
    ///
    /// ## Arguments
    ///
    /// - template_len
    /// - block_len
    /// - freedom
    /// - nist_behaviour = False
    ///
    /// If all arguments are unspecified, a reasonable default will be chosen, `nist_behaviour=False`.
    /// If `nist_behaviour=True`, only template_len may be specified.
    /// If `nist_behaviour=False`, reasonable defaults will be chosen for missing arguments.
    #[new]
    #[pyo3(signature = (template_len=None, block_len=None, freedom=None, nist_behaviour=false))]
    pub fn new(
        template_len: Option<usize>,
        block_len: Option<usize>,
        freedom: Option<usize>,
        nist_behaviour: bool,
    ) -> PyResult<Self> {
        match (template_len, block_len, freedom, nist_behaviour) {
            (None, None, None, false) => Ok(Self(Default::default(), None)),
            (Some(template_len), None, None, true) => {
                let arg =
                    template_matching::overlapping::OverlappingTemplateTestArgs::new_nist_behaviour(
                        template_len,
                    );
                match arg {
                    Some(arg) => Ok(Self(arg, Some((template_len, None)))),
                    None => Err(PyValueError::new_err(
                        "template_len was out of range for nist_behaviour = true",
                    )),
                }
            }
            (Some(template_len), Some(block_len), Some(freedom), false) => {
                let arg = template_matching::overlapping::OverlappingTemplateTestArgs::new(
                    template_len,
                    block_len,
                    freedom,
                );
                match arg {
                    Some(arg) => Ok(Self(arg, Some((template_len, Some((block_len, freedom)))))),
                    None => Err(PyValueError::new_err(
                        "One or more arguments were out of range for nist_behaviour = false",
                    )),
                }
            }
            // use default arguments
            (template_len, block_len, freedom, false) => {
                let template_len = template_len.or(Some(
                    template_matching::overlapping::DEFAULT_TEMPLATE_LENGTH,
                ));
                let block_len =
                    block_len.or(Some(template_matching::overlapping::DEFAULT_BLOCK_LENGTH));
                let freedom = freedom.or(Some(template_matching::overlapping::DEFAULT_FREEDOM));

                // now all arguments are guaranteed some
                Self::new(template_len, block_len, freedom, false)
            }
            (_, _, _, true) => Err(PyValueError::new_err("Invalid combination of arguments")),
        }
    }

    pub fn __repr__(&self) -> String {
        match self.1 {
            None => "OverlappingTemplateTestArgs()".to_owned(),
            Some((template_len, Some((block_len, freedom)))) => {
                format!("OverlappingTemplateTestArgs(template_len={}, block_len={}, freedom={}, nist_behaviour=False)", template_len, block_len, freedom)
            }
            Some((template_len, None)) => {
                format!(
                    "OverlappingTemplateTestArgs(template_len={}, nist_behaviour=True)",
                    template_len
                )
            }
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// The argument for the Linear Complexity Test.
/// Allows to choose the block length manually or automatically.
///
/// If the block length is chosen manually, the following equations must be true:
/// * 500 <= block length <= 5000
/// * total bit length / block length >= 200
#[pyclass(frozen)]
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct LinearComplexityTestArg(pub(crate) linear_complexity::LinearComplexityTestArg);

#[pymethods]
impl LinearComplexityTestArg {
    /// The argument for the Linear Complexity Test.
    /// Allows to choose the block length manually or automatically.
    ///
    /// If the block length is chosen manually, the following equations must be true:
    /// * 500 <= block length <= 5000
    /// * total bit length / block length >= 200
    ///
    /// ## Arguments
    /// * block_length: should be 500 <= block_length <= 5000, can be left unspecified.
    ///
    /// These constraints are only checked when executing the test.
    #[new]
    #[pyo3(signature = (block_length=None))]
    pub fn new(block_length: Option<usize>) -> Self {
        match block_length {
            Some(0) | None => Self(Default::default()),
            Some(block_length) => {
                // just checked for != 0
                let block_length = NonZero::new(block_length).unwrap();
                Self(linear_complexity::LinearComplexityTestArg::ManualBlockLength(block_length))
            }
        }
    }

    pub fn __repr__(&self) -> String {
        match self.0 {
            linear_complexity::LinearComplexityTestArg::ManualBlockLength(block_length) => {
                format!("LinearComplexityTestArg(block_length={})", block_length)
            }
            linear_complexity::LinearComplexityTestArg::ChooseAutomatically => {
                "LinearComplexityTestArg()".to_owned()
            }
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// The argument for the serial test: the block length in bits to check.
///
/// Argument constraints:
/// 1. the given block length must be >= 2.
/// 2. each value of with the bit length the given block length must be representable as usize,
///     i.e. depending on the platform, 32 or 64 bits.
/// 3. the block length must be < (log2(bit_len of sequence) as int) - 2
///
/// Constraints 1 and 2 are checked when creating the arguments.
///
/// Constraint 3 is checked on executing the test. If the constraint is violated,
/// an exception will be raised.
///
/// The default value for this argument is 16. For this to work, the input length must be at least
/// 2^19 bit.
#[pyclass(frozen)]
#[derive(Copy, Clone, Default)]
pub struct SerialTestArg(pub(crate) serial::SerialTestArg, Option<u8>);

#[pymethods]
impl SerialTestArg {
    /// The argument for the serial test: the block length in bits to check.
    ///
    /// Argument constraints:
    /// 1. the given block length must be >= 2.
    /// 2. each value of with the bit length the given block length must be representable as usize,
    ///     i.e. depending on the platform, 32 or 64 bits.
    /// 3. the block length must be < (log2(bit_len of sequence) as int) - 2
    ///
    /// Constraints 1 and 2 are checked when creating the arguments.
    ///
    /// Constraint 3 is checked on executing the test. If the constraint is violated,
    /// an exception will be raised.
    ///
    /// The default value for this argument is 16. For this to work, the input length must be at least
    /// 2^19 bit.
    ///
    /// ## Arguments
    ///
    /// - block_length: may be left unspecified.
    #[new]
    #[pyo3(signature = (block_length=None))]
    pub fn new(block_length: Option<u8>) -> PyResult<Self> {
        match block_length {
            Some(block_length) => {
                let arg = serial::SerialTestArg::new(block_length);
                match arg {
                    Some(arg) => Ok(Self(arg, Some(block_length))),
                    None => Err(PyValueError::new_err("block_length was out of range.")),
                }
            }
            None => Ok(Self(Default::default(), None)),
        }
    }

    pub fn __repr__(&self) -> String {
        match self.1 {
            None => "SerialTestArg()".to_owned(),
            Some(block_length) => {
                format!("SerialTestArg({})", block_length)
            }
        }
    }

    pub fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// The argument for the approximate entropy test: the block length in bits to check.
///
/// Argument constraints:
/// 1. the given block length must be >= 2.
/// 2. each value of with the bit length the given block length must be representable as usize,
///     i.e. depending on the platform, 32 or 64 bits.
/// 3. the block length must be < (log2(bit_len of sequence) as int) - 5
///
/// Constraints 1 and 2 are checked when creating the arguments.
///
/// Constraint 3 is checked on executing the test. If the constraint is violated,
/// an exception will be raised.
///
/// The default value for this argument is 10. For this to work, the input length must be at least
/// 2^16 bit.
#[pyclass(frozen)]
#[derive(Copy, Clone, Default)]
pub struct ApproximateEntropyTestArg(
    pub(crate) approximate_entropy::ApproximateEntropyTestArg,
    Option<u8>,
);

#[pymethods]
impl ApproximateEntropyTestArg {
    /// The argument for the approximate entropy test: the block length in bits to check.
    ///
    /// Argument constraints:
    /// 1. the given block length must be >= 2.
    /// 2. each value of with the bit length the given block length must be representable as usize,
    ///     i.e. depending on the platform, 32 or 64 bits.
    /// 3. the block length must be < (log2(bit_len of sequence]) as int) - 5
    ///
    /// Constraints 1 and 2 are checked when creating the arguments.
    ///
    /// Constraint 3 is checked on executing the test. If the constraint is violated,
    /// an exception will be raised.
    ///
    /// The default value for this argument is 10. For this to work, the input length must be at least
    /// 2^16 bit.
    ///
    /// ## Arguments
    ///
    /// - block_length: may be left unspecified.
    #[new]
    #[pyo3(signature = (block_length=None))]
    pub fn new(block_length: Option<u8>) -> PyResult<Self> {
        match block_length {
            Some(block_length) => {
                let arg = approximate_entropy::ApproximateEntropyTestArg::new(block_length);
                match arg {
                    Some(arg) => Ok(Self(arg, Some(block_length))),
                    None => Err(PyValueError::new_err("block_length was out of range.")),
                }
            }
            None => Ok(Self(Default::default(), None)),
        }
    }
    
    pub fn __repr__(&self) -> String {
        match self.1 {
            None => String::from("ApproximateEntropyTestArg()"),
            Some(block_length) => {
                format!("ApproximateEntropyTestArg({})", block_length)
            }
        }
    }
}
