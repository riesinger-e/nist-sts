//! Constants are problematic with `cbindgen`: they need to be defined as literals.
//!
//! This build script just creates a file with all necessary constants written as literals, with the
//! values coming directly from the sts_lib crate. By doing this, the constants are only defined in
//! one place and need not be manually updated. The file is included in the `constants.rs` module.

use std::path::PathBuf;
use std::{env, fs};
use sts_lib::tests::template_matching::overlapping::{
    DEFAULT_BLOCK_LENGTH as OV_DEFAULT_BLOCK_LENGTH, DEFAULT_FREEDOM as OV_DEFAULT_FREEDOM,
    DEFAULT_TEMPLATE_LENGTH as OV_DEFAULT_TEMPLATE_LENGTH,
};
use sts_lib::tests::template_matching::{
    non_overlapping::DEFAULT_BLOCK_COUNT as NOV_DEFAULT_BLOCK_COUNT,
    DEFAULT_TEMPLATE_LENGTH as NOV_DEFAULT_TEMPLATE_LENGTH,
};
use sts_lib::{EnumCount, DEFAULT_THRESHOLD};

fn main() {
    // Build script needs to be rerun if the sts-lib crate, where the constants come from, changes.
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_crate_dir = manifest_dir.join("../sts-lib/src");
    assert!(source_crate_dir.exists());
    println!("cargo:rerun-if-changed={}", source_crate_dir.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_path = out_dir.join("constants.rs");

    let test_count = sts_lib::Test::COUNT;

    // create the constants as literals.
    let file_content = format!(
        r#"
/// The default length of each block M, in bits, for use in the Overlapping Template Matching Test.
pub const OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_LENGTH: usize = {OV_DEFAULT_BLOCK_LENGTH};

/// The default degree of freedom K for use in the Overlapping Template Matching Test.
pub const OVERLAPPING_TEMPLATE_DEFAULT_FREEDOM: usize = {OV_DEFAULT_FREEDOM};

/// The default template length use in the Overlapping Template Matching Test.
pub const OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LENGTH: usize = {OV_DEFAULT_TEMPLATE_LENGTH};

/// The default block count to use in the Non-overlapping Template Matching Test.
pub const NON_OVERLAPPING_TEMPLATE_DEFAULT_BLOCK_COUNT: usize = {NOV_DEFAULT_BLOCK_COUNT};

/// The default template length to use in the Non-overlapping Template Matching Test.
pub const NON_OVERLAPPING_TEMPLATE_DEFAULT_TEMPLATE_LENGTH: usize = {NOV_DEFAULT_TEMPLATE_LENGTH};

/// The default threshold for determining if a test passes its criteria.
pub const DEFAULT_THRESHOLD: f64 = {DEFAULT_THRESHOLD};

/// The count of tests. The first test has a numerical value of 0 and the last test of test_count - 1
pub const TEST_COUNT: usize = {test_count};
    "#
    );

    // write the file
    fs::write(&out_path, file_content).unwrap();
}
