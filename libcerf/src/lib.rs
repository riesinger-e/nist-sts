/// A safe FFI abstraction around [libcerf](https://jugit.fz-juelich.de/mlz/libcerf/-/blob/main/CMakeLists.txt?ref_type=heads).
/// The documentation of all exported functions is taken from the library.
mod bindings;

/// Compute w(z) = exp(-z^2) erfc(-iz), Faddeeva's scaled complex error function.
/// Special case: Im[w(x)] of real x
pub fn im_w_of_x(x: f64) -> f64 {
    // SAFETY: nothing unsafe about passing a f64
    unsafe { bindings::wrapper_im_w_of_x(x) }
}

/// Compute w(z) = exp(-z^2) erfc(-iz), Faddeeva's scaled complex error function.
/// Real part.
pub fn re_w_of_z(x: f64, y: f64) -> f64 {
    // SAFETY: nothing unsafe about passing a f64
    unsafe { bindings::wrapper_re_w_of_z(x, y) }
}

/// Compute w(z) = exp(-z^2) erfc(-iz), Faddeeva's scaled complex error function.
/// Imaginary part.
pub fn im_w_of_z(x: f64, y: f64) -> f64 {
    // SAFETY: nothing unsafe about passing a f64
    unsafe { bindings::wrapper_im_w_of_z(x, y) }
}

/// Compute erfcx(z) = exp(z^2) erfc(z), an underflow-compensated version of erfc
pub fn erfcx(x: f64) -> f64 {
    // SAFETY: nothing unsafe about passing a f64
    unsafe { bindings::wrapper_erfcx(x) }
}

/// Compute erfi(z) = -i erf(iz), the imaginary error function
pub fn erfi(x: f64) -> f64 {
    // SAFETY: nothing unsafe about passing a f64
    unsafe { bindings::wrapper_erfi(x) }
}

/// Compute dawson(z) = sqrt(pi)/2 * exp(-z^2) * erfi(z), Dawson's integral
pub fn dawson(x: f64) -> f64 {
    // SAFETY: nothing unsafe about passing a f64
    unsafe { bindings::wrapper_dawson(x) }
}

/// Compute voigt(x,...), the convolution of a Gaussian and a Lorentzian
pub fn voigt(x: f64, sigma: f64, gamma: f64) -> f64 {
    // SAFETY: nothing unsafe about passing a f64
    unsafe { bindings::wrapper_voigt(x, sigma, gamma) }
}

/// Compute the full width at half maximum of the Voigt function
pub fn voigt_hwhm(sigma: f64, gamma: f64) -> f64 {
    // SAFETY: nothing unsafe about passing a f64
    unsafe { bindings::wrapper_voigt_hwhm(sigma, gamma) }
}
