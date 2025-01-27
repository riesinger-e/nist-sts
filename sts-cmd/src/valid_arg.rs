//! Struct and conversion method for a validated arg.

use crate::cmd_args::RegularArgs;
use crate::toml_config::{
    TomlConfig, TomlFrequencyBlockLinearComplexity, TomlInput, TomlNonOverlapping, TomlOutput,
    TomlOverlapping, TomlSerialApproximateEntropy, TomlTest, TomlTestArguments,
};
use crate::InputFormat;
use std::num::NonZero;
use std::path::PathBuf;
use sts_lib::{Test, TestArgs};

/// Which tests are to be run (allowed or blocked)
#[derive(Clone, Debug)]
pub enum TestsToRun {
    AllowList(Vec<Test>),
    BlockList(Vec<Test>),
    All,
}

impl From<crate::cmd_args::TestsToRun> for TestsToRun {
    fn from(value: crate::cmd_args::TestsToRun) -> Self {
        if let Some(tests) = value.tests {
            let tests = tests.into_iter().map(From::from).collect();
            TestsToRun::AllowList(tests)
        } else if let Some(tests) = value.exclude_tests {
            let tests = tests.into_iter().map(From::from).collect();
            TestsToRun::BlockList(tests)
        } else {
            TestsToRun::All
        }
    }
}

impl From<TomlTest> for TestsToRun {
    fn from(value: TomlTest) -> Self {
        if let Some(tests) = value.include {
            let tests = tests.into_iter().map(From::from).collect();
            TestsToRun::AllowList(tests)
        } else if let Some(tests) = value.exclude {
            let tests = tests.into_iter().map(From::from).collect();
            TestsToRun::BlockList(tests)
        } else {
            TestsToRun::All
        }
    }
}

/// To represent the max_length value and split flag combination
#[derive(Debug, Clone)]
pub enum MaxLengthOrSplit {
    /// A max length was given, unit is bits.
    MaxLength(NonZero<usize>),
    /// A split length was given, unit is bytes.
    Split(NonZero<usize>),
    /// Neither a max length nor a split length was given.
    None,
}

/// A validated config with a valid state that can be used to run tests.
#[derive(Clone, Debug)]
pub struct ValidatedConfig {
    /// Path to the input file (random data)
    pub input_file: PathBuf,
    /// Input format
    pub input_format: InputFormat,
    /// See [MaxLengthOrSplit]
    pub max_length_or_split: MaxLengthOrSplit,
    /// The exact tests to be run.
    pub tests_to_run: TestsToRun,
    /// Finished test arguments
    pub test_arguments: TestArgs,
    /// An optional path to save the outputs to.
    pub output_path: Option<PathBuf>,
    /// Write console output about individual tests, else only summaries.
    pub console_output: bool,
}

impl ValidatedConfig {
    /// Creates a valid config from the command line arguments.
    ///
    /// These function may only be called if `config_file` was unspecified. Otherwise, a panic will occur.
    pub fn try_from_cmd_args(args: RegularArgs) -> Result<Self, &'static str> {
        let RegularArgs {
            input_file,
            input_format,
            max_length,
            split,
            output_path,
            tests_to_run,
            overrides,
            no_console,
        } = args;

        let input_file =
            input_file.expect("input_file should be Some() except if a config file was specified.");
        let input_format =
            input_format.expect("input_format should be Some() if input_file was given.");

        let test_arguments = if let Some(overrides) = parse_overrides(overrides) {
            overrides?.try_into()?
        } else {
            Default::default()
        };

        let max_length_or_split = handle_split(split, max_length)?;

