//! Non-overlapping Template Matching test.
//!
//! This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
//! This test uses an m-bit window to search for an m-bit pattern.
//!
//! This test allows for parameters, see [NonOverlappingTemplateTestArgs].

use std::num::NonZero;

use rayon::prelude::*;

use crate::{Error, TestResult, BYTE_SIZE};
use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use super::{create_mask, get_byte, right_shift_byte_vec, TemplateArg};

// calculation: min block count (1) * min template length (2)
/// The minimum input length, in bits, for this test.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const { 
    match NonZero::new(2) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};


/// The default block count. For use in [NonOverlappingTemplateTestArgs].
pub const DEFAULT_BLOCK_COUNT: usize = 8;

/// The arguments for the Non-overlapping Template Matching Test.
///
/// 1. The templates, see [TemplateArg];
/// 2. The number of independent blocks to test in the sequence: `N`
///    1 <= `N` < 100 - recommended: 8
///
/// These bounds are checked by all creation functions.
/// A default variant is available with [NonOverlappingTemplateTestArgs::default()].
#[derive(Copy, Clone, Debug)]
pub struct NonOverlappingTemplateTestArgs<'a> {
    templates: TemplateArg<'a>,
    count_blocks: usize,
}

impl NonOverlappingTemplateTestArgs<'static> {
    /// Constructor with all arguments as normal values, evaluated at run time.
    /// For the meaning of the arguments, see [NonOverlappingTemplateTestArgs].
    pub fn new(template_len: usize, count_blocks: usize) -> Option<Self> {
        if (1..100).contains(&count_blocks) {
            Some(Self {
                templates: TemplateArg::new(template_len)?,
                count_blocks,
            })
        } else {
            None
        }
    }

    /// Constructor with all arguments as const generics, which are asserted at compile time.
    /// For the meaning of the arguments, see [NonOverlappingTemplateTestArgs].
    pub fn new_const<const M: usize, const N: usize>() -> Self {
        const {
            assert!(1 <= N, "N must be >= 1");
            assert!(N < 100, "N must be < 100");
        }

        Self {
            templates: TemplateArg::new_const::<M>(),
            count_blocks: N,
        }
    }
}

impl<'a> NonOverlappingTemplateTestArgs<'a> {
    pub fn new_with_custom_template(
        templates: TemplateArg<'a>,
        count_blocks: usize,
    ) -> Option<Self> {
        if (1..100).contains(&count_blocks) {
            Some(Self {
                templates,
                count_blocks,
            })
        } else {
            None
        }
    }
}

impl Default for NonOverlappingTemplateTestArgs<'static> {
    /// The default parameters are the ones recommended by NIST.
    fn default() -> Self {
        Self::new_const::<{ super::DEFAULT_TEMPLATE_LENGTH }, DEFAULT_BLOCK_COUNT>()
    }
}

/// Non-overlapping template match test - No. 7
///
/// See the [module docs](crate::tests::template_matching::non_overlapping)
pub fn non_overlapping_template_matching_test(
    data: &BitVec,
    test_arg: NonOverlappingTemplateTestArgs,
) -> Result<Vec<TestResult>, Error> {
    // Step 0: calculate block length M
    let NonOverlappingTemplateTestArgs {
        templates,
        count_blocks,
    } = test_arg;

    let block_length_bit = data.len_bit() / count_blocks;
    let template_len = templates.template_len;

    if block_length_bit < template_len {
        return Err(Error::InvalidParameter(
            format!("the calculated block length {block_length_bit} is smaller than the passed template length {template_len}!")
        ));
    }

    // Step 2: for each template B, calculate the number of times the template matches
    let count_matches_per_chunk_per_template = count_matches_per_chunk_per_template(
        count_blocks,
        block_length_bit,
        data,
        templates,
    )
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
    let p_values = (0..templates.templates.len())
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

/// Count the matches per chunk and template.
fn count_matches_per_chunk_per_template<'a>(
    block_count: usize,
    block_length_bit: usize,
    data: &'a BitVec,
    templates: TemplateArg<'a>,
) -> impl Iterator<Item = Result<Box<[usize]>, Error>> + 'a {
    let TemplateArg {
        templates,
        template_len,
    } = templates;

    // Create the last byte from the bit list
    let last_byte = data.get_last_byte();

    // For each block, calculate the times each template matches.
    (0..block_count).map(move |block_idx| {
        // calculate the start byte and the bit position in the start byte for this block
        let total_start_bit = block_idx
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
        let (base_templates, base_template_last_bit_index) = (templates, template_len % BYTE_SIZE);

        // for each template, try to match
        let matches_per_template = base_templates
            .par_iter()
            .map(|&base_template| {
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
                    let mut template = Vec::from(base_template);
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
                            right_shift_byte_vec(&mut mask, mask_last_bit_index, shift).unwrap();
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

                        template = Vec::from(base_template);
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
}