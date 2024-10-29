//! Everything needed to save CSV results.

use core::error::Error;
use csv::WriterBuilder;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use sts_lib::{Test, TestResult, DEFAULT_THRESHOLD};

/// Error type for [CsvFile]
#[derive(Debug)]
pub enum CsvFileError {
    Io(std::io::Error),
    Csv(csv::Error),
}

impl Display for CsvFileError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CsvFileError::Io(e) => write!(f, "IO error: {e}"),
            CsvFileError::Csv(e) => write!(f, "CSV error: {e}"),
        }
    }
}

impl Error for CsvFileError {}

impl From<std::io::Error> for CsvFileError {
    fn from(value: std::io::Error) -> Self {
        CsvFileError::Io(value)
    }
}

impl From<csv::Error> for CsvFileError {
    fn from(value: csv::Error) -> Self {
        CsvFileError::Csv(value)
    }
}

/// This struct represents a CSV file to write the test outputs.
#[derive(Debug)]
pub struct CsvFile(csv::Writer<File>);

impl CsvFile {
    /// Create a new CSV File writer writing to the specified path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, CsvFileError> {
        let mut builder = WriterBuilder::new();

        builder.delimiter(b';').has_headers(true);

        // target specific: on windows, lines should end with CRLF, on all other platforms, the default
        // LF is enough.
        #[cfg(target_family = "windows")]
        {
            use csv::Terminator;

            builder.terminator(Terminator::CRLF);
        }

        Ok(Self(builder.from_path(path)?))
    }

    /// Append the given test results to the CSV file.
    pub fn write_test<S: AsRef<[TestResult]>>(
        &mut self,
        test: Test,
        time: Duration,
        results: Result<S, &sts_lib::Error>,
    ) -> Result<(), CsvFileError> {
        // CSV format: test name; time in ms; result no.; PASS/FAIL; P-Value; comment
        let test = test.to_string();
        let time = (time.as_micros() as f64) / 1000.0;

        // struct to use for CSV
        #[derive(Serialize)]
        struct CsvFormat<'a> {
            #[serde(rename = "test name")]
            test: &'a str,
            #[serde(rename = "time in ms")]
            time: f64,
            #[serde(rename = "result no")]
            result_no: usize,
            #[serde(rename = "PASS/FAIL")]
            pass_fail: &'static str,
            #[serde(rename = "p-value")]
            p_value: f64,
            #[serde(rename = "comment")]
            comment: &'a str,
        }

        match results {
            Ok(results) => {
                // Serialization of successful results.
                for (no, result) in results.as_ref().iter().enumerate() {
                    let pass = if result.passed(DEFAULT_THRESHOLD) {
                        "PASS"
                    } else {
                        "FAIL"
                    };

                    let row = CsvFormat {
                        test: &test,
                        time,
                        result_no: no,
                        pass_fail: pass,
                        p_value: result.p_value(),
                        comment: result.comment().unwrap_or(""),
                    };

                    self.0.serialize(row)?;
                }
            }
            Err(e) => {
                // Serialization of errors
                let err = e.to_string();
                let row = CsvFormat {
                    test: &test,
                    time,
                    result_no: 0,
                    pass_fail: "ERROR",
                    p_value: -1.0,
                    comment: &err,
                };

                self.0.serialize(row)?;
            }
        }

        self.0.flush()?;
        Ok(())
    }
}
