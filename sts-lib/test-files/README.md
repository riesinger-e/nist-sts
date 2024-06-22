# Test files

This directory contains the various test files used for testing the functionality. These test files all contain
a binary sequence to be tested (for a specific test). The contained data is the sequence as a byte stream, with full
bytes only.

## Naming convention

The naming convention is as follows: `<SOURCE>.<LENGTH>.bin` where

* `<SOURCE>` describes the source of the sequence, e.g. if the sequence contains digits of *e*, it would be `e`.
* `<LENGTH>` describes the data length in 
  * scientific notation, e.g. for 100 000 bits, it would be `1e5`.
  * expressed with binary prefixes, e.g. for 2^20 bits, it would be `1Mi`.

A full example would be: `e.1e5.bin` or `pi.1e3.bin` or `sha1.1Mi.bin`.

## special sources

The source `sha1` denotes the G-SHA-1 generators, as defined in FIPS 186-2.

