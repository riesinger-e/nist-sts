//! The random excursions test.
//!
//! This test, similarly to the [cumulative sums test](crate::tests::cumulative_sums), calculates
//! cumulative sums of a digit-adjusted (-1, +1) bit sequence, but only from the beginning to the end.
//! This test checks if the frequency of cumulative sums values per cycle is as expected for
//! a random sequence. A cycle consists of all cumulative sums between 2 "0"-values.
//!
//! Since the test needs at least 500 cycles to occur, bit sequences with fewer cycles will lead to an
//! `Ok()` result, but with the values filled with "0.0".
//!
//! If the computation finishes successfully, 8 [TestResult] are returned: one for each tested state,
//! `x`. The results will contain a comment about the state they are calculated from (e.g. "x = 3"),
//! the order is: `[-4, -3, -2, -1, +1, +2, +3, +4]`.
//!
//! The input length must be at least 10^6 bits, otherwise, an error is returned.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, igamc};
use crate::{Error, TestResult, BYTE_SIZE};

/// Constant probabilities, calculated with python script `random_excursions.py`, with the values
/// reinterpreted as fractions.
#[rustfmt::skip]
const PROBABILITIES: [[f64; 8]; 6] = [
    [    7.0 /      8.0,   5.0 /    6.0,  3.0 /    4.0, 1.0 /  2.0, 1.0 /  2.0,  3.0 /    4.0,   5.0 /    6.0,     7.0 /      8.0 ],
    [    1.0 /     64.0,   1.0 /   36.0,  1.0 /   16.0, 1.0 /  4.0, 1.0 /  4.0,  1.0 /   16.0,   1.0 /   36.0,     1.0 /     64.0 ],
    [    7.0 /    512.0,   5.0 /  216.0,  3.0 /   64.0, 1.0 /  8.0, 1.0 /  8.0,  3.0 /   64.0,   5.0 /  216.0,     7.0 /    512.0 ],
    [   49.0 /   4096.0,  25.0 / 1296.0,  9.0 /  256.0, 1.0 / 16.0, 1.0 / 16.0,  9.0 /  256.0,  25.0 / 1296.0,    49.0 /   4096.0 ],
    [  343.0 / 32_768.0, 125.0 / 7776.0, 27.0 / 1024.0, 1.0 / 32.0, 1.0 / 32.0, 27.0 / 1024.0, 125.0 / 7776.0,   343.0 / 32_768.0 ],
    [ 2401.0 / 32_768.0, 625.0 / 7776.0, 81.0 / 1024.0, 1.0 / 32.0, 1.0 / 32.0, 81.0 / 1024.0, 625.0 / 7776.0,  2401.0 / 32_768.0 ],
];

/// Random excursions test - No. 14
///
/// See the [module docs](crate::tests::random_excursions).
/// If the given [BitVec] contains fewer than 10^6 bits, [Error::InvalidParameter] is returned.
pub fn random_excursions_test(data: &BitVec) -> Result<[TestResult; 8], Error> {
    #[cfg(not(test))]
    {
        if data.len_bit() < 1_000_000 {
            return Err(Error::InvalidParameter(format!(
                "The input bit length must be at >= 10^6. Is: {}",
                data.len_bit()
            )));
        }
    }

    // Steps 1 to 5: calculate the cum sums (stored in prev), increment a counter per state
    // per cycle, dynamically create a new entry per cycle. The count of cycles can be determined
    // afterwards from the length of states_per_cycle.
    let mut states_per_cycle = vec![[0_u8; 8]];
    let mut last_index = 0;
    let mut prev: i64 = 0;

    for &byte in &data.data {
        (0..BYTE_SIZE)
            .rev()
            .map(|shift| 1 << shift)
            .for_each(|mask| {
                if byte & mask != 0 {
                    prev += 1
                } else {
                    prev -= 1
                }

                // increment counter for state occurrences per cycle
                if set_state(&mut states_per_cycle[last_index], prev) {
                    states_per_cycle.push(Default::default());
                    last_index += 1;
                }
            });
    }

    for &bit in &data.remainder {
        // set the previous value to the current value.
        if bit {
            prev += 1;
        } else {
            prev -= 1;
        }

        // increment counter for state occurrences per cycle
        if set_state(&mut states_per_cycle[last_index], prev) {
            states_per_cycle.push(Default::default());
            last_index += 1;
        }
    }

    let num_cycles = states_per_cycle.len();

    // only check this property when not running unit tests.
    #[cfg(not(test))]
    {
        if num_cycles < 500 {
            return Ok([TestResult::new_with_comment(0.0, "Too few cycles"); 8]);
        }
    }

    let num_cycles = num_cycles as f64;

    // Step 6: based on states_per_cycle, compute v_k(x) = the total number of cycles in which state
    // x occurred exactly k times, for k = 0, 1, 2, 3, 4, >= 5
    let mut v = [[0_usize; 8]; 6];
    states_per_cycle
        .into_iter()
        .flat_map(|cycle| cycle.into_iter().enumerate())
        .for_each(|(state, occurrences)| {
            let idx = occurrences.clamp(0, 5) as usize;
            v[idx][state] += 1;
        });

    // Step 7: for each state, compute chi = sum_{k} ( v_k(x) - J * pi_k(x) )^2 / ( J * pi_k(x) ).
    // pi_k(x) is the precalculated probability.
    let mut chis = [0.0; 8];
    v.into_iter()
        .zip(PROBABILITIES)
        .flat_map(|(v_k, pi_k)| v_k.into_iter().zip(pi_k).enumerate())
        .for_each(|(i, (v_k_x, pi_k_x))| {
            chis[i] += f64::powi(v_k_x as f64 - num_cycles * pi_k_x, 2) / (num_cycles * pi_k_x);
        });

    let mut p_values = [
        TestResult::new_with_comment(0.0, "x = -4"),
        TestResult::new_with_comment(0.0, "x = -3"),
        TestResult::new_with_comment(0.0, "x = -2"),
        TestResult::new_with_comment(0.0, "x = -1"),
        TestResult::new_with_comment(0.0, "x = +1"),
        TestResult::new_with_comment(0.0, "x = +2"),
        TestResult::new_with_comment(0.0, "x = +3"),
        TestResult::new_with_comment(0.0, "x = +4"),

    ];
    chis.into_iter()
        .enumerate()
        .try_for_each(|(i, chi)| -> Result<(), Error> {
            check_f64(chi)?;
            let p_value = igamc(5.0 / 2.0, chi / 2.0)?;
            check_f64(p_value)?;
            p_values[i].p_value = p_value;
            Ok(())
        })?;

    Ok(p_values)
}

/// Sets the state of the current cycle based on the current cumulative sum.
/// If `true` is returned, a new cycle has begun.
fn set_state(states: &mut [u8; 8], value: i64) -> bool {
    // since we're only interested in occurrences of 0, 1, 2, 3, 4, and >=5, saturating add is
    // completely fine.
    match value {
        -4 => states[0] = states[0].saturating_add(1),
        -3 => states[1] = states[1].saturating_add(1),
        -2 => states[2] = states[2].saturating_add(1),
        -1 => states[3] = states[3].saturating_add(1),
        1 => states[4] = states[4].saturating_add(1),
        2 => states[5] = states[5].saturating_add(1),
        3 => states[6] = states[6].saturating_add(1),
        4 => states[7] = states[7].saturating_add(1),
        0 => {
            return true;
        }
        _ => (),
    }

    false
}
