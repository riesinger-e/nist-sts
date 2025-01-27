use anyhow::Context;
use clap::Parser;
use std::ffi::OsStr;
use std::fs;
use std::io::{ErrorKind, Read, Seek};
use std::path::Path;
use std::str::from_utf8;
use std::time::Instant;
use sts_cmd::cmd_args::CmdArgs;
use sts_cmd::csv::CsvFile;
use sts_cmd::toml_config::TomlConfig;
use sts_cmd::valid_arg::{MaxLengthOrSplit, TestsToRun, ValidatedConfig};
use sts_cmd::InputFormat;
use sts_lib::bitvec::BitVec;
use sts_lib::{test_runner, IntoEnumIterator, Test, TestArgs, TestResult, DEFAULT_THRESHOLD};

/// Arguments for [run_tests] - borrowing from a [ValidatedConfig]
#[derive(Debug, Copy, Clone)]
struct TestRunArgs<'a> {
    tests_to_run: &'a TestsToRun,
    test_args: TestArgs,
    csv_path: Option<&'a Path>,
    console_output: bool,
}

impl<'a> TestRunArgs<'a> {
    /// Create an instance from a validated config
    fn from_config(config: &'a ValidatedConfig) -> Self {
        Self {
            tests_to_run: &config.tests_to_run,
            test_args: config.test_arguments,
            csv_path: config.output_path.as_deref(),
            console_output: config.console_output,
        }
    }
}

/// If multiple parts are tested in one execution
#[derive(Debug, Copy, Clone)]
struct Parts {
    /// The current part number
    current: u64,
    /// How many parts there will be
    count: u64,
}

/// Main function.
///
/// On success: prints the test results to stdout, exit code SUCCESS.
/// On error: prints the error to stderr, exit code FAILURE.
///
/// This program takes some arguments and an optional config file, use `--help`.
fn main() -> anyhow::Result<()> {
    let CmdArgs {
        config_file,
        regular_args,
    } = CmdArgs::parse();

    // parse configuration
    let config = if let Some(config_file) = config_file {
        let toml = fs::read_to_string(&config_file)
            .with_context(|| format!("Failed to read config file \"{}\"", config_file.display()))?;

        let toml_config: TomlConfig =
            toml::from_str(&toml).context("Failed to parse the config file")?;
        ValidatedConfig::try_from_toml(toml_config, regular_args)
    } else {
        ValidatedConfig::try_from_cmd_args(regular_args)
    }
    .map_err(|err| anyhow::anyhow!(err))?;

    println!("Reading input file: \"{}\"", config.input_file.display());
    println!();

    match config.input_format {
        InputFormat::Binary | InputFormat::Ascii => handle_ascii_or_binary_input(config),
        InputFormat::AsciiLossy => handle_ascii_lossy_input(config),
    }?;

    println!("Finished testing.");

    Ok(())
}

