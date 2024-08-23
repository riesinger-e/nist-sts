//! Everything necessary for the [Non-Overlapping](non_overlapping) and [Overlapping](overlapping)
//! template matching tests. For the tests themselves, see the corresponding submodules.
//!
//! This module also contains the template argument used by both tests.

pub mod non_overlapping;
pub mod overlapping;

use crate::BYTE_SIZE;
use std::cmp::Ordering;
use std::io::BufReader;
use std::sync::LazyLock;

/// The default template length. For use in [TemplateArg].
pub const DEFAULT_TEMPLATE_LEN: usize = 9;

/// This argument contains the template to use.
///
/// For the template length `m`, the following bounds must be met: 2 <= `m` <= 21 - recommended: 9.
/// Templates are chosen automatically. Unit is bits.
///
/// These bounds are checked by all creation functions.
/// A default variant is available with [TemplateArg::default()].
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct TemplateArg<'a> {
    templates: &'a [&'a [u8]],
    template_len: usize,
}

impl TemplateArg<'static> {
    /// Constructor with the template length as normal values, evaluated at run time.
    /// See [TemplateArg].
    pub fn new(template_len: usize) -> Option<Self> {
        if (2..=21).contains(&template_len) {
            Some(Self::new_unchecked(template_len))
        } else {
            None
        }
    }

    /// Constructor with the template length as const generic, which is asserted at compile time.
    /// See [TemplateArg].
    pub fn new_const<const M: usize>() -> Self {
        const {
            assert!(2 <= M, "m must be >= 2");
            assert!(M <= 21, "m must be <= 21");
        }

        Self::new_unchecked(M)
    }

    /// Internal constructor: does not check the arguments, just panics if one is wrong.
    fn new_unchecked(template_len: usize) -> Self {
        // the template files are embedded into the program.
        const UNCOMPRESSED_TEMPLATE_FILES: [&[u8]; 16] = [
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template2")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template3")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template4")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template5")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template6")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template7")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template8")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template9")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template10")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template11")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template12")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template13")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template14")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template15")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template16")),
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template17")),
        ];

        // Compressed files are decompressed at run-time.
        const COMPRESSED_TEMPLATE_FILES: [&[u8]; 4] = [
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/templates/template18.xz"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/templates/template19.xz"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/templates/template20.xz"
            )),
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/templates/template21.xz"
            )),
        ];

        // decompression and splitting is lazily done on first access
        static DECOMPRESSED_TEMPLATE_FILES: [LazyLock<Box<[u8]>>; 4] = [
            LazyLock::new(|| decompress_template_file(COMPRESSED_TEMPLATE_FILES[0])),
            LazyLock::new(|| decompress_template_file(COMPRESSED_TEMPLATE_FILES[1])),
            LazyLock::new(|| decompress_template_file(COMPRESSED_TEMPLATE_FILES[2])),
            LazyLock::new(|| decompress_template_file(COMPRESSED_TEMPLATE_FILES[3])),
        ];

        // The split references are stored for reuse later.
        // Again: LazyLock creation so that this is not done on startup.
        static TEMPLATES: [LazyLock<Box<[&[u8]]>>; 20] = [
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[0], 2)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[1], 3)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[2], 4)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[3], 5)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[4], 6)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[5], 7)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[6], 8)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[7], 9)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[8], 10)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[9], 11)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[10], 12)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[11], 13)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[12], 14)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[13], 15)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[14], 16)),
            LazyLock::new(|| split_template_file(UNCOMPRESSED_TEMPLATE_FILES[15], 17)),
            LazyLock::new(|| split_template_file(DECOMPRESSED_TEMPLATE_FILES[0].as_ref(), 18)),
            LazyLock::new(|| split_template_file(DECOMPRESSED_TEMPLATE_FILES[1].as_ref(), 19)),
            LazyLock::new(|| split_template_file(DECOMPRESSED_TEMPLATE_FILES[2].as_ref(), 20)),
            LazyLock::new(|| split_template_file(DECOMPRESSED_TEMPLATE_FILES[3].as_ref(), 21)),
        ];

        // this call decompresses, if necessary, then splits the file into the individual templates
        // and saves the references.
        let templates = TEMPLATES[template_len - 2].as_ref();

        Self {
            templates,
            template_len,
        }
    }
}

impl<'a> TemplateArg<'a> {
    /// Constructor for custom templates - templates are checked for fitting length, if the length
    /// is not ok, `None` is returned.
    pub fn new_with_custom_templates(
        templates: &'a [&'a [u8]],
        template_len: usize,
    ) -> Option<Self> {
        // Basic bounds check
        if !(2..=21).contains(&template_len) {
            return None;
        }

        // calculate template length in bytes
        let template_len_bytes =
            if template_len % BYTE_SIZE == 0 { 0 } else { 1 } + template_len / BYTE_SIZE;

        let all_templates_have_right_len = templates
            .iter()
            .all(|template| template.len() == template_len_bytes);

        if all_templates_have_right_len {
            Some(Self {
                templates,
                template_len,
            })
        } else {
            None
        }
    }
}

