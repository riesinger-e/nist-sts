//! Benchmarking application

#![cfg(unix)]

use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use std::{array, fs};
use sts_lib::bitvec::BitVec;
use sts_lib::test_runner::run_all_tests;
use sts_lib::tests::approximate_entropy::ApproximateEntropyTestArg;
use sts_lib::tests::frequency_block::FrequencyBlockTestArg;
use sts_lib::tests::linear_complexity::LinearComplexityTestArg;
use sts_lib::tests::serial::SerialTestArg;
use sts_lib::tests::template_matching::non_overlapping::NonOverlappingTemplateTestArgs;
use sts_lib::tests::template_matching::overlapping::OverlappingTemplateTestArgs;
use sts_lib::{Test, TestArgs};

// Count of test files
const COUNT_TEST_FILES: usize = 5;
// The amount of runs to do for each test file
const COUNT_RUNS_PER_FILE: usize = 100;

type StatisticStorage = HashMap<Test, (Vec<f64>, Vec<f64>)>;

/// Command line arguments
#[derive(Debug, Parser)]
#[command(version, author, about, long_about = None)]
struct CmdArgs {
    /// The path to the modified and built 'assess' binary.
    bin_path: PathBuf,
    /// The path to the directory containing the test files.
    /// From the repository root: 'sts-lib/test-files'.
    test_files_dir: PathBuf,
}

/// To deserialize the output of the reference implementation.
#[derive(Debug, Clone, Deserialize)]
struct ReferenceImpOutput {
    test: String,
    time: f64,
}

/// Map the name of the test, as printed by the modified reference implementation, to a Test.
fn map_c_name_to_test(name: String) -> Option<Test> {
    match name.as_ref() {
        "Frequency(tp.n)" => Some(Test::Frequency),
        "BlockFrequency(tp.blockFrequencyBlockLength, tp.n)" => Some(Test::FrequencyWithinABlock),
        "CumulativeSums(tp.n)" => Some(Test::CumulativeSums),
        "Runs(tp.n)" => Some(Test::Runs),
        "LongestRunOfOnes(tp.n)" => Some(Test::LongestRunOfOnes),
        "Rank(tp.n)" => Some(Test::BinaryMatrixRank),
        "DiscreteFourierTransform(tp.n)" => Some(Test::SpectralDft),
        "NonOverlappingTemplateMatchings(tp.nonOverlappingTemplateBlockLength, tp.n)" => {
            Some(Test::NonOverlappingTemplateMatching)
        }
        "OverlappingTemplateMatchings(tp.overlappingTemplateBlockLength, tp.n)" => {
            Some(Test::OverlappingTemplateMatching)
        }
        "Universal(tp.n)" => Some(Test::MaurersUniversalStatistical),
        "ApproximateEntropy(tp.approximateEntropyBlockLength, tp.n)" => {
            Some(Test::ApproximateEntropy)
        }
        "RandomExcursions(tp.n)" => Some(Test::RandomExcursions),
        "RandomExcursionsVariant(tp.n)" => Some(Test::RandomExcursionsVariant),
        "Serial(tp.serialBlockLength,tp.n)" => Some(Test::Serial),
        "LinearComplexity(tp.linearComplexitySequenceLength, tp.n)" => Some(Test::LinearComplexity),
        _ => None,
    }
}

/// Get the average of the given list
fn average(list: &[f64]) -> f64 {
    list.iter().sum::<f64>() / (list.len() as f64)
}

/// Print the given statistics
fn print_statistics(test: Test, rust_avg: f64, c_avg: f64) {
    println!("\tTest {test}");
    println!("\t\tAverage time of this implementation:          {rust_avg:.6} ms");
    println!("\t\tAverage time of the reference implementation: {c_avg:.6} ms");

    let diff = 100.0 * rust_avg / c_avg;
    let faster_or_slower = if diff <= 100.0 { "faster" } else { "SLOWER" };

    println!(
        "\t\t{faster_or_slower}: This implementation takes {:.2}% of the time of the reference implementation.",
        diff.abs(),
    );
}

/// Use the C implementation
fn test_c_imp(test_file: &Path, executable: &Path, statistics: &mut StatisticStorage) {
    let output = Command::new(executable)
        .args([test_file.as_os_str(), OsStr::new("1000000")])
        .current_dir(executable.parent().unwrap())
        .output()
        // just crash if there is a problem executing the reference implementation.
        .unwrap();
    if !output.status.success() {
        let msg = String::from_utf8_lossy(&output.stderr);
        panic!("Error when executing the reference implementation: {}", msg);
    }

    // each json entry is 1 line
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let result: ReferenceImpOutput = serde_json::from_str(line).unwrap();
        let test = map_c_name_to_test(result.test).unwrap();
        statistics
            .entry(test)
            .and_modify(|(_, c)| c.push(result.time))
            .or_insert_with(|| (Vec::new(), vec![result.time]));
    }
}

