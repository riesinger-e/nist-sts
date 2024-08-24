## libcerf - a safe FFI abstraction

This library builds a safe FFI abstraction
for [libcerf](https://jugit.fz-juelich.de/mlz/libcerf/-/blob/main/CMakeLists.txt?ref_type=heads). For compatibility with
Windows and Linux, complex types are not supported.

## cerf-wrapper

This is a C++ wrapper around libcerf, exporting C functions. Because Windows does not support the C complex type,
the library has to compiled as C++, the functions that do not use complex types reexported as C functions and then
linked with the Rust project.

## Building

In addition to the Rust tooling ([rustup](https://rustup.rs)), you need [cmake](https://cmake.org),
[clang](https://clang.llvm.org) (on windows, you must also set the environment variable `LIBCLANG_PATH`
to the directory where the LLVM executables reside) and a C/C++ compiler (MSVC on Windows) to build
the library.