//! Non-overlapping Template Matching test.
//!
//! This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
//! This test uses an m-bit window to search for an m-bit pattern.
//!
//! This test allows for parameters, see [NonOverlappingTemplateTestArgs].

use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::{Error, TestResult, BYTE_SIZE};
use rayon::prelude::*;
use std::cmp::Ordering;
use std::io::{BufReader};

/// The default block count. For use in [NonOverlappingTemplateTestArgs].
pub const DEFAULT_BLOCK_COUNT: usize = 8;
/// The default template length. For use in [NonOverlappingTemplateTestArgs].
pub const DEFAULT_TEMPLATE_LEN: usize = 10;

/// The arguments for the Non-overlapping Template Matching Test.
///
/// 1. The template length `m` to use: 2 <= `m` <= 21 - recommended: 10
///    Templates are chosen automatically. Unit is bits.
/// 3. The number of independent blocks to test in the sequence: `N`
///    1 <= `N` < 100 - recommended: 8
///
/// These bounds are checked by all creation functions.
/// A default variant is available with [NonOverlappingTemplateTestArgs::default()].
#[repr(C)]
#[derive(Clone, Debug)]
pub struct NonOverlappingTemplateTestArgs {
    templates: Box<[Vec<u8>]>,
    template_len: usize,
    count_blocks: usize,
}

impl NonOverlappingTemplateTestArgs {
    /// Constructor with all arguments as normal values, evaluated at run time.
    /// For the meaning of the arguments, see [NonOverlappingTemplateTestArgs].
    pub fn new(template_len: usize, count_blocks: usize) -> Option<Self> {
        if (2..=21).contains(&template_len) && (1..100).contains(&count_blocks) {
            Some(Self::new_unchecked(template_len, count_blocks))
        } else {
            None
        }
    }

    /// Constructor with all arguments as const generics, which are asserted at compile time.
    /// For the meaning of the arguments, see [NonOverlappingTemplateTestArgs].
    pub fn new_const<const M: usize, const N: usize>() -> Self {
        const {
            assert!(2 <= M, "m must be >= 2");
            assert!(M <= 21, "m must be <= 21");
            assert!(1 <= N, "N must be >= 1");
            assert!(N < 100, "N must be < 100");
        }

        Self::new_unchecked(M, N)
    }

    /// Constructor for custom templates - templates are checked for fitting length, if the length
    /// is not ok, `None` is returned.
    pub fn new_with_custom_templates(
        templates: Box<[Vec<u8>]>,
        template_len: usize,
        count_blocks: usize,
    ) -> Option<Self> {
        // Basic bounds check
        if !((2..=21).contains(&template_len) && (1..100).contains(&count_blocks)) {
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
                count_blocks,
            })
        } else {
            None
        }
    }

    /// Internal constructor: does not check the arguments, just panics if one is wrong.
    fn new_unchecked(template_len: usize, count_blocks: usize) -> Self {
        // the template files are embedded into the program, with information if compressed or not.
        const TEMPLATE_FILES: [(&[u8], bool); 20] = [
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template2")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template3")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template4")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template5")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template6")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template7")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template8")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template9")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template10")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template11")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template12")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template13")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template14")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template15")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template16")),
                false,
            ),
            (
                include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/template17")),
                false,
            ),
            (
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/templates/template18.xz"
                )),
                true,
            ),
            (
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/templates/template19.xz"
                )),
                true,
            ),
            (
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/templates/template20.xz"
                )),
                true,
            ),
            (
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/templates/template21.xz"
                )),
                true,
            ),
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
            .map(|chunk| chunk.into())
            .collect::<Box<_>>();

        Self {
            templates,
            template_len,
            count_blocks,
        }
    }
}

impl Default for NonOverlappingTemplateTestArgs {
    /// The default parameters are the ones recommended by NIST.
    fn default() -> Self {
        Self::new_const::<DEFAULT_TEMPLATE_LEN, DEFAULT_BLOCK_COUNT>()
    }
}

