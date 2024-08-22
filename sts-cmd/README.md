# sts-cmd

This library implements a command line interface for *sts-lib*, which implements the statistical tests of
NIST SP 800-22r1a.

## Usage

You can either specify everything with command line arguments, or use a TOML config file to specify
all arguments, with command line arguments overriding the corresponding options in the config file.

The application always prints the test result to the command line output, and optionally saves them as
CSV to a specified location.

A reference to the TOML config file can be seen in `sts-example.toml`, which specifies every available
option, and describes them.

Use the command line option `--help` to see all available arguments.

## Examples

#### Run all tests with command line arguments, saving the output to result.csv

```sh
sts-cmd --input e.1e6.bin --input-format binary --output result.csv
```

#### Run only specified tests with command line arguments

```sh
sts-cmd --input e.1e6.bin --input-format binary --tests frequency,runs,cumulative-sums
```

#### Use a config file

```sh
sts-cmd --config-file config.toml
```

### Use a config file, overriding the input file and two test arguments

```sh
sts-cmd --config-file config.toml --input e.1e6.bin --input-format binary \
  --overrides serial.block-length=10,frequency-block.block-length=13
```