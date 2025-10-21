.PHONY: all rust ctest test clean

# Define build directories
RUST_TARGET_DIR := target/debug
BUILD_DIR := build
CMAKE_BUILD_DIR := c_test/build

# Executables and libraries
RUST_LIB_NAME := foxdbg_rs
RUST_STATIC_LIB := $(RUST_TARGET_DIR)/lib$(RUST_LIB_NAME).a
C_TEST_EXE := $(CMAKE_BUILD_DIR)/test_foxdbg # CMake will put it here
C_HEADER := include/foxdbg.h

all: test

# Build the Rust static library and generate the C header
rust:
	@echo "Building Rust static library and generating $(C_HEADER)..."
	cargo build

# Build the C test executable using CMake
ctest: rust
	@echo "Configuring and building C test program with CMake..."
	mkdir -p $(CMAKE_BUILD_DIR)
	cmake -S c_test -B $(CMAKE_BUILD_DIR)
	cmake --build $(CMAKE_BUILD_DIR)

# Run the C test program
test: ctest
	@echo "Running C test program..."
	./$(C_TEST_EXE)

clean:
	@echo "Cleaning up..."
	cargo clean
	rm -rf $(BUILD_DIR) $(C_HEADER) $(CMAKE_BUILD_DIR)
