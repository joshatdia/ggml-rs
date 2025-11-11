# ggml-sys - Shared GGML Crate Setup Guide

## Overview

This document describes how to set up `ggml-sys` as a standalone Rust crate that builds GGML as a shared library (DLL/.so). This shared library can then be used by both `whisper-rs` and `llama-cpp-2` to avoid duplicate symbol conflicts.

## Purpose

- Build GGML once as a shared library
- Provide Rust FFI bindings for GGML
- Allow multiple crates (`whisper-rs`, `llama-cpp-2`) to link against the same GGML library
- Avoid duplicate symbol conflicts when using both whisper and llama-cpp together

## Repository Structure

```
ggml-sys/
├── Cargo.toml
├── build.rs
├── wrapper.h
├── src/
│   └── lib.rs
└── ggml/          # GGML source code (copy from llama.cpp or whisper.cpp)
    ├── CMakeLists.txt
    ├── cmake/
    │   └── ggml-config.cmake.in
    ├── include/
    │   ├── ggml.h
    │   ├── gguf.h
    │   └── ... (other headers)
    └── src/
        ├── ggml.c
        ├── ggml.cpp
        └── ... (other source files)
```

## Step 1: Create the Repository

1. Create a new GitHub repository (e.g., `your-username/ggml-sys`)
2. Clone it locally
3. Initialize as a Rust crate: `cargo init --lib`

## Step 2: Copy GGML Source Code

Copy the `ggml/` directory from either:
- `llama.cpp/ggml/` (recommended - more complete)
- `whisper.cpp/ggml/`

Place it in the root of your `ggml-sys` repository.

## Step 3: Cargo.toml Configuration

Replace the contents of `Cargo.toml` with:

```toml
[package]
name = "ggml-sys"
version = "0.1.0"
edition = "2021"
description = "Rust FFI bindings for GGML (shared library)"
license = "MIT OR Apache-2.0"
repository = "https://github.com/your-username/ggml-sys"
links = "ggml"  # CRITICAL: Prevents multiple crates from linking the same library

[features]
default = []
metal = []
cuda = []
vulkan = []
openblas = []
openmp = []
hipblas = []
intel-sycl = []

[build-dependencies]
cmake = "0.1"
bindgen = "0.71"
cc = { version = "1.0", features = ["parallel"] }

[dependencies]
```

## Step 4: Create wrapper.h

Create `wrapper.h` in the root directory:

```c
#include "ggml/include/ggml.h"
#include "ggml/include/gguf.h"
```

## Step 5: Create build.rs

Create `build.rs` in the root directory with the following content:

