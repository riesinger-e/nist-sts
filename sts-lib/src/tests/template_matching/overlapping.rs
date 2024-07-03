//! Overlapping Template Matching test.
//!
//! This test tries to detect RNGs that produce too many occurrences of a given aperiodic pattern.
//! This test uses an m-bit window to search for an m-bit pattern.
//! The big difference to the [non-overlapping](super::non_overlapping) test is that template matches
//! may overlap.
//!
//! The default arguments for this test derivate significantly from the NIST reference implementation,
//! since the NIST reference implementation for this test is known bad.
//! The corrections are taken from https://eprint.iacr.org/2022/540 - they are the only freely available
//! source on how to calculate the precise PI values according to Hamano and Kaneko.
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
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// The default length of each block M, in bits.
pub const DEFAULT_BLOCK_LENGTH: usize = 1032;

/// The default degree of freedom K.
pub const DEFAULT_FREEDOM: usize = 6;

/// The default template length.
pub const DEFAULT_TEMPLATE_LENGTH: usize = 9;

/// The arguments for the Non-overlapping Template Matching Test.
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
#[repr(C)]
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
/// This test enforces that the input length must be >= 10^6 bits. Smaller values will lead to 1 result
/// with a p-value of 0.0.
///
/// See the [module docs](crate::tests::template_matching::overlapping)
///
/// # About performance
///
/// This test is quite slow in debug mode when using the more precise pi values, taking several
/// seconds - it runs okay (0.25s) when using release mode.
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
            .try_fold(vec![0_usize; freedom], |mut sum, matches_per_template| {
                // short circuit; there is only on template
                let matches = matches_per_template?[0];

                // element to increment
                let el_idx = matches.clamp(0, 5);
                let element = &mut sum[el_idx];
                *element = element
                    .checked_add(1)
                    .ok_or(Error::Overflow(format!(" adding 1 to {}", *element)))?;

                Ok::<_, Error>(sum)
            })?;

    // Step 3 makes no sense without the formulae for pi

    // Step 4: compute chi^2 = sum of (v_i - N * pi_i)^2 / (N * pi_i) for each template,
    // with N denoting the block count, v_i denoting each entry in the occurrences array for the template,
    // and pi_i denoting the value of PI_VALUES in the corresponding index.
    let chi = occurrences
        .into_iter()
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
/// The code here is an adapted form of the code described in https://eprint.iacr.org/2022/540 , but
/// with arbitrary decimal precision.
///
/// # About performance
///
/// This method is quite slow in debug mode, taking several seconds - it runs okay (0.25s) when using
/// release mode.
pub(crate) fn calculate_hamano_kaneko_pis(
    block_length: usize,
    template_length: usize,
    freedom: usize,
) -> Vec<f64> {
    static CACHE: Mutex<Lazy<CacheHashMap>> = Mutex::new(Lazy::new(HashMap::new));

    // check if already cached & return early if it is
    {
        let cache = CACHE.lock().unwrap();
        if let Some(values) = cache.get(&(block_length, template_length, freedom)) {
            return values.clone();
        }
    }

    let signed_template_length = template_length as isize;
    // create tables
    let mut tables: Vec<Vec<BigDecimal>> = vec![Vec::with_capacity(block_length + 2); freedom - 1];

    // compute first table row tables[0]
    (0..(block_length + 2)).for_each(|n| {
        if n == 0 || n == 1 {
            tables[0].push(1u32.into())
        } else if n <= template_length {
            let prev_value = &tables[0][n - 1];
            let new_value = BigDecimal::from(2u32) * prev_value;
            tables[0].push(new_value);
        } else {
            let prev_value_1 = &tables[0][n - 1];
            let prev_value_2 = &tables[0][n - template_length - 1];
            let new_value = BigDecimal::from(2u32) * prev_value_1 - prev_value_2;
            tables[0].push(new_value);
        }
    });

    // compute second table row tables[1]
    (0..(block_length + 2)).for_each(|n| {
        if n <= template_length {
            tables[1].push(0u32.into());
        } else if n == template_length + 1 {
            tables[1].push(1u32.into());
        } else if n == template_length + 2 {
            tables[1].push(2u32.into());
        } else {
            // signed math is needed here
            let n = n as isize - 1;

            let sum = (-1..(n - signed_template_length))
                .map(|j| {
                    &tables[0][(j + 1) as usize]
                        * &tables[0][(n - signed_template_length - 2 - j + 1) as usize]
                })
                .sum::<BigDecimal>();
            tables[1].push(sum)
        }
    });

    // compute the remaining rows - signed math is needed here
    (2..(freedom - 1)).for_each(|a| {
        (0..(block_length + 2)).for_each(|n| {
            let a = a as isize;
            // signed maths is needed here
            let n = n as isize - 1;

            let sum = (-1..(n - (2 * signed_template_length) - a))
                .map(|j| {
                    &tables[0][(j + 1) as usize]
                        * &tables[(a - 1) as usize]
                            [(n - signed_template_length - 2 - j + 1) as usize]
                })
                .sum::<BigDecimal>();

            let sum = if n >= 0 {
                sum + &tables[(a - 1) as usize][n as usize]
            } else {
                sum
            };

            tables[a as usize].push(sum);
        })
    });

    // create the pi vector, the last element is the sum of all pi elements
    let mut pi_sum = 0.0;
    let mut pis = tables
        .iter()
        .map(|row| {
            let pi = {
                let divisor = BigInt::from(2u32).pow(block_length as u32).into();
                let pi = &row[block_length + 1] / &divisor;

                pi.to_f64().unwrap()
            };

            pi_sum += pi;
            pi
        })
        .collect::<Vec<_>>();
    pis.push(1.0 - pi_sum);

    // insert values into cache
    {
        let mut cache = CACHE.lock().unwrap();
        cache.insert((block_length, template_length, freedom), pis.clone());
    }

    pis
}