/// Non-overlapping template match test - No. 7
///
/// See the [module docs](crate::tests::non_overlapping_template_matching)
pub fn non_overlapping_template_matching_test(
    data: &BitVec,
    test_arg: NonOverlappingTemplateTestArgs,
) -> Result<Vec<TestResult>, Error> {
    // Step 0: calculate block length M
    let NonOverlappingTemplateTestArgs {
        templates,
        template_len,
        count_blocks,
    } = test_arg;
    let block_length_bit = data.len_bit() / count_blocks;

    // Create the last byte from the bit list
    let last_byte = data
        .remainder
        .iter()
        .enumerate()
        .fold(0_u8, |byte, (idx, &bit)| {
            if bit {
                byte | (1 << (BYTE_SIZE - idx - 1))
            } else {
                byte
            }
        });

    if block_length_bit < template_len {
        return Err(Error::InvalidParameter(
            format!("the calculated block length {block_length_bit} is smaller than the passed template length {template_len}!")
        ));
    }

    // Step 2: for each template B, calculate the number of times the template matches
    let count_matches_per_chunk_per_template = (0..count_blocks)
        .map(|block_idx| {
            // calculate the start byte and the bit position in the start byte for this block
            let total_start_bit =
                block_idx
                    .checked_mul(block_length_bit)
                    .ok_or(Error::Overflow(format!(
                        "multiplying {block_idx} by {block_length_bit}"
                    )))?;

            let start_byte = total_start_bit / BYTE_SIZE;
            let start_bit = total_start_bit % BYTE_SIZE;

            // calculate the max shifts
            let max_shifts = block_length_bit - (template_len - 1);

            // create the basic bitwise mask (allows only the bits that are the template)
            let (base_mask, base_mask_last_bit_index) =
                (create_mask(template_len), template_len % BYTE_SIZE);

            // create the base template stats
            let (base_templates, base_template_last_bit_index) =
                (&templates, template_len % BYTE_SIZE);

            // for each template, try to match
            let matches_per_template = base_templates
                .par_iter()
                .map(|base_template| {
                    // initialize the working bitwise mask - from the start bit position.
                    // This mask is bitwise shifted to the right position in the current stream.
                    let (mut mask, mut mask_last_bit_index) = {
                        let mut mask = base_mask.clone();
                        let last_bit_index =
                            right_shift_byte_vec(&mut mask, base_mask_last_bit_index, start_bit)
                                .unwrap();
                        (mask, last_bit_index)
                    };

                    // initialize the working template - from the start bit position.
                    // This template is bitwise shifted to the right position in the current stream.
                    let (mut template, mut template_last_bit_index) = {
                        let mut template = base_template.clone();
                        let last_bit_index = right_shift_byte_vec(
                            &mut template,
                            base_template_last_bit_index,
                            start_bit,
                        )
                        .unwrap();
                        (template, last_bit_index)
                    };

                    // go over the current chunk
                    let mut count_matches: usize = 0;

                    let mut i = 0;
                    while i < max_shifts {
                        // a match is:
                        // for every bit, apply bitwise AND with the current mask (which is shifted bitwise
                        // for new position) - now only the bits the template tries to match, are there.
                        let current_byte = (start_byte * BYTE_SIZE + i) / BYTE_SIZE;

                        let matched = (0..mask.len()).all(|idx| {
                            let byte = get_byte(&data.data, &last_byte, current_byte + idx);
                            byte & mask[idx] == template[idx]
                        });

                        // set the next shift necessary (if the template matched, the shift is for
                        // the template length), increment the counter if matched.
                        let shift = if matched {
                            // There are not enough matches possible to warrant checked arithmetic
                            count_matches += 1;
                            template_len
                        } else {
                            1
                        };

                        // Calculate the next mask and template.
                        // Use the current bit position to decide if the mask and template should be restarted
                        // from their base position.
                        if (i % BYTE_SIZE + shift + start_bit) / BYTE_SIZE == 0 {
                            // don't need to restart from base_*
                            mask_last_bit_index =
                                right_shift_byte_vec(&mut mask, mask_last_bit_index, shift)
                                    .unwrap();
                            template_last_bit_index =
                                right_shift_byte_vec(&mut template, template_last_bit_index, shift)
                                    .unwrap();
                        } else {
                            // We crossed a byte boundary - to avoid 0 bytes in the front, we restart
                            // with the base mask and template and shift only the difference (never
                            // a full byte).
                            let shift = (i + shift + start_bit) % BYTE_SIZE;

                            mask.clone_from(&base_mask);
                            mask_last_bit_index =
                                right_shift_byte_vec(&mut mask, base_mask_last_bit_index, shift)
                                    .unwrap();

                            template.clone_from(base_template);
                            template_last_bit_index = right_shift_byte_vec(
                                &mut template,
                                base_template_last_bit_index,
                                shift,
                            )
                            .unwrap();
                        }

                        // increment i - max_shifts cannot be big enough to warrant checked i
                        i += shift;
                    }

                    count_matches
                })
                .collect::<Box<_>>();
            Ok(matches_per_template)
        })
        .collect::<Result<Box<_>, Error>>()?;

    // Step 3: compute the theoretical mean and variance
    let power_2_template_len = f64::powi(2.0, template_len as i32);
    let mean = ((block_length_bit - template_len + 1) as f64) / power_2_template_len;
    let variance = (block_length_bit as f64)
        * (1.0 / power_2_template_len
        - (2.0 * (template_len as f64) - 1.0) / f64::powi(power_2_template_len, 2));

    // Step 4: for each template, compute chi = sum( (W_j - mean)^2 / variance ) ,
    // with W_j denoting the count of matches in the current block.
    // Step 5: for each template, compute p_value = igamc(count_blocks / 2, chi / 2)
    let p_values = (0..templates.len())
        .into_par_iter()
        .map(|template_idx| {
            let chi = count_matches_per_chunk_per_template
                .iter()
                .map(|matches_per_template| {
                    let matches = matches_per_template[template_idx];
                    f64::powi((matches as f64) - mean, 2) / variance
                })
                .sum::<f64>();

            check_f64(chi)?;

            let p_value = igamc((count_blocks as f64) / 2.0, chi / 2.0)?;
            check_f64(p_value)?;

            Ok(TestResult::new(p_value))
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(p_values)
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