/// Handles ASCII or binary input, with the converting function given by the caller (to convert from
/// raw bytes to the BitVec, handling the file format).
fn handle_ascii_or_binary_input(config: ValidatedConfig) -> anyhow::Result<()> {
    assert_ne!(config.input_format, InputFormat::AsciiLossy);

    // use the right converter function
    let converter: fn(&[u8]) -> anyhow::Result<BitVec> = match config.input_format {
        InputFormat::Binary => |i| Ok(BitVec::from(i)),
        InputFormat::Ascii => |input| {
            let input = from_utf8(input).context("Input file contains non-UTF-8 chars")?;
            BitVec::from_ascii_str(input)
                .context("Input file contains characters other than '0' or '1'")
        },
        InputFormat::AsciiLossy => unreachable!(),
    };

    let test_run_args = TestRunArgs::from_config(&config);

    let mut file = fs::File::open(&config.input_file).context("Failed to open input file")?;

    // Read only the necessary amount of bytes
    match config.max_length_or_split {
        MaxLengthOrSplit::MaxLength(max_length) => {
            let count_bytes = match config.input_format {
                InputFormat::Binary => max_length.get() / 8 + 1, // 8 Bits per Byte
                InputFormat::Ascii => max_length.get(),          // 1 Bit per Byte
                InputFormat::AsciiLossy => unreachable!(),
            };

            let mut input = vec![0; count_bytes];
            let res = file.read_exact(&mut input);

            if let Err(e) = res {
                if e.kind() == ErrorKind::UnexpectedEof {
                    // the file has fewer than count_bytes bytes,
                    // fill buffer with everything in the file
                    file.rewind()?;
                    input.clear();
                    file.read_to_end(&mut input)?;
                } else {
                    // another error (serious)
                    return Err(e.into());
                }
            }

            // convert to BitVec
            let mut input = converter(&input)?;

            // crop bits - read can only crop on a byte-level
            input.crop(max_length.get());

            // call test
            run_tests(&input, test_run_args, None)?;
        }
        MaxLengthOrSplit::Split(split_bytes) => {
            let split_bytes = match config.input_format {
                InputFormat::Binary => split_bytes.get(),
                // need 8 bytes of file data for 1 byte of binary data
                InputFormat::Ascii => split_bytes.get() * 8,
                InputFormat::AsciiLossy => unreachable!(),
            };

            let file_size = file.metadata()?.len();
            let count_parts = file_size / (split_bytes as u64);

            let mut i = 1_u64;
            // if all tests passed
            let mut passed = true;
            let mut input_bytes = vec![0; split_bytes];

            loop {
                let res = file.read_exact(&mut input_bytes);

                if let Err(e) = res {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        // the file has fewer than split_bytes bytes left --> regular exit
                        if passed {
                            println!("All tests passed");
                        } else {
                            println!("One or more tests failed / did not pass");
                        }

                        break;
                    } else {
                        // another error (serious)
                        return Err(e.into());
                    }
                }

                // convert to BitVec
                let input = converter(&input_bytes)?;

                // call test
                let parts = Some(Parts {
                    current: i,
                    count: count_parts,
                });
                if !run_tests(&input, test_run_args, parts)? {
                    passed = false;
                }

                // increment counter
                i += 1;
            }
        }
        MaxLengthOrSplit::None => {
            let mut input = Vec::new();
            file.read_to_end(&mut input)?;

            // convert to BitVec
            let input = converter(&input)?;

            // call test
            run_tests(&input, test_run_args, None)?;
        }
    }

    Ok(())
}

/// Handles input of type ASCII lossy
fn handle_ascii_lossy_input(config: ValidatedConfig) -> anyhow::Result<()> {
    let test_run_args = TestRunArgs::from_config(&config);

    // have to read everything - necessary length is not determinable
    let input = fs::read_to_string(&config.input_file).context("Failed to open input file")?;

    match config.max_length_or_split {
        MaxLengthOrSplit::MaxLength(max_length) => {
            let input = BitVec::from_ascii_str_lossy_with_max_length(&input, max_length.get());
            run_tests(&input, test_run_args, None)?;
        }
        MaxLengthOrSplit::Split(split_bytes) => {
            let split_bytes = split_bytes.get();

            // parse and convert back to bytes
            let full_input = BitVec::from_ascii_str_lossy(&input).to_bytes().0;
            let count_parts = (full_input.len() / split_bytes) as u64;

            let mut i = 1_usize;
            let mut passed = true;

            loop {
                // get the current byte list
                let Some(current) = full_input.get((i * split_bytes)..((i + 1) * split_bytes))
                else {
                    if passed {
                        println!("All tests passed");
                    } else {
                        println!("One or more tests failed / did not pass");
                    }

                    break;
                };

                // convert to BitVec
                let input = BitVec::from(current);

                // call test
                let parts = Some(Parts {
                    current: i as u64,
                    count: count_parts,
                });
                if !run_tests(&input, test_run_args, parts)? {
                    passed = false;
                }

                // increment counter
                i += 1;
            }
        }
        MaxLengthOrSplit::None => {
            let input = BitVec::from_ascii_str_lossy(&input);
            run_tests(&input, test_run_args, None)?;
        }
    }

    Ok(())
}

