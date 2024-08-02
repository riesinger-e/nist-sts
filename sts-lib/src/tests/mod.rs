//! All NIST STS tests. See the module documentation for details about each test.

pub mod binary_matrix_rank;
pub mod frequency;
pub mod frequency_block;
pub mod linear_complexity;
pub mod longest_run_of_ones;
pub mod maurers_universal_statistical;
pub mod runs;
pub mod spectral_dft;
pub mod template_matching;
// The approximate entropy test and the serial test share some code.
// This module contains them both, for API consistency, both modules are re-exported as if they
// were defined in this module.
mod serial_and_approximate_entropy;
pub use serial_and_approximate_entropy::{serial, approximate_entropy};
pub mod cumulative_sums;
pub mod random_excursions;
pub mod random_excursions_variant;