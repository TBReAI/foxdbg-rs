# foxdbg-rs

Very early stages foxglove server built in rust. Building generates a **static** c library and a header file.

Can test by running `make`, which will compile a (non release) rust build, then compile a simple c file to test with.

Or with cmake;

```sh
cmake -S c_test -B build
cmake --build build
```