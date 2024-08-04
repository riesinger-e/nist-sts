//! Non-overlapping Template Matching test.
//!
//! This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
//! This test uses an m-bit window to search for an m-bit pattern.
//!
//! This test allows for parameters, see [NonOverlappingTemplateTestArgs].

use super::{DEFAULT_TEMPLATE_LEN, TemplateArg};
use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::{Error, TestResult};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

// calculation: min block count (1) * min template length (2)
/// The minimum input length, in bits, for this test.
pub const MIN_INPUT_LENGTH: usize = 2;


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
        Self::new_const::<{ super::DEFAULT_TEMPLATE_LEN }, DEFAULT_BLOCK_COUNT>()
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
    let count_matches_per_chunk_per_template = super::count_matches_per_chunk_per_template(
        count_blocks,
        block_length_bit,
        data,
        templates,
        template_len,
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
