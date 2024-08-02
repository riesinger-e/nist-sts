#!/bin/sh

rustup run nightly cbindgen --config ./cbindgen.toml --crate sts-cbindings --output ./header.h