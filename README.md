# nist-sts

This is a Rust implementation of the statistical test suite described in
"A Statistical Test Suite for Random and Pseudorandom Number Generators for Cryptographic Applications",
[SP 800-22r1a](https://doi.org/10.6028/NIST.SP.800-22r1a).

Note that on x86_64 architectures (Windows, Linux, and macOS), x86-64-v3 is targeted, meaning your computer's CPU
needs to be Intel Haswell or newer / AMD Excavator or newer. Any x86_64 CPU from 2015 onwards should work.

To build for non-supported CPUs, simply remove the corresponding line in `.cargo/config.toml`.

## Crates

TODO

## Building the C API

TODO: reference appropriate README.md, short TLDR

## Building the Python API

TODO: reference appropriate README.md, short TLDR

## Using the command line application

TODO: reference appropriate README.md, short TLDR