/// Use the Rust implementation
fn test_rust_imp(test_file: &Path, test_args: TestArgs, statistics: &mut StatisticStorage) {
    let raw_data = fs::read(test_file).unwrap();
    let data = BitVec::from(raw_data);
    if data.len_bit() != 1_000_000 {
        panic!(
            "Invalid test file length. Expected 1000000 bits. Got: {} bits. Aborting...",
            data.len_bit()
        );
    }
    let mut results = run_all_tests(data, test_args).unwrap();

    loop {
        let now = Instant::now();
        let Some((test, _)) = results.next() else {
            break;
        };
        let elapsed = now.elapsed();
        let time = (elapsed.as_nanos() as f64) / 1e6;
        statistics
            .entry(test)
            .and_modify(|(rust, _)| rust.push(time))
            .or_insert_with(|| (vec![time], Vec::new()));
    }
}

fn main() {
    // get command line arguments
    let args = CmdArgs::parse();

    // Build paths to the test files.
    // If the path cannot be canonicalized, something went very wrong...
    let test_files_dir = args.test_files_dir.canonicalize().unwrap();

    let test_files: [PathBuf; COUNT_TEST_FILES] = [
        test_files_dir.join("e.1e6.bin"),
        test_files_dir.join("pi.1e6.bin"),
        test_files_dir.join("sha1.1e6.bin"),
        test_files_dir.join("sqrt2.1e6.bin"),
        test_files_dir.join("sqrt3.1e6.bin"),
    ];

    // check existence of files
    for file in &test_files {
        if !file.exists() {
            panic!("Test file {} does not exist! Aborting..", file.display());
        }

        if !file.is_file() {
            panic!(
                "Test file {} is no regular file! Aborting...",
                file.display()
            );
        }
    }

    // check existence of binary
    let executable = args.bin_path;
    if !executable.exists() {
        panic!(
            "Executable {} does not exist! Aborting..",
            executable.display()
        );
    }
    if !executable.is_file() {
        panic!(
            "Executable {} is no regular file! Aborting...",
            executable.display()
        );
    }

    // test arguments for the rust version
    let test_args = TestArgs {
        frequency_block: FrequencyBlockTestArg::Bytewise(NonZero::new(16).unwrap()),
        non_overlapping_template: NonOverlappingTemplateTestArgs::new_const::<9, 8>(),
        overlapping_template: OverlappingTemplateTestArgs::new_nist_behaviour(9).unwrap(),
        linear_complexity: LinearComplexityTestArg::ManualBlockLength(NonZero::new(500).unwrap()),
        serial: SerialTestArg::new(16).unwrap(),
        approximate_entropy: ApproximateEntropyTestArg::new(10).unwrap(),
    };

    // data structures to store the statistics: (rust, c)
    let mut statistics: [StatisticStorage; COUNT_TEST_FILES] =
        array::from_fn(|_| Default::default());
    // will contain all calculated per-file averages
    let mut all_averages: StatisticStorage = HashMap::new();

    for (i, test_file) in test_files.iter().enumerate() {
        eprintln!("Testing {}...", test_file.display());

        let stats = &mut statistics[i];

        for j in 0..COUNT_RUNS_PER_FILE {
            // Rust attempt
            eprintln!(
                "\tAttempt {}/{COUNT_RUNS_PER_FILE} - This implementation",
                j + 1
            );
            test_rust_imp(test_file, test_args, stats);

            // C attempt
            eprintln!(
                "\tAttempt {}/{COUNT_RUNS_PER_FILE} - Reference implementation",
                j + 1
            );
            test_c_imp(test_file, &executable, stats);
        }

        // Print the statistics to stderr for separation
        println!("Statistics for test file {}:", test_file.display());

        // sort the results by the test
        let mut statistics = stats.iter().collect::<Vec<_>>();
        statistics.sort_unstable_by_key(|(test, _)| **test as u8);

        let averages = statistics
            .into_iter()
            .map(|(test, (rust, c))| (test, average(rust), average(c)));

        for (test, rust, c) in averages {
            print_statistics(*test, rust, c);

            all_averages
                .entry(*test)
                .and_modify(|(first, second)| {
                    first.push(rust);
                    second.push(c);
                })
                .or_insert_with(|| (vec![rust], vec![c]));
        }

        println!();
    }

    // calculate overall averages - again, sort by tests
    let mut all_averages = all_averages.into_iter().collect::<Vec<_>>();
    all_averages.sort_unstable_by_key(|(test, _)| *test as u8);
    let averages = all_averages.into_iter().map(|(test, (rust, c))| {
        // this works because the same amount of tests is executed for every file
        (test, average(&rust), average(&c))
    });

    println!("Overall statistics:");

    for (test, rust, c) in averages {
        print_statistics(test, rust, c);
    }
}