impl Default for TemplateArg<'static> {
    /// The default parameters are the ones recommended by NIST.
    fn default() -> Self {
        Self::new_const::<DEFAULT_TEMPLATE_LEN>()
    }
}

/// Decompress a compressed template file.
fn decompress_template_file(compressed: &[u8]) -> Box<[u8]> {
    // bufreader is necessary for xz_decompress
    let mut bufreader = BufReader::new(compressed);
    let mut decompressed = Vec::new();
    // if decompression does not work, something went seriously wrong with the files
    // embedded into the program.
    lzma_rs::xz_decompress(&mut bufreader, &mut decompressed).unwrap();
    decompressed.into_boxed_slice()
}

/// Split a (decompressed) template file.
/// Argument: the template length in bits.
fn split_template_file(template_raw: &[u8], template_len: usize) -> Box<[&[u8]]> {
    // how long each template in the files is, in bytes.
    let template_len_bytes =
        if template_len % BYTE_SIZE == 0 { 0 } else { 1 } + template_len / BYTE_SIZE;

    template_raw
        // templates are stored as 1 big byte array, but the template length is known
        // and for not-filled bytes, 0-padding is used
        .chunks_exact(template_len_bytes)
        .collect::<Box<_>>()
}

/// Right shift the individual bits in the [Vec], carrying over bits to the next element and extending
/// as necessary. `last_bit_index` is the index of the last bit in the last byte.
/// A new last_bit_index is returned. The maximum allowed shift ist 7. The given Vec must have
/// a maximum of 7 elements.
#[must_use]
fn right_shift_byte_vec(
    bytes: &mut Vec<u8>,
    mut last_bit_index: usize,
    shift: usize,
) -> Option<usize> {
    if shift >= BYTE_SIZE || bytes.len() > 7 {
        return None;
    }

    // no-op
    if shift == 0 {
        // unchanged
        return Some(last_bit_index);
    }

    // at maximum, one byte has to be added to the end
    if last_bit_index + shift >= BYTE_SIZE {
        bytes.push(0)
    }
    // write new last bit index
    last_bit_index = (last_bit_index + shift) % BYTE_SIZE;

    // reinterpret as an appropriate number (Big Endian) and shift
    match bytes.len() {
        0 => (), //noop
        1 => bytes[0] >>= shift,
        2 => {
            let value = u16::from_be_bytes(bytes.as_slice().try_into().unwrap()) >> shift;
            *bytes = value.to_be_bytes().into();
        }
        3..=4 => {
            let prev_length = bytes.len();
            for _ in prev_length..4 {
                bytes.push(0);
            }
            let value = u32::from_be_bytes(bytes.as_slice().try_into().unwrap()) >> shift;
            *bytes = value.to_be_bytes().into();
            bytes.truncate(prev_length);
        }
        5..=8 => {
            let prev_length = bytes.len();
            for _ in prev_length..8 {
                bytes.push(0);
            }
            let value = u64::from_be_bytes(bytes.as_slice().try_into().unwrap()) >> shift;
            *bytes = value.to_be_bytes().into();
            bytes.truncate(prev_length);
        }
        _ => unreachable!(),
    }

    Some(last_bit_index)
}

/// Take the template length and create a bitmask to compare if the template matches
fn create_mask(template_bit_len: usize) -> Vec<u8> {
    // Count of bytes that should consist onf only "1" bits
    let one_bytes = template_bit_len / BYTE_SIZE;

    let mut mask = vec![0xff; one_bytes];

    mask.push(match template_bit_len % BYTE_SIZE {
        // early return - no additional byte is needed
        0 => return mask,
        1 => 0b1000_0000,
        2 => 0b1100_0000,
        3 => 0b1110_0000,
        4 => 0b1111_0000,
        5 => 0b1111_1000,
        6 => 0b1111_1100,
        7 => 0b1111_1110,
        _ => unreachable!(),
    });
    mask
}

/// Access the specified index, with the additional byte seen as the last index.
/// Panics if the index is out of bounds.
fn get_byte(data: &[u8], byte: &u8, idx: usize) -> u8 {
    match idx.cmp(&data.len()) {
        Ordering::Less => data[idx],
        Ordering::Equal => *byte,
        _ => panic!("get_byte(): idx {idx} is out of bounds!"),
    }
}
