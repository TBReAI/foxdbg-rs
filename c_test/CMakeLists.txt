cmake_minimum_required(VERSION 3.10)
project(FoxDbgCTest C)

# Set C standard
set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Enable generation of compile_commands.json for LSP support
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Set the output directory for the executable
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR})

# Add the directory where foxdbg.h is located (relative to project root)
include_directories(${CMAKE_SOURCE_DIR}/../include)

# Add the directory where the Rust static library is located (relative to c_test/CMakeLists.txt)
link_directories(${CMAKE_CURRENT_SOURCE_DIR}/../target/release)

# Check if Rust library exists
set(RUST_LIB_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../target/release/foxdbg_rs.lib")
if(NOT EXISTS ${RUST_LIB_PATH})
    message(STATUS "Rust library not found at ${RUST_LIB_PATH}, building...")
    execute_process(
        COMMAND cargo build --release
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/..
        RESULT_VARIABLE CARGO_RESULT
    )
    if(NOT CARGO_RESULT EQUAL 0)
        message(FATAL_ERROR "Cargo build failed with code ${CARGO_RESULT}")
    endif()
else()
    message(STATUS "Rust library found, skipping build")
endif()

# Add the C test executable
add_executable(test_foxdbg test.c)

if (MSVC)
  # Match Rust’s /MD(/MDd) to avoid LNK4098
  set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>DLL")
endif()

# Link against the Rust static library and necessary system libraries
if (WIN32)
    target_link_libraries(
        test_foxdbg
        foxdbg_rs
        ntdll
        bcrypt
        userenv
        advapi32
    )
else()
    target_link_libraries(
        test_foxdbg
        foxdbg_rs
        m
    )
endif()
