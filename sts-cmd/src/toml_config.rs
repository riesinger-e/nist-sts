//! TOML configuration file.

use crate::{ArgTest, InputFormat};
use serde::{Deserialize, Serialize};
use std::num::NonZero;
use std::path::PathBuf;
use sts_lib::tests::approximate_entropy::ApproximateEntropyTestArg;
use sts_lib::tests::frequency_block::FrequencyBlockTestArg;
use sts_lib::tests::linear_complexity::LinearComplexityTestArg;
use sts_lib::tests::serial::SerialTestArg;
use sts_lib::tests::template_matching::non_overlapping::NonOverlappingTemplateTestArgs;
use sts_lib::tests::template_matching::overlapping::OverlappingTemplateTestArgs;
use sts_lib::TestArgs;

/// Struct for the TOML configuration file, the constraints of CmdArgs are not validated here.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TomlConfig {
    // not really optional, must be supplemented from cmd args if missing.
    pub input: TomlInput,
    pub test: TomlTest,
    // each argument is optional
    pub arguments: Option<TomlTestArguments>,
}

/// Input: file, format, max length
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TomlInput {
    pub input_file: Option<PathBuf>,
    pub input_format: Option<InputFormat>,
    pub max_length: Option<NonZero<usize>>,
}

/// Tests to run: allowlist or blocklist
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TomlTest {
    // include tests overrides exclude tests
    pub include: Option<Vec<ArgTest>>,
    pub exclude: Option<Vec<ArgTest>>,
}

/// Test arguments for the test runner. Also used in cmd line overrides.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TomlTestArguments {
    pub frequency_block: Option<TomlFrequencyBlockLinearComplexity>,
    pub non_overlapping_template_matching: Option<TomlNonOverlapping>,
    pub overlapping_template_matching: Option<TomlOverlapping>,
    pub linear_complexity: Option<TomlFrequencyBlockLinearComplexity>,
    pub serial: Option<TomlSerialApproximateEntropy>,
    pub approximate_entropy: Option<TomlSerialApproximateEntropy>,
}

impl TryFrom<TomlTestArguments> for TestArgs {
    type Error = &'static str;

    fn try_from(value: TomlTestArguments) -> Result<Self, Self::Error> {
        let TomlTestArguments {
            frequency_block,
            non_overlapping_template_matching,
            overlapping_template_matching,
            linear_complexity,
            serial,
            approximate_entropy,
        } = value;

        let frequency_block = frequency_block
            .map(|arg| match (arg.choose_automatically, arg.block_length) {
                (_, None) | (Some(true), _) => FrequencyBlockTestArg::ChooseAutomatically,
                (Some(false), Some(block_length)) | (None, Some(block_length)) => {
                    FrequencyBlockTestArg::new(block_length)
                }
            })
            .unwrap_or_default();

        let non_overlapping_template = {
            if let Some(arg) = non_overlapping_template_matching {
                use sts_lib::tests::template_matching::non_overlapping::DEFAULT_BLOCK_COUNT;
                use sts_lib::tests::template_matching::DEFAULT_TEMPLATE_LEN;

                let template_length = arg
                    .template_length
                    .map(NonZero::get)
                    .unwrap_or(DEFAULT_TEMPLATE_LEN);
                let count_blocks = arg
                    .count_blocks
                    .map(NonZero::get)
                    .unwrap_or(DEFAULT_BLOCK_COUNT);

                NonOverlappingTemplateTestArgs::new(template_length, count_blocks)
                    .ok_or("Config file: invalid value for non-overlapping-template-matching.")?
            } else {
                Default::default()
            }
        };

        let overlapping_template = {
            if let Some(arg) = overlapping_template_matching {
                use sts_lib::tests::template_matching::overlapping::{
                    DEFAULT_BLOCK_LENGTH, DEFAULT_FREEDOM, DEFAULT_TEMPLATE_LENGTH,
                };

                let nist_behaviour = arg.nist_behaviour.unwrap_or(false);
                let template_length = arg
                    .template_length
                    .map(NonZero::get)
                    .unwrap_or(DEFAULT_TEMPLATE_LENGTH);

                if nist_behaviour {
                    OverlappingTemplateTestArgs::new_nist_behaviour(template_length)
                } else {
                    let block_length = arg
                        .block_length
                        .map(NonZero::get)
                        .unwrap_or(DEFAULT_BLOCK_LENGTH);
                    let freedom = arg.freedom.map(NonZero::get).unwrap_or(DEFAULT_FREEDOM);
                    OverlappingTemplateTestArgs::new(template_length, block_length, freedom)
                }
                .ok_or("Config file: invalid value for overlapping-template-matching.")?
            } else {
                Default::default()
            }
        };

        let linear_complexity = linear_complexity
            .map(|arg| match (arg.choose_automatically, arg.block_length) {
                (_, None) | (Some(true), _) => LinearComplexityTestArg::ChooseAutomatically,
                (Some(false), Some(block_length)) | (None, Some(block_length)) => {
                    LinearComplexityTestArg::ManualBlockLength(block_length)
                }
            })
            .unwrap_or_default();

        let serial = {
            if let Some(TomlSerialApproximateEntropy {
                block_length: Some(block_length),
            }) = serial
            {
                SerialTestArg::new(block_length.get())
                    .ok_or("Config file: invalid value for serial.block-length")?
            } else {
                Default::default()
            }
        };

        let approximate_entropy = {
            if let Some(TomlSerialApproximateEntropy {
                block_length: Some(block_length),
            }) = approximate_entropy
            {
                ApproximateEntropyTestArg::new(block_length.get())
                    .ok_or("Config file: invalid value for approximate-entropy.block-length")?
            } else {
                Default::default()
            }
        };

        Ok(TestArgs {
            frequency_block,
            non_overlapping_template,
            overlapping_template,
            linear_complexity,
            serial,
            approximate_entropy,
        })
    }
}

/// Test argument for the Frequency test within a block and the linear complexity test.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TomlFrequencyBlockLinearComplexity {
    pub block_length: Option<NonZero<usize>>,
    pub choose_automatically: Option<bool>,
}

/// Test argument for the non-overlapping template matching test.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TomlNonOverlapping {
    pub template_length: Option<NonZero<usize>>,
    pub count_blocks: Option<NonZero<usize>>,
}

/// Test argument for the overlapping template matching test.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TomlOverlapping {
    pub template_length: Option<NonZero<usize>>,
    pub block_length: Option<NonZero<usize>>,
    pub freedom: Option<NonZero<usize>>,
    pub nist_behaviour: Option<bool>,
}

/// Test argument for the serial test and the approximate entropy test.
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", default)]
pub struct TomlSerialApproximateEntropy {
    pub block_length: Option<NonZero<u8>>,
}
