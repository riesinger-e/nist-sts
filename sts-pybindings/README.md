# sts-pybindings

This projects exposes a Python API for the *sts-lib*. Requires at least Python 3.7.

## Usage

In general, the interface tries to stay as close as possible to the Rust interface, while being pythonic.

### Run a single test

If a test requires an argument, it is optional. Each test returns either one `TestResult`, a fixed-length tuple of `TestResult`
or a dynamic-length list of `TestResult`. If a test encounters an error, a `TestError` is thrown.

#### Example

```python
import nist_sts
with open("e.1e6.bin", "rb") as f:
   data = nist_sts.BitVec(f.read())
result = nist_sts.tests.longest_runs_of_ones_test(data)
```

### Run multiple tests

For the test runner, the different methods from the Rust API have been condensed into one method `run_tests()` with optional arguments.
The return type is a lazily-evaluated iterator of tuples, containing the `Test` (enum) as the first value and the result 
as the second value.

Each returned test result can either be one `TestResult`, or a dynamic-length list of `TestResult`.
If a test encounters an error, a `TestError` is thrown.

If invalid arguments are specified to `run_tests()`, a `RunnerError` is thrown immediately.

#### Example

```python
import nist_sts
with open("e.1e6.bin", "rb") as f:
    data = nist_sts.BitVec(f.read())
for test, result in nist_sts.run_tests(data):
    print(f"Test {test}: {result}")
```

## How to build

1. Setup a python virtual env and enter it.
   ```sh
   python -m venv ./venv
   # most linux shells
   source ./venv/bin/activate
   # PowerShell on Windows
   .\venv\Scripts\Activate.ps1
   ```
2. Install [maturin](https://github.com/PyO3/maturin): `pip install maturin`
3. For development / debugging:
   1. Run `maturin develop`
   2. Run `python` and use the package.
4. For releasing the package:
   1. Run `maturin build --release`
   2. The `.whl` can then be found in the location indicated by the output.
