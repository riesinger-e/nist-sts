//! Script to convert a NIST Test file (found in the original NIST STS distribution, folder `data`)
//! to the file format used by the library.

use clap::Parser;
use std::fs;
use std::path::PathBuf;
use sts_lib::bitvec::BitVec;

/// Convert a NIST Test file, as found in the original implementation, folder `data` to the file
/// format used by the library.
#[derive(Parser, Debug)]
#[command(name = "convert_test_file", author, long_about=None)]
struct CmdLine {
    /// The input file to be converted.
    #[arg(short, long)]
    input_file: PathBuf,
    /// The output file to be converted.
    #[arg(short, long)]
    output_file: PathBuf,
    /// The length of the output. When not set, the complete input file is used.
    #[arg(short, long)]
    length: Option<usize>,
}

fn main() {
    let cmd_line = CmdLine::parse();

    let data = fs::read_to_string(&cmd_line.input_file).unwrap();

    let bitvec = if let Some(output_length) = cmd_line.length {
        BitVec::from_ascii_str_lossy_with_max_length(&data, output_length)
    } else {
        BitVec::from_ascii_str_lossy(&data)
    };

    // the remainder is not used
    let (data, _) = bitvec.to_bytes();

    fs::write(&cmd_line.output_file, data).unwrap();
}
