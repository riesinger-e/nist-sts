//! Overlapping Template Matching test.
//!
//! This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
//! This test uses an m-bit window to search for an m-bit pattern.
//! The big difference to the [non-overlapping](super::non_overlapping) test is that template matches
//! may overlap.
//!
//! The default arguments for this test derivate significantly from the NIST reference implementation,
//! since the NIST reference implementation for this test is known bad.
//! The problem is that the PI values from NIST are wrong - the correction from Hamano and Kaneko is used.
//!
//! Details about the problems:
//! * Even though the pi values should be revised according to the paper, both the example and
//!   the implementation still use the old, inaccurate calculation.
//! * The (not working) fixed values according to Hamano and Kaneko only work for very specific cases.
//!
//! The PI values from NIST can still be used for testing purposes by using
//! [OverlappingTemplateTestArgs::new_nist_behaviour].
//!
//! This test needs arguments, see [OverlappingTemplateTestArgs].

use crate::bitvec::BitVec;
use crate::internals::igamc;
use crate::tests::template_matching::{create_mask, overflowing_right_shift};
use crate::{Error, TestResult};
use bigdecimal::num_bigint::BigInt;
use bigdecimal::num_traits::ToPrimitive;
use bigdecimal::BigDecimal;
use rayon::prelude::*;
use std::collections::HashMap;
use std::num::NonZero;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex};
use sts_lib_derive::use_thread_pool;

// calculation: min template length (2) * min block length (4)
/// The minimum input length, in bits, for this test.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(2 * 4) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

/// The default length of each block M, in bits.
pub const DEFAULT_BLOCK_LENGTH: usize = 1032;

/// The default degree of freedom K.
pub const DEFAULT_FREEDOM: usize = 6;

/// The default template length.
pub const DEFAULT_TEMPLATE_LENGTH: usize = 9;

/// The arguments for the Overlapping Template Matching Test.
///
/// 1. The template length *m*. 2 <= *m* <= 21.
/// 2. The length of each block, *M*, in bits. See [DEFAULT_BLOCK_LENGTH]
/// 3. The degrees of freedom, *K*. See [DEFAULT_FREEDOM].
///
/// With these arguments the *pi* values are calculated according to Hamano and Kaneko.
/// These bounds are checked by all creation functions.
/// A default variant is available with [OverlappingTemplateTestArgs::default()].
///
/// To replicate the exact NIST behaviour, use [OverlappingTemplateTestArgs::new_nist_behaviour]
#[derive(Copy, Clone, Debug)]
pub struct OverlappingTemplateTestArgs {
    template_length: usize,
    block_length: usize,
    freedom: usize,
    inaccurate_nist_calculation: bool,
}

impl OverlappingTemplateTestArgs {
    /// Create new arguments. For the meanings and allowed value ranges, see [OverlappingTemplateTestArgs].
    pub fn new(template_length: usize, block_length: usize, freedom: usize) -> Option<Self> {
        if (2..=21).contains(&template_length) {
            Some(Self {
                template_length,
                block_length,
                freedom,
                inaccurate_nist_calculation: false,
            })
        } else {
            None
        }
    }

    /// Force the inaccurate behaviour of the reference implementation.
    /// Template length may only be 9 or 10 here.
    ///
    /// The chosen variables are only accurate for bit lengths of 10^6.
    pub fn new_nist_behaviour(template_length: usize) -> Option<Self> {
        if template_length == 9 || template_length == 10 {
            Some(Self {
                template_length,
                block_length: 1032,
                freedom: 6,
                inaccurate_nist_calculation: true,
            })
        } else {
            None
        }
    }
}

impl Default for OverlappingTemplateTestArgs {
    /// Default values, see the module constants
    fn default() -> Self {
        Self {
            template_length: DEFAULT_TEMPLATE_LENGTH,
            block_length: DEFAULT_BLOCK_LENGTH,
            freedom: DEFAULT_FREEDOM,
            inaccurate_nist_calculation: false,
        }
    }
}

