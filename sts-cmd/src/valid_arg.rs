//! Struct and conversion method for a validated arg.

use std::num::NonZero;
use crate::cmd_args::RegularArgs;
use crate::toml_config::{
    TomlConfig, TomlFrequencyBlockLinearComplexity, TomlInput, TomlNonOverlapping, TomlOverlapping,
    TomlSerialApproximateEntropy, TomlTest, TomlTestArguments,
};
use crate::InputFormat;
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

/// A validated config with a valid state that can be used to run tests.
#[derive(Clone, Debug)]
pub struct ValidatedConfig {
    pub input_file: PathBuf,
    pub input_format: InputFormat,
    pub max_length: Option<NonZero<usize>>,
    pub tests_to_run: TestsToRun,
    pub test_arguments: TestArgs,
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
            tests_to_run,
            overrides,
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

        Ok(Self {
            input_file,
            input_format,
            max_length,
            tests_to_run: tests_to_run.into(),
            test_arguments,
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
                },
            test,
            arguments,
        } = toml;

        let RegularArgs {
            input_file: args_input_file,
            input_format: args_input_format,
            max_length: args_input_length,
            tests_to_run,
            overrides,
        } = args;

        // cmd args overwrite everywhere
        let input_file = args_input_file
            .or(input_file)
            .ok_or("The input file is unspecified in the config file and the cmd args!")?;
        let input_format = args_input_format
            .or(input_format)
            .ok_or("The input format is unspecified in the config file and the cmd args!")?;
        let max_length = max_length.or(args_input_length);

        let tests_to_run: TestsToRun = {
            let cmd_tests_to_run = tests_to_run.into();

            if let TestsToRun::All = &cmd_tests_to_run {
                // no command line switch was specified, use the toml file
                test.into()
            } else {
                cmd_tests_to_run
            }
        };

        let test_arguments = if let Some(mut args) = arguments {
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
                    match args.frequency_block.as_mut() {
                        Some(outer) => override_frequency_linear(outer, arg),
                        None => args.frequency_block = Some(arg),
                    }
                }

                if let Some(arg) = non_overlapping_template_matching {
                    match args.non_overlapping_template_matching.as_mut() {
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
                        None => args.non_overlapping_template_matching = Some(arg),
                    }
                }

                if let Some(arg) = overlapping_template_matching {
                    match args.overlapping_template_matching.as_mut() {
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
                        None => args.overlapping_template_matching = Some(arg),
                    }
                }

                if let Some(arg) = linear_complexity {
                    match args.linear_complexity.as_mut() {
                        Some(outer) => override_frequency_linear(outer, arg),
                        None => args.linear_complexity = Some(arg),
                    }
                }

                if let Some(arg) = serial {
                    match args.serial.as_mut() {
                        Some(outer) => override_serial_entropy(outer, arg),
                        None => args.serial = Some(arg),
                    }
                }

                if let Some(arg) = approximate_entropy {
                    match args.approximate_entropy.as_mut() {
                        Some(outer) => override_serial_entropy(outer, arg),
                        None => args.approximate_entropy = Some(arg),
                    }
                }
            }

            args.try_into()?
        } else if let Some(overrides) = parse_overrides(overrides) {
            // only overrides
            overrides?.try_into()?
        } else {
            Default::default()
        };

        Ok(Self {
            input_file,
            input_format,
            max_length,
            tests_to_run,
            test_arguments,
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
