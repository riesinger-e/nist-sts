//! Non-overlapping Template Matching test.
//!
//! This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
//! This test uses an m-bit window to search for an m-bit pattern.
//!
//! This test allows for parameters, see [NonOverlappingTemplateTestArgs].

use std::io::BufReader;
use crate::bitvec::BitVec;
use crate::{Error, TestResult, BYTE_SIZE};
use rayon::prelude::*;

/// The arguments for the Non-overlapping Template Matching Test.
///
/// 1. The template length `m` to use: 2 <= `m` <= 21 - recommended: 10
///    Templates are chosen automatically.
/// 3. The number of independent blocks to test in the sequence: `N`
///    1 <= `N` < 100 - recommended: 8
///
/// These bounds are checked by all creation functions.
/// A default variant is available with [NonOverlappingTemplateTestArgs::default()].
#[repr(C)]
#[derive(Clone, Debug)]
pub struct NonOverlappingTemplateTestArgs {
    templates: Box<[BitVec]>,
    count_blocks: usize,
}

impl NonOverlappingTemplateTestArgs {
    /// Constructor with all arguments as normal values, evaluated at run time.
    pub fn new(template_len: usize, count_blocks: usize) -> Option<Self> {
        if !(2..=21).contains(&template_len) || !(1..100).contains(&count_blocks) {
            None
        } else {
            Some(Self::new_unchecked(template_len, count_blocks))
        }
    }

    /// Constructor with all arguments as const generics, which are asserted at compile time.
    pub fn new_const<const M: usize, const N: usize>() -> Self {
        const {
            assert!(2 <= M, "m must be >= 2");
            assert!(M <= 21, "m must be <= 21");
            assert!(1 <= N, "N must be >= 1");
            assert!(N < 100, "N must be < 100");
        }

        Self::new_unchecked(M, N)
    }

    /// Internal constructor: does not check the arguments, just panics if one is wrong.
    fn new_unchecked(template_len: usize, count_blocks: usize) -> Self {
        // the template files are embedded into the program, with information if compressed or not.
        const TEMPLATE_FILES: [(&[u8], bool); 20] = [
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template2")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template3")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template4")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template5")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template6")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template7")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template8")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template9")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template10")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template11")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template12")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template13")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template14")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template15")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template16")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template17")), false),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template18.xz")), true),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template19.xz")), true),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template20.xz")), true),
            (include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template21.xz")), true),
        ];

        // how long each template in the files is, in bytes.
        let template_len_bytes =
            if template_len % BYTE_SIZE == 0 { 0 } else { 1 } + template_len / BYTE_SIZE;

        // get the file content and if it is compressed
        let (file_content, compressed) = TEMPLATE_FILES[template_len - 2];
        // Create a buffer for decompression, but don't allocate anything yet.
        // This is needed for the buffer to live long enough to use later
        let mut decompressed = Vec::with_capacity(0);

        let template_raw = if compressed {
            // bufreader is necessary for xz_decompress
            let mut bufreader = BufReader::new(file_content);
            // if decompression does not wrong, something went seriously wrong with the files
            // embedded into the programs
            lzma_rs::xz_decompress(&mut bufreader, &mut decompressed).unwrap();
            &decompressed
        } else {
            // not compress, just use the file content
            file_content
        };

        let templates = template_raw
            // templates are stored as 1 big byte array, but the template length is known
            // and for not-filled bytes, 0-padding is used
            .chunks_exact(template_len_bytes)
            .map(|chunk| {
                let mut bitvec = BitVec::from(chunk);
                // remove padding
                bitvec.crop(template_len);
                bitvec
            })
            .collect::<Box<_>>();

        debug_assert_eq!(templates[0].len_bit(), template_len);

        Self {
            templates,
            count_blocks
        }
    }
}

impl Default for NonOverlappingTemplateTestArgs {
    /// The default parameters are the ones recommended by NIST.
    fn default() -> Self {
        Self::new_const::<10, 8>()
    }
}

pub fn non_overlapping_template_matching_test(
    data: &BitVec,
    test_arg: NonOverlappingTemplateTestArgs,
) -> Result<Vec<TestResult>, Error> {
    todo!()
}