/// Overlapping template match test - No. 8
///
/// This test enforces that the input length must be >= 10^6 bits. Smaller values will lead to
/// [Error::InvalidParameter].
///
/// See the [module docs](crate::tests::template_matching::overlapping)
///
/// # About performance
///
/// This test is quite slow in debug mode when using the more precise pi values, taking several
/// seconds - it runs good when using release mode.
/// For better performance, values that are calculated once are cached.
#[use_thread_pool]
pub fn overlapping_template_matching_test(
    data: &BitVec,
    arg: OverlappingTemplateTestArgs,
) -> Result<TestResult, Error> {
    let OverlappingTemplateTestArgs {
        template_length,
        block_length,
        freedom,
        inaccurate_nist_calculation,
    } = arg;

    if block_length < template_length {
        return Err(Error::InvalidParameter(
            format!("the calculated block length {block_length} is smaller than the passed template length {template_length}!")
        ));
    }

    let block_count = data.len_bit() / block_length;

    // calculate the pi values
    let pi_values = if inaccurate_nist_calculation && freedom == 6 {
        calculate_nist_pis(block_length, template_length)
    } else {
        // accurate calculation
        calculate_hamano_kaneko_pis(block_length, template_length, freedom)
    };

    // Step 2: calculate the occurrences of each template in each block. Step only 1 bit on success.
    // sort the number of occurrences in an array with 6 values, 0 stands for no matches,
    // 1 for 1 match, ..., 5 for 5 or more matches
    let occurrences = {
        let mut vec = Vec::with_capacity(freedom);
        vec.resize_with(freedom, || AtomicUsize::new(0));
        vec.into_boxed_slice()
    };
    count_matches_per_chunk(block_count, DEFAULT_BLOCK_LENGTH, data, template_length)
        .try_for_each(|matches_per_chunk| {
            // short circuit; there is only one template
            let matches = matches_per_chunk?;

            // element to increment
            let el_idx = matches.clamp(0, freedom - 1);
            let prev = occurrences[el_idx].fetch_add(1, Ordering::Relaxed);
            if prev == usize::MAX {
                Err(Error::Overflow(format!(" adding 1 to {}", prev)))
            } else {
                Ok(())
            }
        })?;

    // Step 3 makes no sense without the formulae for pi

    // Step 4: compute chi^2 = sum of (v_i - N * pi_i)^2 / (N * pi_i) for each template,
    // with N denoting the block count, v_i denoting each entry in the occurrences array for the template,
    // and pi_i denoting the value of PI_VALUES in the corresponding index.
    let chi = Box::into_iter(occurrences)
        .zip(pi_values)
        .fold(0.0, |sum, (v_i, pi_i)| {
            let numerator = f64::powi((v_i.into_inner() as f64) - (block_count as f64) * pi_i, 2);
            let denominator = (block_count as f64) * pi_i;

            sum + numerator / denominator
        });

    // Step 5: compute p-value = igamc(5/2, chi^2 / 2).
    let p_value = igamc(5.0 / 2.0, chi / 2.0)?;
    Ok(TestResult::new(p_value))
}

/// Calculate the PI values according to the NIST reference implementation.
/// If this function is chosen, the freedom degrees must be 6.
///
/// Returns an array with 6 pi values
fn calculate_nist_pis(block_length: usize, template_length: usize) -> Vec<f64> {
    let lambda =
        ((block_length - template_length + 1) as f64) / f64::powi(2.0, template_length as i32);
    let eta = lambda / 2.0;

    // implementation of the formula described in 3.8
    let pi_0 = f64::exp(-eta);
    let mut pi = vec![
        pi_0,
        eta / 2.0 * pi_0,
        eta / 8.0 * pi_0 * (eta + 2.0),
        eta / 8.0 * pi_0 * (eta * eta / 6.0 + eta + 1.0),
        eta / 16.0 * pi_0 * (eta * eta * eta / 24.0 + eta * eta / 2.0 + 3.0 * eta / 2.0 + 1.0),
    ];
    pi.push(1.0 - pi.iter().sum::<f64>());

    pi
}

/// Type for a pi caching hashmap
type CacheHashMap = HashMap<(usize, usize, usize), Vec<f64>>;

