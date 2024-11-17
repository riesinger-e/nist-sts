//! The Spectral Discrete Fourier Transform test.
//!
//! This test focuses on the peak heights in the DFT of the input sequence. This is used to detect
//! periodic features that indicate a deviation from a random sequence.
//!
//! It is recommended (but not required) for the input to be of at least 1000 bits.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, erfc, BitPrimitive};
use crate::{Error, TestResult};
use rayon::prelude::*;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::f64::consts::FRAC_1_SQRT_2;
use std::num::NonZero;
use std::ops::Range;
use std::sync::{LazyLock, Mutex};
use sts_lib_derive::use_thread_pool;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(1000) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

// Use a global planner to allow for caching if the test is run multiple times.
static FFT_PLANNER: LazyLock<Mutex<FftPlanner<f32>>> =
    LazyLock::new(|| Mutex::new(FftPlanner::new()));

/// Spectral DFT test - No. 6
///
/// See the [module docs](crate::tests::spectral_dft).
/// If an error happens, it means either arithmetic underflow or overflow.
#[use_thread_pool]
pub fn spectral_dft_test(data: &BitVec) -> Result<TestResult, Error> {
    // Step 1: convert the input bit sequence to a sequence of -1 and +1 (x)
    // This is done in parallel. f32 is used for better performance with such large lists.
    // For use in the fourier transformation, the number is converted to a complex number.
    let (words, last_word) = data.as_full_slice();

    let mut x = words
        .par_iter()
        .flat_map_iter(|&word| {
            // one number per bit
            convert_word(word, 0..usize::BITS)
        })
        .collect::<Vec<_>>();
    // add remaining bits
    if let Some(last_word) = last_word {
        let bits = 0..(data.bit_count_last_word as u32);
        x.extend(convert_word(last_word, bits))
    }

    // the bit length
    let n = data.len_bit();

    debug_assert_eq!(x.len(), n);

    // Step 2: apply a DFT to produce 's'
    // A FFT is a DFT.
    // About the implementation: Panics from another thread should propagate here. The scope is used
    // to keep the Mutex lock as short as possible.
    let fft = {
        let mut fft_planner = FFT_PLANNER.lock().unwrap();
        // The paper is wrong (?), the formula in 3.6 describes the inverse dft
        fft_planner.plan_fft_inverse(x.len())
    };
    // result is stored into the passed buffer
    fft.process(&mut x);

    // Step 4: compute T = sqrt(ln(1/0.05)*n)
    let t = f64::sqrt(f64::ln(1.0 / 0.05) * (n as f64));

    // Step 5: compute n_0 = 0.95 * n / 2
    let n_0 = 0.95 * (n as f64) / 2.0;

    // Step 3: calculate M = |S'|, with S' being the first half of S (=x)
    // Step 6: compute n_1 = count of observed entries in M that are < t
    let n_1 = x[0..(n / 2)]
        .par_iter()
        .try_fold(
            || 0_usize,
            |count, s| {
                let no = Complex::<f64> {
                    re: s.re as f64,
                    im: s.im as f64,
                };
                let norm = no.norm();
                check_f64(norm)?;

                if norm < t {
                    count.checked_add(1).ok_or(Error::Overflow(format!(
                        "adding 1 to count of elements in fft that are > {t}"
                    )))
                } else {
                    Ok(count)
                }
            },
        )
        .try_reduce(
            || 0_usize,
            |a, b| {
                a.checked_add(b)
                    .ok_or(Error::Overflow(format!("fft: adding part-sum {a} to {b}")))
            },
        )? as f64;

    // Step 7: compute d = (n_1 - n_0) / sqrt(data.len_bit() * 0.95 * 0.05 / 4.0)
    let d = (n_1 - n_0) / f64::sqrt((data.len_bit() as f64) * 0.95 * 0.05 / 4.0);
    check_f64(d)?;

    // Step 8: compute p_value = erfc(|d| * 1 / sqrt(2))
    let p_value = erfc(d.abs() * FRAC_1_SQRT_2);
    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}

/// Convert a word into a sequence of bit, with bit 1 -> 1.0 and bit 0 -> -1.0
#[inline]
fn convert_word(word: usize, bits: Range<u32>) -> impl Iterator<Item = Complex<f32>> {
    bits.map(move |bit| {
        let bit = word.get_bit(bit);
        Complex::from(if bit { 1.0 } else { -1.0 })
    })
}
