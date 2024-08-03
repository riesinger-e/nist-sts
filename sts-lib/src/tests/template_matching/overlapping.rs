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
//! * The value *K*, as given in the paper, ist just wrong. You don't need a statistics degree to see
//!   that it is 6 and not 5.
//!
//! This test needs arguments, see [OverlappingTemplateTestArgs].

use crate::bitvec::BitVec;
use crate::internals::igamc;
use crate::tests::template_matching::{count_matches_per_chunk_per_template, TemplateArg};
use crate::{Error, TestResult, BYTE_SIZE};
use bigdecimal::num_bigint::BigInt;
use bigdecimal::num_traits::ToPrimitive;
use bigdecimal::BigDecimal;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

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

    // input check
    #[cfg(not(test))]
    {
        if data.len_bit() < 1_000_000 {
            return Err(Error::InvalidParameter(format!(
                "The passed length of the input sequence is smaller than 10^6. Is: {}",
                data.len_bit()
            )));
        }
    }

    if block_length < template_length {
        return Err(Error::InvalidParameter(
            format!("the calculated block length {block_length} is smaller than the passed template length {template_length}!")
        ));
    }

    let block_count = data.len_bit() / block_length;

    // dynamically create the template
    let template_bits = {
        let whole_bytes = template_length / BYTE_SIZE;
        let mut template_bits = vec![0xFF; whole_bytes];

        let extra_bits = template_length % BYTE_SIZE;
        if extra_bits != 0 {
            let mut byte: u8 = 0;
            for i in 0..extra_bits {
                byte |= 1 << (BYTE_SIZE - i - 1);
            }

            template_bits.push(byte);
        }

        template_bits
    };
    let templates = [template_bits.as_slice()];
    let templates =
        TemplateArg::new_with_custom_templates(templates.as_slice(), template_length).unwrap();

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
    let occurrences =
        count_matches_per_chunk_per_template(block_count, DEFAULT_BLOCK_LENGTH, data, templates, 1)
            .try_fold(
                vec![0_usize; freedom].into_boxed_slice(),
                |mut sum, matches_per_template| {
                    // short circuit; there is only on template
                    let matches = matches_per_template?[0];

                    // element to increment
                    let el_idx = matches.clamp(0, 5);
                    let element = &mut sum[el_idx];
                    *element = element
                        .checked_add(1)
                        .ok_or(Error::Overflow(format!(" adding 1 to {}", *element)))?;

                    Ok::<_, Error>(sum)
                },
            )?;

    // Step 3 makes no sense without the formulae for pi

    // Step 4: compute chi^2 = sum of (v_i - N * pi_i)^2 / (N * pi_i) for each template,
    // with N denoting the block count, v_i denoting each entry in the occurrences array for the template,
    // and pi_i denoting the value of PI_VALUES in the corresponding index.
    let chi = Box::into_iter(occurrences)
        .zip(pi_values)
        .fold(0.0, |sum, (v_i, pi_i)| {
            let numerator = f64::powi((v_i as f64) - (block_count as f64) * pi_i, 2);
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
#[allow(clippy::vec_init_then_push)]
fn calculate_nist_pis(block_length: usize, template_length: usize) -> Vec<f64> {
    let lambda =
        ((block_length - template_length + 1) as f64) / f64::powi(2.0, template_length as i32);
    let eta = lambda / 2.0;

    let mut pi = Vec::with_capacity(6);

    // implementation of the formula described in 3.8
    pi.push(f64::exp(-eta));
    pi.push(eta / 2.0 * pi[0]);
    pi.push(eta / 8.0 * pi[0] * (eta + 2.0));
    pi.push(eta / 8.0 * pi[0] * (eta * eta / 6.0 + eta + 1.0));
    pi.push(
        eta / 16.0 * pi[0] * (eta * eta * eta / 24.0 + eta * eta / 2.0 + 3.0 * eta / 2.0 + 1.0),
    );
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

    // static cache for already calculated values.
    static CACHE: LazyLock<Mutex<CacheHashMap>> = LazyLock::new(|| Mutex::new(CacheHashMap::new()));

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
