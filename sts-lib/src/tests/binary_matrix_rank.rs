//! Binary Matrix Rank Test
//!
//! This test checks for linear dependence among fixed length substrings of the sequence.
//! These substrings are interpreted as matrices of size 32x32.
//!
//! The sequence must consist of at least 38 912 bits = 4864 bytes.

use crate::bitvec::BitVec;
use crate::{Error, TestResult, BYTE_SIZE};
use rayon::prelude::*;
use crate::internals::{check_f64, igamc};

/// Rows and columns
const M: usize = 32;
const MATRIX_BYTE_SIZE: usize = M * M / BYTE_SIZE;

// Probabilities, calculated with `binary_matrix_probabilities.py`
const PROBABILITIES: [f64; 3] = [0.1283502644231667, 0.5775761901732046, 0.2887880951538411];

/// Binary matrix rank test.
///
/// See also the [module docs](crate::tests::binary_matrix_rank).

pub fn binary_matrix_rank_test(data: &BitVec) -> Result<TestResult, Error> {
    if data.len_bit() < 38_912 {
        return Ok(TestResult::new_with_comment(
            0.0,
            "Data is too short! Minimum is 38 912 Bits.",
        ));
    }

    // Step 1: divide the sequence into blocks with length M * Q. Since M and Q are both
    // whole bytes, we don't have to think about the remainder.
    let block_count = data.data.len() / M * M / BYTE_SIZE;

    let categories = data
        .data
        .par_chunks_exact(M * M / BYTE_SIZE)
        .try_fold(
            || [0_usize; 3],
            |mut categories, chunk| {
                // this cannot fail, the chunks are this exact size
                let matrix: [u8; M * M / BYTE_SIZE] = chunk.try_into().unwrap();
                // Step 2: determine the binary rank of each matrix
                let binary_rank = binary_rank(matrix)?;

                // Step 3: categorise based on the binary rank
                if binary_rank == M {
                    categories[0] = categories[0].checked_add(1).ok_or(Error::Overflow(
                        format!("adding 1 to the rank category F_M, value {}", categories[0]),
                    ))?;
                } else if binary_rank == M - 1 {
                    categories[1] =
                        categories[1].checked_add(1).ok_or(Error::Overflow(format!(
                            "adding 1 to the rank category F_M-1, value {}",
                            categories[1]
                        )))?;
                } else {
                    categories[2] =
                        categories[2].checked_add(1).ok_or(Error::Overflow(format!(
                            "adding 1 to the rank category F_rem, value {}",
                            categories[2]
                        )))?;
                }

                Ok::<_, Error>(categories)
            },
        )
        .try_reduce(
            || [0_usize; 3],
            |mut a, b| {
                for i in 0..3 {
                    a[i] = a[i].checked_add(b[i]).ok_or(Error::Overflow(format!(
                        "adding part category sums {} and {}",
                        a[i], b[i]
                    )))?;
                }

                Ok::<_, Error>(a)
            },
        )?;

    // Step 4: compute chi
    let chi = categories.into_iter().zip(PROBABILITIES)
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

/// Calculate the binary rank of the given matrix according to Appendix F.1.
fn binary_rank(mut matrix: [u8; M * M / BYTE_SIZE]) -> Result<usize, Error> {
    // Forward row operations
    for i in 0..(M - 1) {
        if !get_matrix_el_at(&matrix, i, i)? {
            // Step 2
            // Search for a next row with a 1 in this column
            let mut found_row = None;
            for row in (i + 1)..M {
                if get_matrix_el_at(&matrix, row, i)? {
                    found_row = Some(row);
                    break;
                }
            }
            // If found, swap the elements, else: look at next row
            if let Some(row) = found_row {
                swap_matrix_rows(&mut matrix, i, row)?;
            } else {
                continue;
            }
        }

        // Now, el_i,i contains a 1 in every case
        // Step 3
        // For all rows with a 1 in the column i, replace each element e_r,j in the row with e_r,j ^ e_i,j
        for row in (i + 1)..M {
            // test if row is to be changed
            if !get_matrix_el_at(&matrix, row, i)? {
                continue;
            }

            for col in i..M {
                let v1 = get_matrix_el_at(&matrix, i, col)?;
                let v2 = get_matrix_el_at(&matrix, row, col)?;
                set_matrix_el_at(&mut matrix, row, col, v1 ^ v2)?;
            }
        }

        // Step 4 is integrated in the loop
    }

    // Backward row operation
    for i in (1..M).rev() {
        if !get_matrix_el_at(&matrix, i, i)? {
            // Step 2
            // Search for a next row with a 1 in this column
            let mut found_row = None;
            for row in (0..i).rev() {
                if get_matrix_el_at(&matrix, row, i)? {
                    found_row = Some(row);
                    break;
                }
            }
            // If found, swap the elements, else: look at next row
            if let Some(row) = found_row {
                swap_matrix_rows(&mut matrix, i, row)?;
            } else {
                continue;
            }
        }

        // Now, el_i,i contains a 1 in every case
        // Step 3
        // For all rows with a 1 in the column i, replace each element e_r,j in the row with e_r,j ^ e_i,j
        for row in (0..i).rev() {
            // test if row is to be changed
            if !get_matrix_el_at(&matrix, row, i)? {
                continue;
            }

            for col in (0..i).rev() {
                let v1 = get_matrix_el_at(&matrix, i, col)?;
                let v2 = get_matrix_el_at(&matrix, row, col)?;
                set_matrix_el_at(&mut matrix, row, col, v1 ^ v2)?;
            }
        }

        // Step 4 is integrated in the loop
    }

    // rank of the matrix = the number of non-zero rows
    let mut rank = 0;
    for row in 0..M {
        for col in 0..M {
            if get_matrix_el_at(&matrix, row, col)? {
                // row has at least one non-zero element
                rank += 1;
                break;
            }
        }
    }

    Ok(rank)
}

/// Get the matrix element at the specified, zero-based indices.
/// Returns [Error::Overflow] if the given indices are invalid.
fn get_matrix_el_at(
    matrix: &[u8; M * M / BYTE_SIZE],
    row: usize,
    col: usize,
) -> Result<bool, Error> {
    let (byte_idx, bit_idx) = calculate_matrix_indices(row, col)?;

    let byte = matrix[byte_idx];
    let bit = (byte >> (BYTE_SIZE - bit_idx - 1)) & 0x01;
    Ok(bit == 1)
}

/// Set the matrix element at the specified, zero-based indices.
/// Returns [Error::Overflow] if the given indices are invalid.
fn set_matrix_el_at(
    matrix: &mut [u8; M * M / BYTE_SIZE],
    row: usize,
    col: usize,
    value: bool,
) -> Result<(), Error> {
    let (byte_idx, bit_idx) = calculate_matrix_indices(row, col)?;

    let byte = &mut matrix[byte_idx];

    let new_bit = 1 << (BYTE_SIZE - bit_idx - 1);

    if value {
        // use binary OR to ensure the 1
        *byte |= new_bit;
    } else {
        // use binary AND to ensure the 0 - binary negation to ensure the right value
        *byte &= !new_bit;
    }

    Ok(())
}

/// Swap the rows at specified, zero-based column indices.
/// Returns [Error::Overflow] if the given indices are invalid.
fn swap_matrix_rows(matrix: &mut [u8; M * M / BYTE_SIZE], row1: usize, row2: usize) -> Result<(), Error> {
    for j in 0..M {
        let el1 = get_matrix_el_at(matrix, row1, j)?;
        let el2 = get_matrix_el_at(matrix, row2, j)?;

        set_matrix_el_at(matrix, row1, j, el2)?;
        set_matrix_el_at(matrix, row2, j, el1)?;
    }
    Ok(())
}

/// Validate the matrix indices and return the tuple (byte_idx, bit_idx), byte_idx meaning which byte
/// and bit_idx meaning which bit in the byte.
///
/// Returns [Error::Overflow] if the given indices are invalid.
fn calculate_matrix_indices(row: usize, col: usize) -> Result<(usize, usize), Error> {
    if row >= M {
        return Err(Error::Overflow(format!("Row {row} is out of bounds: {M}")));
    }
    if col >= M {
        return Err(Error::Overflow(format!(
            "Column {col} is out of bounds: {M}"
        )));
    }
    let total_bit_idx = row * M + col;
    // Which byte
    let byte_idx = total_bit_idx / BYTE_SIZE;
    // Which bit in the byte
    let bit_idx = total_bit_idx % BYTE_SIZE;
    Ok((byte_idx, bit_idx))
}
