//! Frequency Test within a block.
//!
//! This tests for the same property as [crate::frequency_test], but within M-bit blocks.

use crate::bitvec::BitVec;
use crate::{BYTE_SIZE, CommonResult, Error};
use rayon::prelude::*;
use crate::internals::{check_f64, igamc};

/// The arguments for the Frequency test within a block.
///
/// This is used to pass the block length.
#[repr(transparent)]
pub struct FrequencyBlockTestArgs {
    // the block length, in bytes
    block_length: usize
}

impl FrequencyBlockTestArgs {
    /// Creates a new argument for the test. Note that, different to the NIST
    /// reference implementation, for performance reasons, only multiples of 8 are
    /// allowed. If a block length is specified that is not a multiple of 8, this
    /// returns `None`.
    pub fn new(block_length: usize) -> Option<Self> {
        if block_length % BYTE_SIZE == 0 {
            Some(Self {
                block_length: block_length / BYTE_SIZE
            })
        } else {
            None
        }
    }
}

/// Frequency test within a block - No. 2
///
/// See the [module docs](crate::frequency_block_test).
/// If no [FrequencyBlockTestArgs] are passed, a reasonable default, based on 2.2.7, is chosen.
/// If an error happens, it means either arithmetic underflow or overflow - beware.
pub fn frequency_block_test(data: BitVec, test_args: Option<FrequencyBlockTestArgs>) -> Result<CommonResult, Error> {
    // Step 0 - get the block length or calculate one
    let block_length = if let Some(test_args) = test_args {
        test_args.block_length
    } else {
        choose_block_length(data.data.len())
    };

    // Step 1 - calculate the amount of blocks
    let block_count = data.data.len() / block_length;

    let half_chi: f64 = data.data.chunks_exact(block_length)
        .map(|chunk| {
            // Step 2 - calculate pi_i = (ones in the block) / block_length for each block

            // count of ones in the block
            let count_ones = chunk.par_iter()
                .try_fold(|| 0_usize, |sum, value| {
                    sum.checked_add(value.count_ones() as usize)
                        .ok_or(Error::Overflow(format!("adding ones to the sum: {sum}")))
                })
                .try_reduce(|| 0_usize, |a, b| {
                    a.checked_add(b).ok_or(Error::Overflow(format!("Adding two parts of the sum: {a} + {b}")))
                })? as f64;

            let pi = count_ones / (block_length as f64);

            // Step 3 - compute the chi^2 statistics - calculate the current part
            let chi_part = (pi - 0.5).powi(2);
            Result::<_, Error>::Ok(chi_part)
        })
        // Step 3 - build sum and multiply with 4 * block_length
        // In Step 4, chi is again halved - do this now (replace 4 with 2)
        .sum::<Result<f64, _>>()? * 2.0 * (block_length as f64);

    check_f64(half_chi)?;
    
    // Step 4: compute p-value = igamc(block_count / 2, chi / 2)
    let p_value = igamc(block_count as f64 / 2.0, half_chi)?;

    check_f64(p_value)?;
    
    Ok(CommonResult { p_value })
}

/// Choose a block length based on 2.2.7. Needs the amount of bytes (each byte contains 8 values)
/// as the parameter.
fn choose_block_length(byte_vec_length: usize) -> usize {
    const MIN_BLOCK_LENGTH: usize = 20 / BYTE_SIZE + 1;

    // Start with the recommended minimum block length based on the length of the data.
    // This also satisfies that less than 100 block should exist.
    let block_length = byte_vec_length / 100 + 1;

    if block_length < MIN_BLOCK_LENGTH {
        // has to be at least the min block length
        MIN_BLOCK_LENGTH
    } else {
        block_length
    }
}