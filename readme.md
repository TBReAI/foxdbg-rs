# foxdbg-rs

Rust library which provides a C-compatible API for sending data to [Foxglove Studio](https://foxglove.dev/). It replicates the functionality of the original [foxdbg](https://github.com/TBReAI/foxdbg) C library, with the addition of MCAP file logging support. 

## Core Functionality

- Streams data to Foxglove Studio over a WebSocket connection.
- Records data to `.mcap` files.
- Exposes a C API for use in non-Rust codebases.
- Supports the following data types:
  - Primitives: `float`, `int`, `bool`
  - Images (JPEG compressed)
  - Point Clouds
  - Scene Primitives: Cubes, Lines, Poses
  - Transforms
  - GPS Locations

## Build and Run Test

Execute the following command in the project root:

```bash
make
```

This command performs the following steps:
1.  Builds the Rust library into a static C library (`libfoxdbg_rs.a`).
2.  Generates the C header file (`include/foxdbg.h`).
3.  Builds the C test application from the `c_test/` directory.
4.  Runs the compiled test application.

## Usage

To use `foxdbg-rs` in a C project, include the `foxdbg.h` header and link against the `libfoxdbg_rs.a` static library.
