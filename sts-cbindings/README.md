# sts-cbindings

This projects exposes a C API for the *sts-lib*.

## Usage

TODO

thread local error printing, how to run a single test, how to use TestRunner

## How to build

You need the Rust tooling, i.e. [rustup](https://rustup.rs/) with a stable Rust toolchain.

Execute this command:
```
cargo build -p sts-cbindings --release
```

Afterward, you will have a dynamic and a static library in `<REPO_PATH>/target/release`.

These are named as follows:

|            | Linux       | macOS           | Windows (MinGW)  | Windows (MSVC)  |
|------------|-------------|-----------------|------------------|-----------------|
| dylib      | `libsts.so` | `libsts.dylib`  | `libsts.dll`     | `libsts.dll`    |
| staticlib  | `libsts.a`  | `libsts.a`      | `libsts.a`       | `libsts.lib`    |

If the public interface changed and you need to re-generate the header file, use
the script `generate-header.sh`. To use this script, you need a *nightly* Rust toolchain and
`cbindgen` (`cargo install cbindgen`).

TODO: Makefile