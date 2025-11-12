# Guide for Dependent Crates Using ggml-rs with Namespacing

This guide explains how to configure dependent crates (like `llama-cpp-rs` and `whisper-rs`) to use `ggml-rs` with namespacing support.

## Overview

When using `ggml-rs` with namespacing, each dependent crate should use a different namespace to avoid symbol conflicts. This allows multiple GGML-based crates to coexist in the same application.

## Namespace Features

- `namespace-llama` - For llama.cpp-based crates (e.g., `llama-cpp-rs`)
- `namespace-whisper` - For whisper.cpp-based crates (e.g., `whisper-rs`)

## Step 1: Update Cargo.toml

### For llama-cpp-rs:

```toml
[dependencies]
ggml-rs = { path = "../ggml-rs", features = ["cuda", "namespace-llama"] }

[features]
default = []
cuda = ["ggml-rs/cuda"]  # Propagate cuda feature to ggml-rs
```

### For whisper-rs:

```toml
[dependencies]
ggml-rs = { path = "../ggml-rs", features = ["cuda", "namespace-whisper"] }

[features]
default = []
cuda = ["ggml-rs/cuda"]  # Propagate cuda feature to ggml-rs
```

## Step 2: Update build.rs

### Important: Do NOT Link to GGML Libraries Directly

`ggml-rs` already handles all linking automatically. Your `build.rs` should **NOT** include lines like:
```rust
// ❌ DON'T DO THIS - ggml-rs handles it
println!("cargo:rustc-link-lib=dylib=ggml");
println!("cargo:rustc-link-lib=dylib=ggml-cuda");
```

### Correct Approach: Use Environment Variables

Your `build.rs` should only use the environment variables provided by `ggml-rs`:

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    #[cfg(feature = "use-shared-ggml")]
    {
        // Get environment variables from ggml-rs
        let root = env::var("DEP_GGML_RS_ROOT")
            .expect("DEP_GGML_RS_ROOT not set. Make sure ggml-rs is in dependencies with the correct namespace feature.");
        
        let include = env::var("DEP_GGML_RS_INCLUDE")
            .expect("DEP_GGML_RS_INCLUDE not set. Make sure ggml-rs is properly configured.");
        
        let lib_dir = env::var("DEP_GGML_RS_LIB_DIR")
            .expect("DEP_GGML_RS_LIB_DIR not set. Make sure ggml-rs is properly configured.");
        
        // Configure CMake to use shared GGML
        // The library names will be automatically namespaced:
        // - For namespace-llama: ggml_llama, ggml_llama-base, ggml_llama-cuda, etc.
        // - For namespace-whisper: ggml_whisper, ggml_whisper-base, ggml_whisper-cuda, etc.
        
        // Add library search path (ggml-rs already links the libraries)
        println!("cargo:rustc-link-search=native={}", lib_dir);
        
        // Configure CMake to find GGML
        // ... your CMake configuration ...
    }
}
```

## Step 3: Verify Library Names

When `ggml-rs` is built with a namespace feature, it creates libraries with namespaced names:

### With `namespace-llama`:
- **Linking files (.lib on Windows, .a/.so on Unix):**
  - `ggml_llama.lib`, `ggml_llama-base.lib`, `ggml_llama-cpu.lib`, `ggml_llama-cuda.lib`
- **Runtime files (.dll on Windows, .so/.dylib on Unix):**
  - `ggml_llama.dll`, `ggml_llama-base.dll`, `ggml_llama-cpu.dll`, `ggml_llama-cuda.dll`

### With `namespace-whisper`:
- **Linking files (.lib on Windows, .a/.so on Unix):**
  - `ggml_whisper.lib`, `ggml_whisper-base.lib`, `ggml_whisper-cpu.lib`, `ggml_whisper-cuda.lib`
- **Runtime files (.dll on Windows, .so/.dylib on Unix):**
  - `ggml_whisper.dll`, `ggml_whisper-base.dll`, `ggml_whisper-cpu.dll`, `ggml_whisper-cuda.dll`

## Step 4: Check Your CMake Configuration

If your crate uses CMake to build additional native code, make sure it uses the shared GGML library:

```cmake
# Find the shared GGML library
find_library(GGML_LIBRARY
    NAMES ggml_llama  # or ggml_whisper depending on namespace
    PATHS ${DEP_GGML_RS_LIB_DIR}
    NO_DEFAULT_PATH
)

