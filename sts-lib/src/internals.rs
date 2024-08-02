//! Internal functions that are used by tests - can be changed anytime

use libcerf::erfcx;

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