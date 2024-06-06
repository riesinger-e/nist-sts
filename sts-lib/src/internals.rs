//! Internal functions that are used by tests - can be changed anytime

use libcerf::erfcx;
use std::f64::consts::E;

/// The [complementary error function](https://en.wikipedia.org/wiki/Error_function)
pub(crate) fn erfc(value: f64) -> f64 {
    // from https://en.wikipedia.org/wiki/Error_function#Complementary_error_function

    // if arithmetic underflow is observed, switching to pure erfcx would likely help

    let exponent = -(value * value);
    E.powf(exponent) * erfcx(value)
}