```rust
#![allow(clippy::uninlined_format_args)]

extern crate bindgen;

use cmake::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    
    // Link C++ standard library
    if let Some(cpp_stdlib) = get_cpp_link_stdlib(&target) {
        println!("cargo:rustc-link-lib=dylib={}", cpp_stdlib);
    }
    
    // Link macOS Accelerate framework for matrix calculations
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=framework=Accelerate");
        #[cfg(feature = "metal")]
        {
            println!("cargo:rustc-link-lib=framework=Foundation");
            println!("cargo:rustc-link-lib=framework=Metal");
            println!("cargo:rustc-link-lib=framework=MetalKit");
        }
    }

    #[cfg(feature = "openblas")]
    {
        if let Ok(openblas_path) = env::var("OPENBLAS_PATH") {
            println!(
                "cargo:rustc-link-search={}",
                PathBuf::from(openblas_path).join("lib").display()
            );
        }
        if cfg!(windows) {
            println!("cargo:rustc-link-lib=libopenblas");
        } else {
            println!("cargo:rustc-link-lib=openblas");
        }
    }

    #[cfg(feature = "cuda")]
    {
        println!("cargo:rustc-link-lib=cublas");
        println!("cargo:rustc-link-lib=cudart");
        println!("cargo:rustc-link-lib=culibos");
    }

    // Get the manifest directory and locate ggml source
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
    let manifest_path = PathBuf::from(&manifest_dir);
    let ggml_root = manifest_path.join("ggml");

    if !ggml_root.exists() {
        panic!("GGML source directory not found at: {}", ggml_root.display());
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", ggml_root.join("include").display()))
        .allowlist_function("ggml_.*")
        .allowlist_type("ggml_.*")
        .allowlist_function("gguf_.*")
        .allowlist_type("gguf_.*")
        .allowlist_var("GGML_.*")
        .allowlist_var("GGUF_.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = out_dir.join("bindings.rs");
    bindings
        .write_to_file(&out_path)
        .expect("Couldn't write bindings!");

    // Stop if we're on docs.rs
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    // Build ggml as shared library using CMake
    let mut config = Config::new(&ggml_root);

    config
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "ON")  // CRITICAL: Build as shared library
        .define("GGML_ALL_WARNINGS", "OFF")
        .define("GGML_ALL_WARNINGS_3RD_PARTY", "OFF")
        .define("GGML_BUILD_TESTS", "OFF")
        .define("GGML_BUILD_EXAMPLES", "OFF")
        .very_verbose(true)
        .pic(true);

    if cfg!(target_os = "windows") {
        config.cxxflag("/utf-8");
    }

    if cfg!(feature = "cuda") {
        config.define("GGML_CUDA", "ON");
        config.define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");
        config.define("CMAKE_CUDA_FLAGS", "-Xcompiler=-fPIC");
    }

    if cfg!(feature = "hipblas") {
        config.define("GGML_HIP", "ON");
        config.define("CMAKE_C_COMPILER", "hipcc");
        config.define("CMAKE_CXX_COMPILER", "hipcc");
        println!("cargo:rerun-if-env-changed=AMDGPU_TARGETS");
        if let Ok(gpu_targets) = env::var("AMDGPU_TARGETS") {
            config.define("AMDGPU_TARGETS", gpu_targets);
        }
    }

    if cfg!(feature = "vulkan") {
        config.define("GGML_VULKAN", "ON");
        if cfg!(windows) {
            println!("cargo:rerun-if-env-changed=VULKAN_SDK");
            println!("cargo:rustc-link-lib=vulkan-1");
            let vulkan_path = match env::var("VULKAN_SDK") {
                Ok(path) => PathBuf::from(path),
                Err(_) => panic!(
                    "Please install Vulkan SDK and ensure that VULKAN_SDK env variable is set"
                ),
            };
            let vulkan_lib_path = vulkan_path.join("Lib");
            println!("cargo:rustc-link-search={}", vulkan_lib_path.display());
        } else if cfg!(target_os = "macos") {
            println!("cargo:rerun-if-env-changed=VULKAN_SDK");
            println!("cargo:rustc-link-lib=vulkan");
            let vulkan_path = match env::var("VULKAN_SDK") {
                Ok(path) => PathBuf::from(path),
                Err(_) => panic!(
                    "Please install Vulkan SDK and ensure that VULKAN_SDK env variable is set"
                ),
            };
            let vulkan_lib_path = vulkan_path.join("lib");
            println!("cargo:rustc-link-search={}", vulkan_lib_path.display());
        } else {
            println!("cargo:rustc-link-lib=vulkan");
        }
    }

    if cfg!(feature = "openblas") {
        config.define("GGML_BLAS", "ON");
        config.define("GGML_BLAS_VENDOR", "OpenBLAS");
        if env::var("BLAS_INCLUDE_DIRS").is_err() {
            panic!("BLAS_INCLUDE_DIRS environment variable must be set when using OpenBLAS");
        }
        config.define("BLAS_INCLUDE_DIRS", env::var("BLAS_INCLUDE_DIRS").unwrap());
        println!("cargo:rerun-if-env-changed=BLAS_INCLUDE_DIRS");
    }

    if cfg!(feature = "metal") {
        config.define("GGML_METAL", "ON");
        config.define("GGML_METAL_NDEBUG", "ON");
        config.define("GGML_METAL_EMBED_LIBRARY", "ON");
    } else {
        // Metal is enabled by default on macOS, so we need to explicitly disable it
        if target.contains("apple") {
            config.define("GGML_METAL", "OFF");
        }
    }

    if cfg!(not(feature = "openmp")) {
        config.define("GGML_OPENMP", "OFF");
    }

    if cfg!(feature = "intel-sycl") {
        config.define("GGML_SYCL", "ON");
        config.define("GGML_SYCL_TARGET", "INTEL");
        config.define("CMAKE_C_COMPILER", "icx");
        config.define("CMAKE_CXX_COMPILER", "icpx");
    }

    // Allow passing any GGML or CMAKE compile flags
    for (key, value) in env::vars() {
        let is_ggml_flag = key.starts_with("GGML_");
        let is_cmake_flag = key.starts_with("CMAKE_");
        if is_ggml_flag || is_cmake_flag {
            config.define(&key, &value);
        }
    }

    let destination = config.build();

    // Export the library path for CMake to find
    let lib_dir = destination.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    
    // Export library path as environment variable for CMake find_package
    println!("cargo:GGML_LIB_DIR={}", lib_dir.display());
    println!("cargo:GGML_INCLUDE_DIR={}", ggml_root.join("include").display());
    
    // CRITICAL: Link to shared libraries (not static)
    println!("cargo:rustc-link-lib=dylib=ggml");
    println!("cargo:rustc-link-lib=dylib=ggml-base");
    println!("cargo:rustc-link-lib=dylib=ggml-cpu");
    
    if cfg!(target_os = "macos") || cfg!(feature = "openblas") {
        println!("cargo:rustc-link-lib=dylib=ggml-blas");
    }
    
    if cfg!(feature = "vulkan") {
        println!("cargo:rustc-link-lib=dylib=ggml-vulkan");
    }

    if cfg!(feature = "hipblas") {
        println!("cargo:rustc-link-lib=dylib=ggml-hip");
    }

    if cfg!(feature = "metal") {
        println!("cargo:rustc-link-lib=dylib=ggml-metal");
    }

    if cfg!(feature = "cuda") {
        println!("cargo:rustc-link-lib=dylib=ggml-cuda");
    }

    if cfg!(feature = "openblas") {
        println!("cargo:rustc-link-lib=dylib=ggml-blas");
    }

    if cfg!(feature = "intel-sycl") {
        println!("cargo:rustc-link-lib=dylib=ggml-sycl");
    }
}

// From https://github.com/alexcrichton/cc-rs/blob/fba7feded71ee4f63cfe885673ead6d7b4f2f454/src/lib.rs#L2462
fn get_cpp_link_stdlib(target: &str) -> Option<&'static str> {
    if target.contains("msvc") {
        None
    } else if target.contains("apple") || target.contains("freebsd") || target.contains("openbsd") {
        Some("c++")
    } else if target.contains("android") {
        Some("c++_shared")
    } else {
        Some("stdc++")
    }
}
```