if(GGML_LIBRARY)
    message(STATUS "Found GGML library: ${GGML_LIBRARY}")
    target_link_libraries(your_target PRIVATE ${GGML_LIBRARY})
else()
    message(FATAL_ERROR "GGML library not found in ${DEP_GGML_RS_LIB_DIR}")
endif()
```

## Step 5: Runtime DLL Copying

`ggml-rs` automatically copies the namespaced DLLs to `target/debug/` or `target/release/` when built. You don't need to do anything extra.

However, if you need to copy DLLs manually or for distribution:

```rust
// In your build.rs or a separate script
use std::fs;
use std::path::PathBuf;

fn copy_ggml_dlls() {
    let lib_dir = env::var("DEP_GGML_RS_LIB_DIR").unwrap();
    let target_dir = PathBuf::from(env::var("OUT_DIR").unwrap())
        .parent().unwrap()  // out/
        .parent().unwrap()  // build/
        .parent().unwrap()  // debug/
        .parent().unwrap()  // target/
        .join(env::var("PROFILE").unwrap());
    
    // Copy namespaced DLLs
    // The names will be: ggml_llama*.dll or ggml_whisper*.dll
    // depending on which namespace feature is enabled
}
```

## Verification Checklist

Before building your dependent crate, verify:

- [ ] `ggml-rs` is in dependencies with the correct namespace feature
- [ ] Your `build.rs` does NOT link to GGML libraries directly
- [ ] Your `build.rs` uses `DEP_GGML_RS_*` environment variables
- [ ] CMake configuration (if any) uses the namespaced library names
- [ ] No hardcoded references to `ggml`, `ggml-base`, `ggml-cuda`, etc. in your code

## Example: Complete llama-cpp-rs Configuration

```toml
# Cargo.toml
[dependencies]
ggml-rs = { path = "../ggml-rs", features = ["cuda", "namespace-llama"] }

[features]
default = []
cuda = ["ggml-rs/cuda"]
use-shared-ggml = []
```

```rust
// build.rs
fn main() {
    #[cfg(feature = "use-shared-ggml")]
    {
        let include = env::var("DEP_GGML_RS_INCLUDE").unwrap();
        let lib_dir = env::var("DEP_GGML_RS_LIB_DIR").unwrap();
        
        // Configure CMake
        let mut config = cmake::Config::new(".");
        config.define("GGML_INCLUDE_DIR", include);
        config.define("GGML_LIB_DIR", lib_dir);
        // ... rest of CMake configuration
        
        // ggml-rs already handles linking, so we don't need:
        // println!("cargo:rustc-link-lib=dylib=ggml_llama");
    }
}
```

## Troubleshooting

### Error: "cannot find -lggml"
This means you're trying to link to `ggml` directly. Remove any direct linking and let `ggml-rs` handle it.

### Error: "undefined reference to ggml_*"
This means the namespace feature isn't enabled. Make sure you have `namespace-llama` or `namespace-whisper` in your `ggml-rs` dependency features.

### Error: "multiple definition of ggml_*"
This means two crates are using the same namespace. Make sure:
- `llama-cpp-rs` uses `namespace-llama`
- `whisper-rs` uses `namespace-whisper`
- They don't both use the same namespace

## Summary

1. **Add namespace feature** to `ggml-rs` dependency in `Cargo.toml`
2. **Don't link directly** - let `ggml-rs` handle all linking
3. **Use environment variables** - `DEP_GGML_RS_*` variables are automatically set
4. **Verify library names** - check that namespaced libraries are created
5. **Test coexistence** - build both crates and verify they work together

