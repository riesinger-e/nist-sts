//! Frequency (mono bit) test - No. 1
//!
//! This test focuses on the numbers of ones and zeros in the sequence - the proportion should
//! be roughly 50:50.

use crate::{CommonResult, Error};

#[repr(transparent)]
pub struct FrequencyTestArgs<'a> {
    data: &'a [u8],
}

pub struct FrequencyTestArgsBuilder<'a> {
    data: &'a [u8],
    length: Option<usize>,
}

impl<'a> FrequencyTestArgsBuilder<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, length: None }
    }

    pub fn custom_length(mut self, length: usize) -> Self {
        self.length = Some(length);
        self
    }

    pub fn build(self) -> FrequencyTestArgs<'a> {
        let data = if let Some(length) = self.length {
            &self.data[0..length]
        } else {
            self.data
        };

        FrequencyTestArgs { data }
    }
}

pub fn frequency_test(args: FrequencyTestArgs) -> Result<CommonResult, Error> {
    let data = args.data;

    // Step 1: convert 0 values to -1 and calculate the sum of all bits.
    let sum = data.iter().try_fold(0isize, |mut sum, value| {
        // the count of bits with value '1' in the byte
        let count_ones = value.count_ones();
        // the count of zeros is built from the count of ones (1 byte = 8 bits)
        let count_zeros = 8 - count_ones;

        // Adding and subtracting the count from the sum ist the same as conversion to -1 and +1.
        // Conversion to usize is definitely safe - count_ones and count_zeros range `0..=8`
        sum = sum
            .checked_add_unsigned(count_ones as usize)
            .ok_or(Error::Overflow(format!(
                "adding Ones to the sum: {sum} + {count_ones}"
            )))?;
        sum = sum
            .checked_sub_unsigned(count_zeros as usize)
            .ok_or(Error::Overflow(format!(
                "removing Zeroes from the sum: {sum} + {count_zeros}"
            )))?;
        sum -= count_zeros as isize;
        Ok(sum)
    })?;

    // Step 2: compute s_obs = abs(sum) / sqrt(n)
    let s_obs = (sum
        .checked_abs()
        .ok_or(Error::Overflow(format!("abs({sum}) - type isize")))? as f64)
        / f64::sqrt(data.len() as f64);

    // Step 3: compute P-value = erfc(s_obs / sqrt(2))
    todo!()
}
