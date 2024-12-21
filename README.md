# nist-sts

This is a Rust implementation of the statistical test suite described in
"A Statistical Test Suite for Random and Pseudorandom Number Generators for Cryptographic Applications",
[SP 800-22r1a](https://doi.org/10.6028/NIST.SP.800-22r1a).

Note that on x86_64 architectures (Windows, Linux, and macOS), x86-64-v3 is targeted, meaning your computer's CPU
needs to be Intel Haswell or newer / AMD Excavator or newer. Any x86_64 CPU from 2015 onwards should work.

To build for unsupported x86_64 CPUs, simply remove the corresponding line in `.cargo/config.toml`.

Additionally, note that only CPU architecture with a target_pointer_size of 32 or 64 bit are supported.
This should not be a real restriction, because who uses a 16-bit CPU today?

In addition to the Rust tooling ([rustup](https://rustup.rs)), you need [cmake](https://cmake.org), 
[clang](https://clang.llvm.org) (on windows, you must also set the environment variable `LIBCLANG_PATH`
to the directory where the LLVM executables reside) and a C/C++ compiler (MSVC on Windows) to build 
the library.

This code is under *MIT license*.

## Crates

This repository contains several crates:

1. `sts-lib` - the library that implements the statistical tests. See `sts-lib/README.md`.
2. `sts-cmd` - a command line application as a frontend for `sts-lib`. See `sts-cmd/README.md`.
3. `sts-cbindings` - a C API frontend for `sts-lib`. See `sts-cbindings/README.md`.
4. `sts-pybindings` - a Python API frontend for `sts-lib`. See `sts-pybindings/README.md`.
5. `scripts` - several scripts used to calculate constants / do conversion operations. The results of these scripts
    are used in `sts-lib`. This crate is contained in the folder `const-calculation-scripts`, which also contains
    additional python scripts with the same purpose.
6. `benchmarking` - contains a README on how to benchmark against the NIST reference implementation, and a command line
    executable to do that.

### Build all libraries and the command line application

You need the Rust tooling, i.e. [rustup](https://rustup.rs/) with a stable Rust toolchain.

Execute the following command:

```
cargo build --workspace --release
```

To build the python package in a way that it is usable for python, see the instructions in `sts-pybindings/README.md`.

## Using the Rust API

See `sts-lib/README.md`.

*TLDR:*

```rust
use std::fs;
use std::path::Path;
use sts_lib::bitvec::BitVec;
use sts_lib::tests::random_excursions::random_excursions_variant_test;

fn main() {
    let file_path = Path::new("e.1e6.bin");
    let data = fs::read(file_path).unwrap();
    let data = BitVec::from(data);

    let result = random_excursions_variant_test(&data).unwrap();
    println!("P-Value: {}", result.p_value());
}
```

## Using the C API

See `sts-cbindings/README.md`.

*TLDR:*

```c++
BitVec *data = sts_BitVec_from_str("01000100010");
// error handling...

TestResult *result = sts_frequency_test(data);
// do error handling...

printf("P-Value: %lf", sts_TestResult_get_p_value(result));
sts_TestResult_destroy(result);
sts_BitVec_destroy(data);
```

## Using the Python API

See `sts-pybindings/README.md`.

*TLDR:*

```python
import nist_sts
with open("e.1e6.bin", "rb") as f:
   data = nist_sts.BitVec(f.read())
result = nist_sts.tests.longest_runs_of_ones_test(data)
```

## Using the command line application

See `sts-cmd/README.md`.

*TLDR:*

```sh
sts-cmd --input e.1e6.bin --input-format binary --output result.csv
```

## Benchmarking / Performance

This implementation should generally be faster than the reference implementation if the input sequence is large enough.
For small input sequences, the reference implementation might catch up. 

For input sequences with 10^6 bits, run times from ~0.6% to ~55% of the reference implementation were observed 
(measured for each test separately). A full benchmark output can be found in `benchmarking/benchmark_result.txt`.

See also `benchmarking/README.md`.