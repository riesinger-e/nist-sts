#![doc = include_str!("../README.md")]

use thiserror::Error;

// internal usage only
pub(crate) mod internals;
#[cfg(test)]
mod unit_tests;

// public exports
pub mod bitvec;
pub mod test_runner;
pub mod tests;

// shared data structures

/// How many bits a byte has
const BYTE_SIZE: usize = 8;

/// List of all tests, used e.g. for automatic running.
#[repr(C)]
#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum Test {
    FrequencyTest,
    FrequencyTestWithinABlock,
}

/// The common test result type, as used by most tests.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct TestResult {
    p_value: f64,
}

impl TestResult {
    pub fn p_value(&self) -> f64 {
        self.p_value
    }

    pub fn passed(&self, level_value: f64) -> bool {
        self.p_value >= level_value
    }
}

/// The error type for all tests
#[derive(Error, Debug)]
pub enum Error {
    /// A numeric overflow happened. The String gives further information on where exactly.
    #[error("Overflow in {0}.")]
    Overflow(String),
    #[error("Result is not a number.")]
    NaN,
    #[error("Result is infinite.")]
    Infinite,
    #[error(transparent)]
    GammaFunctionFailed(#[from] statrs::StatsError),
}
