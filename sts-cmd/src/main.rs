use clap::Parser;
use std::fs;
use std::io::{ErrorKind, Read, Seek};
use std::process::ExitCode;
use std::str::from_utf8;
use std::time::Instant;
use sts_cmd::cmd_args::CmdArgs;
use sts_cmd::csv::CsvFile;
use sts_cmd::toml_config::TomlConfig;
use sts_cmd::valid_arg::{TestsToRun, ValidatedConfig};
use sts_cmd::InputFormat;
use sts_lib::bitvec::BitVec;
use sts_lib::{test_runner, IntoEnumIterator, Test, TestResult, DEFAULT_THRESHOLD};

/// Macro to try fallible operations and exit if it fails.
/// Supports custom literal messages that are printed at the start of the error message.
macro_rules! exit_on_error {
    ($try: expr, $msg: literal) => {
        match $try {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Aborting: {}: {}", $msg, e);
                return ExitCode::FAILURE;
            }
        }
    };
    ($try: expr) => {
        exit_on_error!($try, "An error occurred")
    };
    ($try: expr, $msg: literal, $($args: expr),+) => {
        match $try {
            Ok(val) => val,
            Err(e) => {
                let msg = format!($msg, $($args)+);
                eprintln!("{}: {}", msg, e);
                return ExitCode::FAILURE;
            }
        }
    }
}

/// Main function.
///
/// On success: prints the test results to stdout, exit code SUCCESS.
/// On error: prints the error to stderr, exit code FAILURE.
///
/// This program takes some arguments and an optional config file, use `--help`.
fn main() -> ExitCode {
    let CmdArgs {
        config_file,
        regular_args,
    } = CmdArgs::parse();

    let config = if let Some(config_file) = config_file {
        let toml = exit_on_error!(
            fs::read_to_string(&config_file),
            "Failed to read config file \"{}\"",
            config_file.display()
        );
        let toml_config: TomlConfig =
            exit_on_error!(toml::from_str(&toml), "Failed to parse the config file");
        exit_on_error!(ValidatedConfig::try_from_toml(toml_config, regular_args))
    } else {
        exit_on_error!(ValidatedConfig::try_from_cmd_args(regular_args))
    };

    println!("Reading input file: \"{}\"", config.input_file.display());

    let input: BitVec = match config.input_format {
        format @ InputFormat::Binary | format @ InputFormat::Ascii => {
            let mut file = exit_on_error!(
                fs::File::open(&config.input_file),
                "Failed to open input file"
            );

            // Read only the necessary amount of bytes
            let input = if let Some(max_length) = config.max_length {
                let count_bytes = max_length.get() / 8 + 1;
                let mut input = vec![0; count_bytes];
                let res = file.read_exact(&mut input);

                if let Err(e) = res {
                    if e.kind() == ErrorKind::UnexpectedEof {
                        // fill buffer with everything in the file
                        exit_on_error!(file.rewind(), "Failed to read input file");
                        input.clear();
                        exit_on_error!(file.read_to_end(&mut input), "Failed to read input file");
                    }
                }

                input
            } else {
                let mut input = Vec::new();
                exit_on_error!(file.read_to_end(&mut input), "Failed to read input file");
                input
            };

            // convert to BitVec
            let mut input = match format {
                InputFormat::Binary => BitVec::from(input),
                InputFormat::Ascii => {
                    let input =
                        exit_on_error!(from_utf8(&input), "Input file contains non-ASCII chars");
                    match BitVec::from_ascii_str(input) {
                        Some(vec) => vec,
                        None => {
                            eprintln!(
                                "Aborting: Input file contains characters other than '0' or '1'"
                            );
                            return ExitCode::FAILURE;
                        }
                    }
                }
                InputFormat::AsciiLossy => unreachable!(),
            };

            if let Some(max_length) = config.max_length {
                input.crop(max_length.get());
            }
            input
        }
        InputFormat::AsciiLossy => {
            // have to read everything - necessary length is not determinable
            let input = exit_on_error!(
                fs::read_to_string(config.input_file),
                "Failed to read input file"
            );

            match config.max_length {
                Some(max_length) => {
                    BitVec::from_ascii_str_lossy_with_max_length(&input, max_length.get())
                }
                None => BitVec::from_ascii_str_lossy(&input),
            }
        }
    };

    println!("Input file read!");
    println!();

    // Select tests
    let selected_tests = match config.tests_to_run {
        TestsToRun::AllowList(tests) => tests,
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
    };

    // Create CSV file, if necessary
    let mut csv_file = match config.output_path {
        Some(path) => Some(exit_on_error!(CsvFile::new(path))),
        None => None,
    };

    // Create runner and run tests
    println!("Running the selected tests: ");
    selected_tests.iter().for_each(|test| print!("{test} "));
    println!();
    println!();

    // iterator is evaluated lazy - each test is only run, when .next() is called.
    let mut iter = exit_on_error!(test_runner::run_tests(
        &input,
        selected_tests.iter().copied(),
        config.test_arguments
    ));

    // use a manual loop to be able to time the test.
    loop {
        let begin = Instant::now();
        let Some((test, result)) = iter.next() else {
            break;
        };
        let time = begin.elapsed();

        // print as csv
        if let Some(csv_file) = &mut csv_file {
            exit_on_error!(csv_file.write_test(test, time, result.as_ref()));
        }

        let time_as_ms = (time.as_micros() as f64) / 1000.0;

        match result {
            Ok(res) => {
                if res.len() == 1 {
                    print_test_result(format!("Test {test} ({}ms)", time_as_ms), res[0]);
                } else {
                    println!("Test: {test} ({}ms): multiple Results", time_as_ms);
                    for (i, res) in res.into_iter().enumerate() {
                        print_test_result(format!("- Result {i}"), res);
                    }
                }
            }
            Err(e) => println!("Test {test}: ERROR: {e}"),
        }
    }

    println!("Finished testing.");

    ExitCode::SUCCESS
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
            "{start_str}: {passed}. P-Value: {}. Comment: {}",
            result.p_value(),
            comment
        );
    } else {
        println!("{start_str}: {passed}. P-Value: {}", result.p_value());
    }
}
