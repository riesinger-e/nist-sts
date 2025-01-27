//! Everything necessary for command line arguments.

use crate::{ArgTest, InputFormat};
use clap::{Args, Parser};
use std::num::NonZero;
use std::path::PathBuf;

/// The command line arguments.
#[derive(Debug, Parser)]
#[command(version, author, about, long_about = None)]
pub struct CmdArgs {
    // If an input file is specified, a config file is not needed, but allowed.
    /// Path to an optional configuration file. Required if the input file is not specified.
    ///
    /// The configuration written in the config file can be supplemented by command line switches.
    #[arg(short, long, required_unless_present = "input_file")]
    pub config_file: Option<PathBuf>,
    #[command(flatten)]
    pub regular_args: RegularArgs,
}

/// The "regular" command line arguments (everything except for config file)
#[derive(Debug, Clone, Args)]
#[group(required = false, multiple = true)]
pub struct RegularArgs {
    /// Path to the input file. Mandatory.
    #[arg(short, long = "input", requires = "input_format")]
    pub input_file: Option<PathBuf>,
    /// The input file format. Required if a input file is specified.
    #[arg(short = 'f', long)]
    pub input_format: Option<InputFormat>,
    /// The maximum length of the sequence to test, in bits.
    #[arg(short = 'l', long)]
    pub max_length: Option<NonZero<usize>>,
    /// Split the input file into parts with exactly max_length bits, testing each part.
    /// 
    /// The remainder is discarded. Requires max_length to be whole bytes (divisible by 8).
    /// If the output path is set, multiple output files with the names 
    /// "<FILE_NAME>_<IDX>.<EXTENSION>" will be created, with <FILE_NAME> denoting the user-provided 
    /// filename, <EXTENSION> the user-provided extension, and <IDX> the index of the tested part, 
    /// supplied by the application.
    #[arg(long, requires = "max_length")]
    pub split: bool,
    /// Optional path to save the results to. Optional.
    ///
    /// If given, the results will be saved in CSV format with ';' delimiter and the following columns:
    /// test name; time in ms; result no.; PASS/FAIL; P-Value; comment
    ///
    /// If a test returns multiple results, test name and time in ms will be the same for all of them.
    /// If a test returns an error, PASS/FAIL will read "ERROR", P-Value will be -1 and comment will
    /// specify the exact error.
    #[arg(short, long = "output")]
    pub output_path: Option<PathBuf>,
    /// The tests to run: either include specific tests or exclude specific tests, if neither is
    /// set: run all tests.
    #[command(flatten)]
    pub tests_to_run: TestsToRun,
    /// Test argument overrides in TOML format.
    ///
    /// Use the same format as the config file, key 'arguments' is implied.
    /// e.g. 'serial.block-length = 3'.
    #[arg(long, value_delimiter = ',')]
    pub overrides: Option<Vec<String>>,
    /// Reduce the console output to only test run summaries (either all tests passed or not).
    #[arg(long)]
    pub no_console: bool,
}

/// Which tests are to be run. Allows only one of these options to be used.
#[derive(Debug, Clone, Args)]
#[group(required = false, multiple = false)]
pub struct TestsToRun {
    /// Run only the specified tests.
    ///
    /// If neither this option nor '--exclude-tests' is specified, all tests are run, except
    /// for those whose input length requirements are not satisfied.
    #[arg(short, long, value_delimiter = ',')]
    pub tests: Option<Vec<ArgTest>>,
    /// Run all available tests except for the excluded tests.
    /// Tests whose input length requirements are not satisfied, are skipped.
    ///
    /// If neither this option nor '--tests' is specified, all tests are run, except
    /// for those whose input length requirements are not satisfied.
    #[arg(short, long, value_delimiter = ',')]
    pub exclude_tests: Option<Vec<ArgTest>>,
}