/// Run the specified tests on the specified BitVec, handle IO.
/// If a test index is given, this function behaves as if a file is split into subfiles and tested in
/// the same program execution.
///
/// Returns true if all tests passed, else false.
fn run_tests(input: &BitVec, args: TestRunArgs, parts: Option<Parts>) -> anyhow::Result<bool> {
    // calculate applicable tests
    let selected_tests = select_tests(args.tests_to_run, input);

    // Create CSV file, if necessary
    let mut csv_file = match args.csv_path {
        Some(path) => Some(create_csv_file(path, parts)?),
        None => None,
    };

    // Print the start info for this test runner.
    if let Some(parts) = parts{
        print!("{} / {} ", parts.current, parts.count);
    }
    println!("Running the selected tests: ");

    if args.console_output {
        print!("\t");
        selected_tests.iter().for_each(|test| print!("{test} "));
        println!();
        println!();
    }

    // Create runner - iterator is evaluated lazy - each test is only run, when .next() is called.
    let mut iter = test_runner::run_tests(&input, selected_tests.iter().copied(), args.test_args)?;

    // if all tests passed
    let mut passed = true;

    // use a manual loop to be able to time the test.
    loop {
        let begin = Instant::now();
        let Some((test, result)) = iter.next() else {
            if passed {
                println!("\tSummary: all tests passed");
            } else {
                println!("\tSummary: one or more tests failed / did not pass");
            }

            return Ok(passed);
        };
        let time = begin.elapsed();

        // print as csv
        if let Some(csv_file) = &mut csv_file {
            csv_file.write_test(test, time, result.as_ref())?;
        }

        // Print test results
        match result {
            Ok(res) => {
                // check if all tests passed
                if !res.iter().all(|r| r.passed(DEFAULT_THRESHOLD)) {
                    passed = false;
                }

                if args.console_output {
                    let time_as_ms = (time.as_micros() as f64) / 1000.0;

                    if res.len() == 1 {
                        print_test_result(format!("Test {test} ({}ms)", time_as_ms), res[0]);
                    } else {
                        println!("\tTest: {test} ({}ms): multiple Results", time_as_ms);
                        for (i, res) in res.into_iter().enumerate() {
                            print_test_result(format!("- Result {i}"), res);
                        }
                    }
                }
            }
            Err(e) => {
                passed = false;
                if args.console_output {
                    println!("\tTest {test}: ERROR: {e}")
                }
            }
        }
    }
}

/// Print a test result with a given start string
fn print_test_result(start_str: String, result: TestResult) {
    let passed = if result.passed(DEFAULT_THRESHOLD) {
        "PASSED"
    } else {
        "FAILED"
    };

    if let Some(comment) = result.comment() {
        println!(
            "\t{start_str}: {passed}. P-Value: {}. Comment: {}",
            result.p_value(),
            comment
        );
    } else {
        println!("\t{start_str}: {passed}. P-Value: {}", result.p_value());
    }
}

/// Create the [CsvFile] instance for the test output, based on the path and the idx (if given).
fn create_csv_file(csv_path: &Path, parts: Option<Parts>) -> anyhow::Result<CsvFile> {
    let file = match parts {
        Some(parts) => {
            if csv_path.file_name().is_none() {
                // Very wrong
                return Err(anyhow::anyhow!("Given output path contains no file name."));
            }

            if csv_path.try_exists()? && !csv_path.is_file() {
                // path exists, but is no file (i.e. dir)
                return Err(anyhow::anyhow!(
                    "Given output path already exists, but is no file."
                ));
            }

            let max_idx_len = format!("{}", parts.count).len();
            
            // create one file per idx - filename_{idx}.extension
            // create the filename with the _{idx} suffix and the extension
            let file_name = {
                let mut stem = csv_path
                    .file_stem()
                    .map(OsStr::to_os_string)
                    .unwrap_or_default();
                stem.push(format!("_{:0>1$}", parts.current, max_idx_len));
                if let Some(ext) = csv_path.extension() {
                    stem.push(".");
                    stem.push(ext);
                }
                stem
            };

            // create the full path
            CsvFile::new(csv_path.with_file_name(file_name))
        }
        None => CsvFile::new(csv_path),
    }?;

    Ok(file)
}

/// Select the tests to run
fn select_tests(tests_to_run: &TestsToRun, input: &BitVec) -> Vec<Test> {
    match tests_to_run {
        TestsToRun::AllowList(tests) => tests.clone(),
        t @ TestsToRun::BlockList(_) | t @ TestsToRun::All => {
            // all tests that are applicable based on the length
            let iter = Test::iter()
                .filter(|test| sts_lib::get_min_length_for_test(*test).get() <= input.len_bit());

            if let TestsToRun::BlockList(block_list) = t {
                iter.filter(|test| block_list.contains(test)).collect()
            } else {
                iter.collect()
            }
        }
    }
}