        Ok(Self {
            input_file,
            input_format,
            max_length_or_split,
            tests_to_run: tests_to_run.into(),
            test_arguments,
            output_path,
            console_output: !no_console,
        })
    }

    /// Creates a valid config from the specified toml configuration, uses overrides from the
    /// command line.
    pub fn try_from_toml(toml: TomlConfig, args: RegularArgs) -> Result<Self, &'static str> {
        let TomlConfig {
            input:
                TomlInput {
                    input_file,
                    input_format,
                    max_length,
                    split,
                },
            test,
            output,
            arguments,
        } = toml;

        let TomlOutput {
            path: output_path,
            no_console,
        } = output.unwrap_or_default();

        let RegularArgs {
            input_file: args_input_file,
            input_format: args_input_format,
            max_length: args_input_length,
            split: args_split,
            tests_to_run,
            overrides,
            output_path: args_output_path,
            no_console: args_no_console,
        } = args;

        // cmd args overwrite everywhere
        let input_file = args_input_file
            .or(input_file)
            .ok_or("The input file is unspecified in the config file and the cmd args!")?;
        let input_format = args_input_format
            .or(input_format)
            .ok_or("The input format is unspecified in the config file and the cmd args!")?;
        let max_length = max_length.or(args_input_length);
        let split = args_split || split;
        let output_path = args_output_path.or(output_path);
        let console_output = !(args_no_console || no_console);

        let tests_to_run: TestsToRun = {
            let cmd_tests_to_run = tests_to_run.into();

            if let TestsToRun::All = &cmd_tests_to_run {
                // no command line switch was specified, use the toml file
                test.into()
            } else {
                cmd_tests_to_run
            }
        };

        let test_arguments = if let Some(mut toml_args) = arguments {
            // override if necessary
            if let Some(overrides) = parse_overrides(overrides) {
                let TomlTestArguments {
                    frequency_block,
                    non_overlapping_template_matching,
                    overlapping_template_matching,
                    linear_complexity,
                    serial,
                    approximate_entropy,
                } = overrides?;

                if let Some(arg) = frequency_block {
                    match toml_args.frequency_block.as_mut() {
                        Some(outer) => override_frequency_linear(outer, arg),
                        None => toml_args.frequency_block = Some(arg),
                    }
                }

                if let Some(arg) = non_overlapping_template_matching {
                    match toml_args.non_overlapping_template_matching.as_mut() {
                        Some(outer) => {
                            let TomlNonOverlapping {
                                template_length,
                                count_blocks,
                            } = arg;

                            if template_length.is_some() {
                                outer.template_length = template_length;
                            }

                            if count_blocks.is_some() {
                                outer.count_blocks = count_blocks;
                            }
                        }
                        None => toml_args.non_overlapping_template_matching = Some(arg),
                    }
                }

                if let Some(arg) = overlapping_template_matching {
                    match toml_args.overlapping_template_matching.as_mut() {
                        Some(outer) => {
                            let TomlOverlapping {
                                template_length,
                                block_length,
                                freedom,
                                nist_behaviour,
                            } = arg;

                            if template_length.is_some() {
                                outer.template_length = template_length;
                            }

                            if block_length.is_some() {
                                outer.block_length = block_length;
                            }

                            if freedom.is_some() {
                                outer.freedom = freedom;
                            }

                            if nist_behaviour.is_some() {
                                outer.nist_behaviour = nist_behaviour;
                            }
                        }
                        None => toml_args.overlapping_template_matching = Some(arg),
                    }
                }

                if let Some(arg) = linear_complexity {
                    match toml_args.linear_complexity.as_mut() {
                        Some(outer) => override_frequency_linear(outer, arg),
                        None => toml_args.linear_complexity = Some(arg),
                    }
                }

                if let Some(arg) = serial {
                    match toml_args.serial.as_mut() {
                        Some(outer) => override_serial_entropy(outer, arg),
                        None => toml_args.serial = Some(arg),
                    }
                }

                if let Some(arg) = approximate_entropy {
                    match toml_args.approximate_entropy.as_mut() {
                        Some(outer) => override_serial_entropy(outer, arg),
                        None => toml_args.approximate_entropy = Some(arg),
                    }
                }
            }

            toml_args.try_into()?
        } else if let Some(overrides) = parse_overrides(overrides) {
            // only overrides
            overrides?.try_into()?
        } else {
            Default::default()
        };

        let max_length_or_split = handle_split(split, max_length)?;

        Ok(Self {
            input_file,
            input_format,
            max_length_or_split,
            tests_to_run,
            test_arguments,
            output_path,
            console_output,
        })
    }
}

/// Parse the overrides given via command line
fn parse_overrides(
    overrides: Option<Vec<String>>,
) -> Option<Result<TomlTestArguments, &'static str>> {
    let overrides =
        overrides.and_then(|overrides| overrides.into_iter().reduce(|a, b| a + "\n" + &b))?;

    Some(toml::from_str(&overrides).map_err(|_| "argument overrides is not valid TOML"))
}

/// Does the overrides for frequency block test and linear complexity test: same TOML argument type
fn override_frequency_linear(
    outer: &mut TomlFrequencyBlockLinearComplexity,
    new_data: TomlFrequencyBlockLinearComplexity,
) {
    let TomlFrequencyBlockLinearComplexity {
        block_length,
        choose_automatically,
    } = new_data;

    if block_length.is_some() {
        outer.block_length = block_length;
    }

    if choose_automatically.is_some() {
        outer.choose_automatically = choose_automatically;
    }
}

/// Does the overrides for serial test and approximate entropy test: same TOML argument type
fn override_serial_entropy(
    outer: &mut TomlSerialApproximateEntropy,
    new_data: TomlSerialApproximateEntropy,
) {
    let TomlSerialApproximateEntropy { block_length } = new_data;

    if block_length.is_some() {
        outer.block_length = block_length;
    }
}

/// Handle the split flag, in combination with max_length
fn handle_split(
    split: bool,
    max_length: Option<NonZero<usize>>,
) -> Result<MaxLengthOrSplit, &'static str> {
    if split {
        let Some(max_length) = max_length else {
            return Err("max_length should be Some() - split is set");
        };

        if max_length.get() % 8 != 0 {
            return Err("max_length must denote full bytes (be divisible by 8)");
        }

        // since max_length % 8 == 0 and max_length != 0 --> max_length >= 8 --> unwrap()
        // is unreachable.
        let split_bytes = NonZero::new(max_length.get() / 8).unwrap();
        Ok(MaxLengthOrSplit::Split(split_bytes))
    } else {
        match max_length {
            None => Ok(MaxLengthOrSplit::None),
            Some(max_length) => Ok(MaxLengthOrSplit::MaxLength(max_length)),
        }
    }
}
