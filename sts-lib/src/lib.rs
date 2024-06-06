#![doc = include_str!("../README.md")]

use thiserror::Error;

pub mod frequency_test;

/// Trait with the common methods of all result types
pub trait TestResult {
    /// The calculated p value
    fn p_value(&self) -> f64;

    /// If the sequence passed the test.
    /// `level_value` denotes the needed threshold, e.g. for a 1%-percent level it should be 0.01.
    fn passed(&self, level_value: f64) -> bool;
}

/// The common test result type, as used by most tests.
#[repr(transparent)]
pub struct CommonResult {
    p_value: f64,
}

impl TestResult for CommonResult {
    fn p_value(&self) -> f64 {
        self.p_value
    }

    fn passed(&self, level_value: f64) -> bool {
        self.p_value < level_value
    }
}

/// The error type for all tests
#[derive(Error, Debug)]
pub enum Error {
    /// A numeric overflow happened. The String gives further information on where exactly.
    #[error("Overflow in {0}.")]
    Overflow(String),
}
