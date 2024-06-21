//! Script to convert NIST template files for template matching to the more space-efficient format
//! used by the library.
//!
//! For the format description, see the *README.md* in *sts-lib/templates*.

use std::{fs, io};
use std::fs::{OpenOptions};
use std::io::{BufRead, BufReader, Seek};
use std::path::PathBuf;
use clap::Parser;
use xz2::write::XzEncoder;
use regex::Regex;
use sts_lib::bitvec::BitVec;

// how many bits a byte has
const BYTE_SIZE: usize = 8;
// the file size threshold of 200 KiB, files over this threshold should be compressed
const THRESHOLD: usize = 200 * 1024;
// compression level: the default of xz
const COMPRESSION_LEVEL: u32 = 6;

/// Convert a NIST Test file, as found in the original implementation, folder `templates` to the file
/// format used by the library. Files over 200 KiB will be compressed using xz.
#[derive(Parser, Debug)]
#[command(name = "convert_templates", author, long_about=None)]
struct CmdLine {
    /// The input directory to be converted.
    #[arg(short, long)]
    input_dir: PathBuf,
    /// The output directory to save the converted files.
    #[arg(short, long)]
    output_dir: PathBuf,
}


fn main() {
    let cmd_line = CmdLine::parse();

    let file_name_regex = Regex::new(r"^template\d+$").unwrap();

    for entry in fs::read_dir(&cmd_line.input_dir).unwrap() {
        let entry = entry.unwrap();

        let path = entry.path();
        let file_name = entry.file_name().into_string().unwrap();

        // Validate that path points to a file
        if !path.is_file() {
            continue;
        }

        // Validate the filename - there may be other files that are not templates
        if !file_name_regex.is_match(&file_name) {
            continue;
        }

        // to store the output
        let mut output = Vec::new();

        // read the file, each line is a template
        let input_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let bufreader = BufReader::new(input_file);

        for line in bufreader.lines() {
            let line = line.unwrap();

            let bitvec = BitVec::from_ascii_str_lossy(&line);
            let (data, remainder) = bitvec.into_parts();

            output.extend_from_slice(&data);

            // create a full byte for the remainder
            if !remainder.is_empty() {
                let byte = remainder.iter().enumerate()
                    // get the indexes where the bit is 1
                    .filter_map(|(idx, &bit)| bit.then_some(idx))
                    .fold(0, |byte, idx| {
                        // use the index where the bit is one to shift a 1 to the correct position
                        byte | (1 << (BYTE_SIZE - idx - 1))
                    });
                output.push(byte)
            }
        }

        // open the destination and write the output to it.
        if output.len() < THRESHOLD {
            // need not be compressed
            let output_file = cmd_line.output_dir.join(file_name);
            fs::write(output_file, output).unwrap();
        } else {
            // open the output file
            let output_file = cmd_line.output_dir.join(format!("{file_name}.xz"));
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(output_file)
                .unwrap();
            // create compressor writing to the file - use the c-library liblzma for compression,
            // use the rust library lzma-rs for decompression (no external dependencies)
            let mut compressor = XzEncoder::new(&file, COMPRESSION_LEVEL);
            // copy the output into the compressor, which writes the compressed bytes to the file
            io::copy(&mut output.as_slice(), &mut compressor).unwrap();
            // finalize the compression
            compressor.finish().unwrap();

            // Test the decompression
            file.rewind().unwrap();
            // buffering is necessary
            let mut bufreader = BufReader::new(file);

            // to store the decompressed file
            let mut decompressed = Vec::with_capacity(output.len());
            lzma_rs::xz_decompress(&mut bufreader, &mut decompressed).unwrap();

            if decompressed != output {
                panic!("Decompression does not work!");
            }
        }
    }
}