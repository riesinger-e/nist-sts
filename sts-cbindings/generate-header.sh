#!/bin/sh

# generates the header, needs installed:
# - rustup
# - a nightly toolchain
# - cbindgen

# generate the header file
rustup run nightly cbindgen --config ./cbindgen.toml --crate sts-cbindings --output ./sts-lib.h

# delete this typedef - enum Test already behaves like an int.
sed -i "/typedef int Test;/d" ./sts-lib.h