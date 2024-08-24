# sts-cbindings

This projects exposes a C API for the *sts-lib*.

## Usage

In general, the interface tries to stay as close as possible to the Rust interface, but there are some key differences.

### Error handling

Every fallible function either returns a status code or a (nullable) pointer. The exact return value if an error happens is 
documented for each function.

If an error happened, the function `int get_last_error(char *ptr, size_t *len)` can be used to receive the exact error code 
and a user-readable error message.

This function works in 2 stages:
1. `get_last_error(NULL, &len)` is called. The error code is returned and the needed buffer size is written to `len`.
2. `get_last_error(buffer, &len)` is called. The error code is returned and the error message is written to the passed buffer.

### Allocations

All allocations of library-defined types are handled by the corresponding functions. Pointers allocated by this library may not 
be `free()`'d, but instead must be passed to their destruction functions.

### Run a single test

A test may be run by calling the appropriate function. Each test either returns a heap-allocated `TestResult`, or a 
heap-allocated list of `TestResult`. If the returned pointer is `NULL`, an error happened.

The length of a heap-allocated list is either returned via an out-pointer argument, or documented.

#### Example

```c++
BitVec *data = bitvec_from_str("01000100010");

// example of error handling
if (data == NULL) {
    size_t length = 0;
    int error_code = get_last_error(NULL, &length);
    char* buffer = malloc(sizeof(char) * length);
    error_code = get_last_error(buffer, &length);
    
    printf("Error (Code %d): %s", error_code, buffer);
    return;
}

TestResult *result = frequency_test(data);
// do error handling...

printf("P-Value: %lf", test_result_get_p_value(result));

test_result_destroy(result);
bitvec_destroy(data);
```

### Run multiple tests

For running multiple results, a runner struct is used. On calling the appropriate runner function, all tests
are run and a status code is returned, possibly indicating that an error happened. If the error happened while
executing a test, all other tests are still run and their results can still be retrieved.

Test results are retrieved via `test_runner_get_result()`. Once retrieved, the same result cannot be retrieved again. 

#### Example

```c++
BitVec *data = bitvec_from_str("01000100010");
// error handling ...

// create the runner and run tests
TestRunner *runner = test_runner_new();
int result = test_runner_run_all_automatic(runner, data);
// error handling if result != 0...

// get the results for a test and do something with them
size_t length = 0;
TestResult **results = test_runner_get_result(runner, Test_Frequency, &length);
// check errors, size, ...
printf("P-Value: %lf", test_result_get_p_value(results[0]));
// do something with the other results...

test_result_list_destroy(results, length);
test_runner_destroy(runner);
bitvec_destroy(data);
```

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

## How to use

Once you have the library file, you can use it, along with the header file, just like a normal C library.

If suing the static library, you may need to link `libm` (e.g. `gcc -lm`) on operating systems that
split the maths library from the standard C library.