/// Calculate the PI values according to Hamano & Kaneko (as it should be according to the paper).
///
/// Returns an array of count *freedom* with the pi values.
///
/// The code here is an implementation of the formulas described in
/// 'Hamano, Kenji & Kaneko, Toshinobu. (2007). Correction of Overlapping Template Matching Test
/// Included in NIST Randomness Test Suite. IEICE Transactions. 90-A. 1788-1792.
/// 10.1093/ietfec/e90-a.9.1788.'
///
/// # About performance
///
/// This method is quite slow in debug mode, taking several seconds - it runs okay (0.25s) when using
/// release mode. For better performance when running multiple tests, once calculated results are
/// cached.
pub(crate) fn calculate_hamano_kaneko_pis(
    block_length: usize,
    template_length: usize,
    freedom: usize,
) -> Vec<f64> {
    // index transformation helper for the column indexes - rust does not support negative indexes.
    #[inline]
    fn idx(i: isize) -> usize {
        (i + 1) as usize
    }

    // The type to use in the calculations - may be swapped out if e.g. f128 becomes stable in Rust.
    type Decimal = BigDecimal;

    // static cache for already calculated values. Always contains the values for the default
    // argument for better performance.
    static CACHE: LazyLock<Mutex<CacheHashMap>> = LazyLock::new(|| {
        Mutex::new(CacheHashMap::from([(
            (
                DEFAULT_BLOCK_LENGTH,
                DEFAULT_TEMPLATE_LENGTH,
                DEFAULT_FREEDOM,
            ),
            vec![
                0.3640910532167278,
                0.18565890010624034,
                0.13938113045903266,
                0.10057114399877809,
                0.07043232634639843,
                0.13986544587282246,
            ],
        )]))
    });

    // check if already cached & return early if it is
    {
        let cache = CACHE.lock().unwrap();
        if let Some(values) = cache.get(&(block_length, template_length, freedom)) {
            return values.clone();
        }
    }

    // internally, this uses the identifiers used in the paper
    let m = template_length as isize;
    let n = block_length as isize;

    // allocate the necessary tables
    let mut tables: Vec<Vec<Decimal>> = vec![Vec::with_capacity(block_length + 2); freedom - 1];

    // Step 1: compute T_0(n) according to formula (2) - use iterators to allow the Rust compiler
    // to optimize as much as possible.
    (-1..(n + 1)).for_each(|n| {
        if n == -1 || n == 0 {
            tables[0].push(Decimal::from(1u16));
        } else if n < m {
            let value = 2 * &tables[0][idx(n - 1)];
            tables[0].push(value);
        } else {
            let value = 2 * &tables[0][idx(n - 1)] - &tables[0][idx(n - m - 1)];
            tables[0].push(value);
        }
    });

    // Step 2: calculate T_1(n) according to formula (3)
    (-1..(n + 1)).for_each(|n| {
        if n < m {
            tables[1].push(Decimal::from(0u16));
        } else if n == m {
            tables[1].push(Decimal::from(1u16));
        } else if n == m + 1 {
            tables[1].push(Decimal::from(2u16));
        } else {
            let sum = (-1..(n - m))
                .map(|j| &tables[0][idx(j)] * &tables[0][idx(n - m - 2 - j)])
                .sum::<Decimal>();
            tables[1].push(sum);
        }
    });

    // Step 3: for each row with index 'a' left, calculate T_a(n) according to formula (4)
    (2..(freedom - 1)).for_each(|a| {
        // 'a' is the row index.
        // Add a start element with value 0 to each row: this is necessary because else we would
        // try to access the non-existent value at index '-2', we can avoid that by starting
        // with index 0 and setting the first value to 0 (which is the correct one)
        tables[a].push(Decimal::from(0u16));

        (0..(n + 1)).for_each(|n| {
            let part_1 = &tables[a - 1][idx(n - 1)];
            let sum = (-1..(n - 2 * m - (a as isize) + 1))
                .map(|j| &tables[0][idx(j)] * &tables[a - 1][idx(n - m - 2 - j)])
                .sum::<Decimal>();
            let total = part_1 + sum;
            tables[a].push(total);
        })
    });

    // Step 4: calculate each pi value using formula (1) and calculate the last value
    // create the pi vector, the last element is the sum of all pi elements
    let mut pi_sum = Decimal::from(0u8);
    let mut pis = tables
        .iter()
        .map(|row| {
            let divisor = BigInt::from(2u16).pow(block_length as u32).into();
            let pi = &row[block_length + 1] / &divisor;

            pi_sum += &pi;
            pi.to_f64().unwrap()
        })
        .collect::<Vec<_>>();
    pis.push((Decimal::from(1u8) - pi_sum).to_f64().unwrap());

    // insert values into cache
    {
        let mut cache = CACHE.lock().unwrap();
        cache.insert((block_length, template_length, freedom), pis.clone());
    }

    pis
}

/// Count the matches per chunk
fn count_matches_per_chunk(
    block_count: usize,
    block_length_bit: usize,
    data: &BitVec,
    template_len: usize,
) -> impl ParallelIterator<Item = Result<usize, Error>> + '_ {
    // For each block, calculate the times each template matches.
    (0..block_count).into_par_iter().map(move |block_idx| {
        // calculate the start byte and the bit position in the start byte for this block
        let total_start_bit = block_idx
            .checked_mul(block_length_bit)
            .ok_or(Error::Overflow(format!(
                "multiplying {block_idx} by {block_length_bit}"
            )))?;

        // calculate the max shifts
        let max_shifts = block_length_bit - (template_len - 1);

        // create the base template stats
        let base_template = create_mask(template_len);

        // absolute current shift - but still based on word bit count
        let mut absolute_shift = total_start_bit % (usize::BITS as usize);

        // go over the current chunk
        let mut count_matches: usize = 0;

        let mut i = 0;
        while i < max_shifts {
            // the working template
            // This template is bitwise shifted to the right position in the current stream.
            let (template1, template2) =
                overflowing_right_shift(base_template, template_len, absolute_shift);

            // a match is:
            // for every bit, apply bitwise AND with the current mask (which is shifted bitwise
            // for new position) - now only the bits the template tries to match, are there.
            let current_word_idx = (total_start_bit + i) / (usize::BITS as usize);

            let mut matched = data.words[current_word_idx] & template1 == template1;
            // if the first word matched and the data for a second word is there
            if let (true, Some(template2)) = (matched, template2) {
                matched = data.words[current_word_idx + 1] & template2 == template2
            }

            // set the next shift necessary (if the template matched, the shift is for
            // the template length), increment the counter if matched.
            if matched {
                // There are not enough matches possible to warrant checked arithmetic
                count_matches += 1;
            }

            // Calculate the next mask and template.
            absolute_shift = (absolute_shift + 1) % (usize::BITS as usize);

            // increment i - max_shifts cannot be big enough to warrant checked i
            i += 1;
        }

        Ok(count_matches)
    })
}
