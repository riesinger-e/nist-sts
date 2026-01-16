//! Binary Matrix Rank Test
//!
//! This test checks for linear dependence among fixed length substrings of the sequence.
//! These substrings are interpreted as matrices of size 32x32.
//!
//! The sequence must consist of at least 38 912 bits = 4864 bytes.
//!
//! The probability constants were recalculated, using the generic formula for p_r given in 3.5.
//! 
//! Per feedback by Mikołaj Leonarski <m dot leonarski at uw dot edu dot pl>, the 3rd constant was
//! again corrected from p_{m-2} to 1 - p_m - p_{m-1}.

use crate::bitvec::BitVec;
use crate::internals::{check_f64, checked_add, igamc, BitPrimitive};
use crate::{Error, TestResult};
use rayon::prelude::*;
use std::num::NonZero;
use sts_lib_derive::use_thread_pool;

/// The minimum input length, in bits, for this test, as recommended by NIST.
pub const MIN_INPUT_LENGTH: NonZero<usize> = const {
    match NonZero::new(38_912) {
        Some(v) => v,
        None => panic!("Literal should be non-zero!"),
    }
};

/// Rows and columns
const M: usize = u32::BITS as usize;

// Probabilities, calculated with `binary_matrix_probabilities.py`
const PROBABILITIES: [f64; 3] = {
    let p1 = 0.2887880951538411;
    let p2 = 0.5775761901732046;
    [p1, p2, 1.0 - p1 - p2]
};

/// Binary matrix rank test - No. 5.
///
/// See also the [module docs](crate::tests::binary_matrix_rank).
#[use_thread_pool]
pub fn binary_matrix_rank_test(data: &BitVec) -> Result<TestResult, Error> {
    if data.len_bit() < MIN_INPUT_LENGTH.get() {
        return Ok(TestResult::new_with_comment(
            0.0,
            "Data is too short! Minimum is 38 912 Bits.",
        ));
    }

    // Step 1: divide the sequence into blocks with length M * Q = 32 * 32 bits = 32 u32
    let data = data.par_array_chunks_u32::<M>();
    let block_count = data.len();

    let categories = data
        .try_fold(
            || [0_usize; 3],
            |mut categories, chunk| {
                let mut matrix = Matrix(chunk);
                // Step 2: determine the binary rank of each matrix
                let binary_rank = matrix.binary_rank();

                // Step 3: categorise based on the binary rank
                if binary_rank == M {
                    categories[0] = checked_add!(categories[0], 1)?;
                } else if binary_rank == M - 1 {
                    categories[1] = checked_add!(categories[1], 1)?;
                } else {
                    categories[2] = checked_add!(categories[2], 1)?;
                }

                Ok::<_, Error>(categories)
            },
        )
        .try_reduce(
            || [0_usize; 3],
            |mut a, b| {
                for i in 0..3 {
                    a[i] = checked_add!(a[i], b[i])?;
                }

                Ok::<_, Error>(a)
            },
        )?;

    // Step 4: compute chi
    let chi = categories
        .into_iter()
        .zip(PROBABILITIES)
        .map(|(f, p)| {
            let x = p * (block_count as f64);
            f64::powi((f as f64) - x, 2) / x
        })
        .sum::<f64>();

    check_f64(chi)?;

    // Step 5: compute the p_value
    let p_value = igamc(1.0, chi / 2.0)?;
    check_f64(p_value)?;

    Ok(TestResult::new(p_value))
}

/// Matrix: each u32 is 1 row of 32 bits, 32 rows.
#[repr(transparent)]
struct Matrix([u32; M]);

impl Matrix {
    /// Swap 2 rows by their indices
    fn swap_rows(&mut self, i: usize, j: usize) {
        self.0.swap(i, j)
    }

    /// Get the bit in the given row and column
    fn bit(&self, row_idx: usize, col_idx: usize) -> bool {
        self.0[row_idx].get_bit(col_idx as u32)
    }

    /// xor 2 rows, save the result in row target
    fn xor_rows(&mut self, target: usize, i: usize) {
        self.0[target] ^= self.0[i];
    }

    /// Calculate the binary rank of the given matrix according to Appendix F.1.
    fn binary_rank(&mut self) -> usize {
        // Forward row operations
        for i in 0..(M - 1) {
            if !self.bit(i, i) {
                // Step 2
                // Search for a next row with a 1 in this column
                let mut found_row = None;
                for row in (i + 1)..M {
                    if self.bit(row, i) {
                        found_row = Some(row);
                        break;
                    }
                }

                // If found, swap the elements, else: look at next row
                if let Some(row) = found_row {
                    // each value is a row
                    self.swap_rows(i, row);
                } else {
                    continue;
                }
            }

            // Now, el_i,i contains a 1 in every case
            // Step 3
            // For all rows with a 1 in the column i, replace each element e_r,j in the row with e_r,j ^ e_i,j
            for row in (i + 1)..M {
                // test if row is to be changed
                if self.bit(row, i) {
                    self.xor_rows(row, i);
                }
            }

            // Step 4 is integrated in the loop
        }

        // Backward row operation
        for i in (1..M).rev() {
            if !self.bit(i, i) {
                // Step 2
                // Search for a next row with a 1 in this column
                let mut found_row = None;
                for row in (0..i).rev() {
                    if self.bit(row, i) {
                        found_row = Some(row);
                        break;
                    }
                }
                // If found, swap the elements, else: look at next row
                if let Some(row) = found_row {
                    // each value is a row
                    self.swap_rows(i, row);
                } else {
                    continue;
                }
            }

            // Now, el_i,i contains a 1 in every case
            // Step 3
            // For all rows with a 1 in the column i, replace each element e_r,j in the row with e_r,j ^ e_i,j
            for row in (0..i).rev() {
                // test if row is to be changed
                if self.bit(row, i) {
                    self.xor_rows(row, i);
                }
            }

            // Step 4 is integrated in the loop
        }

        // rank of the matrix = the number of non-zero rows
        self.0.iter().filter(|&row| row.count_ones() > 0).count()
    }
}
