//! Internal functions that are used by tests - can be changed anytime

use libcerf::erfcx;
use rayon::ThreadPoolBuilder;
use std::sync::{LazyLock, OnceLock};
use sts_lib_derive::register_thread_pool;

use crate::Error;

/// The [complementary error function](https://en.wikipedia.org/wiki/Error_function)
pub(crate) fn erfc(value: f64) -> f64 {
    // from https://en.wikipedia.org/wiki/Error_function#Complementary_error_function

    // if arithmetic underflow is observed, switching to pure erfcx would likely help

    let exponent = -(value * value);
    f64::exp(exponent) * erfcx(value)
}

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
