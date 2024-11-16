//! The random excursions variant test.
//!
//! This test is quite similar to the [random excursions test](crate::tests::random_excursions),
//! with the key difference being that the frequencies are calculated over all cycles, instead of per
//! cycle.
//!
//! This test does not require a minimum number of cycles.
//!
//! If the computation finishes successfully, 18 [TestResult] are returned: one for each tested state,
//! `x`. The results will contain a comment about the state they are calculated from (e.g. "x = 3"),
//! the order is: `[-9, -8, -7, -6, -5, -4, -3, -2, -1, +1, +2, +3, +4, +5, +6, +7, +8, +9]`.
//!
//! The input length must be at least 10^6 bits, otherwise, an error is returned.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, erfc, get_bit_from_value};
use crate::{Error, TestResult};
use std::num::NonZero;
use std::ops::Range;
use sts_lib_derive::use_thread_pool;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(1_000_000) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

/// Random excursions variant test - No. 15.
///
/// See the [module docs](crate::tests::random_excursions_variant).
/// If the given [BitVec] contains fewer than 10^6 bits, [Error::InvalidParameter] is returned.
#[use_thread_pool]
pub fn random_excursions_variant_test(data: &BitVec) -> Result<[TestResult; 18], Error> {
    #[cfg(not(test))]
    {
        if data.len_bit() < 1_000_000 {
            return Err(Error::InvalidParameter(format!(
                "The input bit length must be at >= 10^6. Is: {}",
                data.len_bit()
            )));
        }
    }

    // Step 1 to 4 - see also the random excursions test.
    let mut frequencies = [0_usize; 18];
    let mut prev: i64 = 0;
    let mut num_cycles = 1;

    let (words, last_word) = data.as_full_slice();

    for &word in words {
        handle_word(
            word,
            0..(usize::BITS as usize),
            &mut prev,
            &mut num_cycles,
            &mut frequencies,
        )?;
    }

    if let Some(word) = last_word {
        let bits = 0..(data.bit_count_last_word as usize);
        handle_word(word, bits, &mut prev, &mut num_cycles, &mut frequencies)?;
    }

    #[cfg(not(test))]
    {
        // check the number of cycles based on the last paragraph of 3-22. Although the need for this
        // check is not mentioned in 2.15, it is mentioned in 3.15.
        let min_cycles = f64::max(0.005 * f64::sqrt(data.len_bit() as f64), 500.0);
        if (num_cycles as f64) < min_cycles {
            return Ok([TestResult::new_with_comment(0.0, "Too few cycles"); 18]);
        }
    }

    // Step 5: calculate p_values
    let mut p_values = [
        TestResult::new_with_comment(0.0, "x = -9"),
        TestResult::new_with_comment(0.0, "x = -8"),
        TestResult::new_with_comment(0.0, "x = -7"),
        TestResult::new_with_comment(0.0, "x = -6"),
        TestResult::new_with_comment(0.0, "x = -5"),
        TestResult::new_with_comment(0.0, "x = -4"),
        TestResult::new_with_comment(0.0, "x = -3"),
        TestResult::new_with_comment(0.0, "x = -2"),
        TestResult::new_with_comment(0.0, "x = -1"),
        TestResult::new_with_comment(0.0, "x = +1"),
        TestResult::new_with_comment(0.0, "x = +2"),
        TestResult::new_with_comment(0.0, "x = +3"),
        TestResult::new_with_comment(0.0, "x = +4"),
        TestResult::new_with_comment(0.0, "x = +5"),
        TestResult::new_with_comment(0.0, "x = +6"),
        TestResult::new_with_comment(0.0, "x = +7"),
        TestResult::new_with_comment(0.0, "x = +8"),
        TestResult::new_with_comment(0.0, "x = +9"),
    ];

    let num_cycles = num_cycles as f64;

    for (i, frequency) in frequencies.into_iter().enumerate() {
        let x = if i < 9 {
            // 0 -> -9
            // 8 -> -1
            (i as f64) - 9.0
        } else {
            // 9 -> 1
            // 17 -> 9
            (i as f64) - 8.0
        };

        let p_value = erfc(
            f64::abs(frequency as f64 - num_cycles)
                / f64::sqrt(2.0 * num_cycles * (4.0 * f64::abs(x) - 2.0)),
        );

        check_f64(p_value)?;

        p_values[i].p_value = p_value;
    }

    Ok(p_values)
}

/// Handle step 1 to 4 for one word, with a specified bit range
fn handle_word(
    word: usize,
    mut bits: Range<usize>,
    prev: &mut i64,
    num_cycles: &mut usize,
    frequencies: &mut [usize; 18],
) -> Result<(), Error> {
    bits.try_for_each(|bit| -> Result<(), Error> {
        if get_bit_from_value(word, bit) {
            *prev += 1
        } else {
            *prev -= 1
        }

        // increment counter for state occurrences per cycle
        if inc_frequency(frequencies, *prev)? {
            *num_cycles += 1;
        }

        Ok(())
    })
}

/// Increments the right frequency counter based on the current value, returns true if a new
/// cycle started.
fn inc_frequency(frequencies: &mut [usize; 18], value: i64) -> Result<bool, Error> {
    let idx = match value {
        ..-9 => return Ok(false),
        // -9 -> 0
        // -1 -> 8
        -9..0 => (value + 9) as usize,
        0 => return Ok(true),
        // 1 -> 9
        1..=9 => (value + 8) as usize,
        10.. => return Ok(false),
    };

    frequencies[idx] = frequencies[idx]
        .checked_add(1)
        .ok_or(Error::Overflow(format!(
            "Frequency {} overflowed when adding 1.",
            frequencies[idx]
        )))?;

    Ok(false)
}
