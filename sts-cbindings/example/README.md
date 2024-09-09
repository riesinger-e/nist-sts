# Example code for the C bindings

This is example code for the STS C bindings. The example code provides a minimal 
command line application that reads the given byte input file and runs all available 
tests on it.

## On Linux

Copy sts-lib.h and the built dynamic and static library into the `lib` directory. 
Use the provided `CMakeLists.txt` with CMake. There are two targets:

1. `nist_sts_dyn`: Links to the dynamic library.
2. `nist_sts_static`: Links to the static library.

Both targets do the same.

## On Windows

TODO