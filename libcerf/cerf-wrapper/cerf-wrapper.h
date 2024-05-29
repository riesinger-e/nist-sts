//
// Created by Elias Riesinger on 2024-05-29.
// This wrapper exposes all non-complex functions from cerfcpp as C functions.
// The documentation of these functions is taken from libcerf.
//

#ifndef NIST_STS_CERF_WRAPPER_H
#define NIST_STS_CERF_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

#if _WIN32
#define EXPORT __declspec(dllexport)
#else
#define EXPORT
#endif


// compute w(z) = exp(-z^2) erfc(-iz), Faddeeva's scaled complex error function
EXPORT double wrapper_im_w_of_x(double x); // special case Im[w(x)] of real x
EXPORT double wrapper_re_w_of_z(double x, double y);
EXPORT double wrapper_im_w_of_z(double x, double y);

// compute erfcx(z) = exp(z^2) erfc(z), an underflow-compensated version of erfc
EXPORT double wrapper_erfcx(double x); // special case for real x

// compute erfi(z) = -i erf(iz), the imaginary error function
EXPORT double wrapper_erfi(double x); // special case for real x

// compute dawson(z) = sqrt(pi)/2 * exp(-z^2) * erfi(z), Dawson's integral
EXPORT double wrapper_dawson(double x); // special case for real x

// compute voigt(x,...), the convolution of a Gaussian and a Lorentzian
EXPORT double wrapper_voigt(double x, double sigma, double gamma);
// compute the full width at half maximum of the Voigt function
EXPORT double wrapper_voigt_hwhm(double sigma, double gamma);

#ifdef __cplusplus
};
#endif

#endif //NIST_STS_CERF_WRAPPER_H