## Step 6: Create src/lib.rs

Create `src/lib.rs`:

```rust
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

## Step 7: Create ggml.pc.in (if needed)

If the GGML CMakeLists.txt requires `ggml.pc.in`, create it in `ggml/`:

```
prefix=@CMAKE_INSTALL_PREFIX@
libdir=@CMAKE_INSTALL_LIBDIR@
includedir=@CMAKE_INSTALL_INCLUDEDIR@

Name: ggml
Description: GGML library
Version: 0.1.0
Libs: -L${libdir} -lggml
Cflags: -I${includedir}
```

## Step 8: Test the Build

1. Build the crate: `cargo build`
2. Verify that shared libraries are created:
   - Windows: `target/debug/build/ggml-sys-*/out/lib/ggml.dll`
   - Linux: `target/debug/build/ggml-sys-*/out/lib/libggml.so`
   - macOS: `target/debug/build/ggml-sys-*/out/lib/libggml.dylib`

## Step 9: Publish or Use as Git Dependency

### Option A: Publish to crates.io

1. Update version in `Cargo.toml`
2. `cargo publish`

### Option B: Use as Git Dependency

In your main project's `Cargo.toml`:

```toml
[dependencies]
ggml-sys = { git = "https://github.com/your-username/ggml-sys.git", branch = "main" }
```

## Critical Points

1. **`BUILD_SHARED_LIBS=ON`**: Must be set to build as shared library
2. **`links = "ggml"`**: Prevents multiple crates from linking the same library
3. **`dylib=ggml`**: Links to dynamic library, not static
4. **Export paths**: `GGML_LIB_DIR` and `GGML_INCLUDE_DIR` are exported for dependent crates

## Verification Checklist

- [ ] `BUILD_SHARED_LIBS=ON` is set in build.rs
- [ ] Links to `dylib=ggml` (not `static=ggml`)
- [ ] `links = "ggml"` in Cargo.toml
- [ ] GGML source code exists in `ggml/` directory
- [ ] Shared libraries are generated after build
- [ ] `GGML_LIB_DIR` and `GGML_INCLUDE_DIR` are exported

## Troubleshooting

### CMake can't find GGML source
- Ensure `ggml/` directory exists in the crate root
- Check that `ggml/CMakeLists.txt` exists

### Duplicate symbol errors
- Ensure `links = "ggml"` is set in Cargo.toml
- Verify you're linking to `dylib=ggml`, not `static=ggml`

### Build fails with missing headers
- Ensure `ggml/include/` contains all necessary headers
- Check that `wrapper.h` includes the correct paths

