//
// Created by Elias Riesinger on 2024-05-29.
//

#include "cerf-wrapper.h"
#include "libcerf/lib/cerf.h"

double wrapper_im_w_of_x(double x) {
    return im_w_of_x(x);
}

double wrapper_re_w_of_z(double x, double y) {
    return re_w_of_z(x, y);
}

double wrapper_im_w_of_z(double x, double y) {
    return im_w_of_z(x, y);
}

double wrapper_erfcx(double x) {
    return erfcx(x);
}

double wrapper_erfi(double x) {
    return erfi(x);
}

double wrapper_dawson(double x) {
    return dawson(x);
}

double wrapper_voigt(double x, double sigma, double gamma) {
    return voigt(x, sigma, gamma);
}

double wrapper_voigt_hwhm(double sigma, double gamma) {
    return voigt_hwhm(sigma, gamma);
}