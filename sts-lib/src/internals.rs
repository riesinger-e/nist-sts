//! Internal functions that are used by tests - can be changed anytime

use rayon::ThreadPoolBuilder;
use std::fmt::Debug;
use std::sync::{LazyLock, OnceLock};
use sts_lib_derive::register_thread_pool;

use crate::Error;

/// The [complementary error function](https://en.wikipedia.org/wiki/Error_function)
pub(crate) use statrs::function::erf::erfc;

/// igamc, the upper regularized incomplete gamma function.
pub(crate) use statrs::function::gamma::checked_gamma_ur as igamc;

/// Checks the f64 value for NaN and Infinite, returns an error if this is the case.
/// This function should be used as a guard.
pub(crate) fn check_f64(value: f64) -> Result<(), Error> {
    if value.is_nan() {
        Err(Error::NaN)
    } else if value.is_infinite() {
        Err(Error::Infinite)
    } else {
        Ok(())
    }
}

/// The number of threads to use in multithreading. Defaults to the number of physical CPUs, which
/// is better for CPU-bound tasks. Note: use [crate::set_max_threads] to set this variable.
pub(crate) static RAYON_THREAD_COUNT: OnceLock<usize> = OnceLock::new();

register_thread_pool! {
    /// The threadpool itself, lazily initialized on first use.
    static THREAD_POOL = LazyLock::new(|| {
        let num_threads = *RAYON_THREAD_COUNT.get_or_init(num_cpus::get_physical);

        ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .thread_name(|idx| format!("sts-{idx}"))
            .build()
            .expect("Could not build STS library thread pool. This should never happen!")
    });
}

/// Trait for primitive types that are used to store bits.
pub(crate) trait BitPrimitive
where
    Self: Sized + Copy + Clone + Debug,
{
    /// How many bits are stored in the primitive
    const BITS: u32;

    /// Get a specific bit from the primitive.
    fn get_bit(self, bit_idx: u32) -> bool;

    /// Return the number of '1' bits in the primitive.
    fn count_ones(self) -> u32;
}

macro_rules! impl_bit_primitive {
    ($primitive: ty) => {
        impl BitPrimitive for $primitive {
            const BITS: u32 = <$primitive>::BITS;

            #[inline]
            fn get_bit(self, bit_idx: u32) -> bool {
                let mask = 1 << (Self::BITS - bit_idx - 1);
                (self & mask) != 0
            }

            #[inline]
            fn count_ones(self) -> u32 {
                <$primitive>::count_ones(self)
            }
        }
    };
    ($($primitive: ty),* $(,)?) => {
        $(impl_bit_primitive!($primitive);)*
    }
}

impl_bit_primitive!(u8, u32, usize);

/// Returns the bit on the given idx from the sequence
#[inline]
pub(crate) fn get_bit_from_sequence<T: BitPrimitive>(seq: &[T], bit_idx: u32) -> bool {
    let word_idx = bit_idx / T::BITS;
    let bit_idx = bit_idx % T::BITS;

    seq[word_idx as usize].get_bit(bit_idx)
}

/// Generate a macro for checked arithmetic that returns a good error message
macro_rules! gen_checked_arithmetic {
    ($method: ident => $op: literal) => {
        macro_rules! $method {
            ($p1: expr, $p2: expr) => {
                $p1.$method($p2)
                    .ok_or_else(|| $crate::Error::Overflow(format!("{} ({}) {} {} ({})", $p1, stringify!($p1), $op, $p2, stringify!($p2))))
            }
        }
    };
    ($m: ident => $op: literal, $($m2: ident => $o2: literal),+ $(,)?) => {
        gen_checked_arithmetic!($m => $op);
        gen_checked_arithmetic!($($m2 => $op),+);
    }
}

gen_checked_arithmetic! {
    checked_add => '+',
    checked_add_unsigned => '+',
    checked_sub_unsigned => '-',
    checked_mul => '*',
}

#[allow(clippy::single_component_path_imports)]
pub(super) use {checked_add, checked_add_unsigned, checked_mul, checked_sub_unsigned};
