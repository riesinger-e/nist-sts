#!/bin/sh

# generates the header, needs installed:
# - rustup
# - a nightly toolchain
# - cbindgen

rustup run nightly cbindgen --config ./cbindgen.toml --crate sts-cbindings --output ./sts-lib.h