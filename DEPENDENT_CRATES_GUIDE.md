# Guide for Dependent Crates Using ggml-rs with Namespacing

This guide explains how to configure dependent crates (like `llama-cpp-rs` and `whisper-rs`) to use `ggml-rs` with namespacing support.

## Overview

`ggml-rs` **automatically builds BOTH variants** (llama and whisper) unconditionally. This ensures both sets of libraries are available regardless of which dependent crate builds first, avoiding Cargo's feature unification issues.

Each dependent crate links to its own variant using environment variables exported by `ggml-rs`.

## Important: No Namespace Features Needed

**You do NOT need to enable namespace features in `ggml-rs`**. The crate builds both variants automatically.

## Step 1: Update Cargo.toml

### For llama-cpp-rs:

```toml
[dependencies]
ggml-rs = { path = "../ggml-rs", features = ["cuda"] }  # NO namespace feature needed!

[features]
default = []
cuda = ["ggml-rs/cuda"]  # Propagate cuda feature to ggml-rs
```

### For whisper-rs:

```toml
[dependencies]
ggml-rs = { path = "../ggml-rs", features = ["cuda"] }  # NO namespace feature needed!

[features]
default = []
cuda = ["ggml-rs/cuda"]  # Propagate cuda feature to ggml-rs
```

**Note:** Both crates use the same `ggml-rs` dependency with the same features. `ggml-rs` builds both variants internally.

## Step 2: Update build.rs

### Important: Link to Your Own Variant

`ggml-rs` builds both variants but **does NOT auto-link**. Each dependent crate must link to its own variant.

### For llama-cpp-rs:

```rust
use std::env;

fn main() {
    // Get environment variables from ggml-rs for the llama variant
    let lib_dir = env::var("DEP_GGML_RS_GGML_LLAMA_LIB_DIR")
        .expect("GGML_LLAMA_LIB_DIR not set. Make sure ggml-rs is in dependencies.");
    
    let bin_dir = env::var("DEP_GGML_RS_GGML_LLAMA_BIN_DIR")
        .expect("GGML_LLAMA_BIN_DIR not set. Make sure ggml-rs is in dependencies.");
    
    let base_name = env::var("DEP_GGML_RS_GGML_LLAMA_BASENAME")
        .unwrap_or_else(|_| "ggml_llama".to_string());
    
    // Link to llama variant libraries
    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=dylib={}", base_name);  // -> ggml_llama
    println!("cargo:rustc-link-lib=dylib={}-base", base_name);  // -> ggml_llama-base
    println!("cargo:rustc-link-lib=dylib={}-cpu", base_name);  // -> ggml_llama-cpu
    
    // Link to CUDA if enabled
    #[cfg(feature = "cuda")]
    {
        println!("cargo:rustc-link-lib=dylib={}-cuda", base_name);  // -> ggml_llama-cuda
    }
    
    // Export runtime directory for DLL copying (Windows)
    println!("cargo:rustc-env=GGML_LLAMA_RUNTIME_DIR={}", bin_dir);
    
    // Configure CMake to use shared GGML
    // ... your CMake configuration ...
}
```

### For whisper-rs:

```rust
use std::env;

fn main() {
    // Get environment variables from ggml-rs for the whisper variant
    let lib_dir = env::var("DEP_GGML_RS_GGML_WHISPER_LIB_DIR")
        .expect("GGML_WHISPER_LIB_DIR not set. Make sure ggml-rs is in dependencies.");
    
    let bin_dir = env::var("DEP_GGML_RS_GGML_WHISPER_BIN_DIR")
        .expect("GGML_WHISPER_BIN_DIR not set. Make sure ggml-rs is in dependencies.");
    
    let base_name = env::var("DEP_GGML_RS_GGML_WHISPER_BASENAME")
        .unwrap_or_else(|_| "ggml_whisper".to_string());
    
    // Link to whisper variant libraries
    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=dylib={}", base_name);  // -> ggml_whisper
    println!("cargo:rustc-link-lib=dylib={}-base", base_name);  // -> ggml_whisper-base
    println!("cargo:rustc-link-lib=dylib={}-cpu", base_name);  // -> ggml_whisper-cpu
    
    // Link to CUDA if enabled
    #[cfg(feature = "cuda")]
    {
        println!("cargo:rustc-link-lib=dylib={}-cuda", base_name);  // -> ggml_whisper-cuda
    }
    
    // Export runtime directory for DLL copying (Windows)
    println!("cargo:rustc-env=GGML_WHISPER_RUNTIME_DIR={}", bin_dir);
    
    // Configure CMake to use shared GGML
    // ... your CMake configuration ...
}
```

## Step 3: Environment Variables Available

`ggml-rs` exports the following environment variables (accessible via `DEP_GGML_RS_*`):

### For llama variant:
- `DEP_GGML_RS_GGML_LLAMA_LIB_DIR` - Path to llama variant library directory
- `DEP_GGML_RS_GGML_LLAMA_BIN_DIR` - Path to llama variant binary directory (DLLs)
- `DEP_GGML_RS_GGML_LLAMA_BASENAME` - Base library name: `ggml_llama`

### For whisper variant:
- `DEP_GGML_RS_GGML_WHISPER_LIB_DIR` - Path to whisper variant library directory
- `DEP_GGML_RS_GGML_WHISPER_BIN_DIR` - Path to whisper variant binary directory (DLLs)
- `DEP_GGML_RS_GGML_WHISPER_BASENAME` - Base library name: `ggml_whisper`

### Common:
- `DEP_GGML_RS_INCLUDE` - Path to GGML include directory (same for both variants)

## Step 4: Verify Library Names

`ggml-rs` builds both variants automatically. The libraries have namespaced names:

### llama variant:
- **Linking files (.lib on Windows, .a/.so on Unix):**
  - `ggml_llama.lib`, `ggml_llama-base.lib`, `ggml_llama-cpu.lib`, `ggml_llama-cuda.lib`
- **Runtime files (.dll on Windows, .so/.dylib on Unix):**
  - `ggml_llama.dll`, `ggml_llama-base.dll`, `ggml_llama-cpu.dll`, `ggml_llama-cuda.dll`

### whisper variant:
- **Linking files (.lib on Windows, .a/.so on Unix):**
  - `ggml_whisper.lib`, `ggml_whisper-base.lib`, `ggml_whisper-cpu.lib`, `ggml_whisper-cuda.lib`
- **Runtime files (.dll on Windows, .so/.dylib on Unix):**
  - `ggml_whisper.dll`, `ggml_whisper-base.dll`, `ggml_whisper-cpu.dll`, `ggml_whisper-cuda.dll`

## Step 5: Check Your CMake Configuration

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

## Step 6: Runtime DLL Copying

Each dependent crate should copy its own variant's DLLs to the target directory. `ggml-rs` provides the binary directory path via environment variables.

### For llama-cpp-rs:

```rust
// In your build.rs
use std::fs;
use std::path::PathBuf;

fn copy_llama_dlls() {
    let bin_dir = env::var("DEP_GGML_RS_GGML_LLAMA_BIN_DIR").unwrap();
    let target_dir = PathBuf::from(env::var("OUT_DIR").unwrap())
        .parent().unwrap()  // out/
        .parent().unwrap()  // build/
        .parent().unwrap()  // debug/
        .parent().unwrap()  // target/
        .join(env::var("PROFILE").unwrap());
    
    // Copy all ggml_llama*.dll files from bin_dir to target_dir
    // ... implementation ...
}
```

### For whisper-rs:

```rust
// In your build.rs
use std::fs;
use std::path::PathBuf;

fn copy_whisper_dlls() {
    let bin_dir = env::var("DEP_GGML_RS_GGML_WHISPER_BIN_DIR").unwrap();
    let target_dir = PathBuf::from(env::var("OUT_DIR").unwrap())
        .parent().unwrap()  // out/
        .parent().unwrap()  // build/
        .parent().unwrap()  // debug/
        .parent().unwrap()  // target/
        .join(env::var("PROFILE").unwrap());
    
    // Copy all ggml_whisper*.dll files from bin_dir to target_dir
    // ... implementation ...
}
```

## Verification Checklist

Before building your dependent crate, verify:

- [ ] `ggml-rs` is in dependencies (NO namespace features needed)
- [ ] Your `build.rs` links to the correct variant (llama or whisper)
- [ ] Your `build.rs` uses `DEP_GGML_RS_GGML_LLAMA_*` or `DEP_GGML_RS_GGML_WHISPER_*` environment variables
- [ ] CMake configuration (if any) uses the namespaced library names
- [ ] No hardcoded references to `ggml`, `ggml-base`, `ggml-cuda`, etc. in your code

## Example: Complete llama-cpp-rs Configuration

```toml
# Cargo.toml
[dependencies]
ggml-rs = { path = "../ggml-rs", features = ["cuda"] }  # NO namespace feature!

[features]
default = []
cuda = ["ggml-rs/cuda"]
use-shared-ggml = []
```

```rust
// build.rs
use std::env;

fn main() {
    #[cfg(feature = "use-shared-ggml")]
    {
        // Get llama variant paths
        let include = env::var("DEP_GGML_RS_INCLUDE").unwrap();
        let lib_dir = env::var("DEP_GGML_RS_GGML_LLAMA_LIB_DIR").unwrap();
        let base_name = env::var("DEP_GGML_RS_GGML_LLAMA_BASENAME").unwrap_or_else(|_| "ggml_llama".to_string());
        
        // Link to llama variant
        println!("cargo:rustc-link-search=native={}", lib_dir);
        println!("cargo:rustc-link-lib=dylib={}", base_name);
        println!("cargo:rustc-link-lib=dylib={}-base", base_name);
        println!("cargo:rustc-link-lib=dylib={}-cpu", base_name);
        
        #[cfg(feature = "cuda")]
        {
            println!("cargo:rustc-link-lib=dylib={}-cuda", base_name);
        }
        
        // Configure CMake
        let mut config = cmake::Config::new(".");
        config.define("GGML_INCLUDE_DIR", include);
        config.define("GGML_LIB_DIR", lib_dir);
        // ... rest of CMake configuration
    }
}
```

## Troubleshooting

### Error: "cannot find -lggml_llama" or "cannot find -lggml_whisper"
- Check that `DEP_GGML_RS_GGML_LLAMA_LIB_DIR` (or `GGML_WHISPER_LIB_DIR`) is set
- Verify that `ggml-rs` built successfully (check build output)
- Make sure you're linking to the correct variant (llama vs whisper)

### Error: "undefined reference to ggml_*"
- Make sure you're linking to all required libraries (base, cpu, cuda if enabled)
- Check that the library names match: `ggml_llama`, `ggml_llama-base`, etc.

### Error: "multiple definition of ggml_*"
- This shouldn't happen with namespacing, but if it does:
  - Make sure `llama-cpp-rs` links to `ggml_llama*` libraries
  - Make sure `whisper-rs` links to `ggml_whisper*` libraries
  - Verify both variants were built (check `ggml-rs` build output)

### Error: "GGML_LLAMA_LIB_DIR not set"
- Make sure `ggml-rs` is in your dependencies
- Rebuild `ggml-rs` first: `cargo build -p ggml-rs`
- The environment variable name is `DEP_GGML_RS_GGML_LLAMA_LIB_DIR` (with `DEP_GGML_RS_` prefix)

## Summary

1. **Add `ggml-rs` dependency** (NO namespace features needed - both variants build automatically)
2. **Link to your variant** - Use `DEP_GGML_RS_GGML_LLAMA_*` or `DEP_GGML_RS_GGML_WHISPER_*` variables
3. **Copy DLLs** - Copy your variant's DLLs to the target directory for runtime
4. **Test coexistence** - Build both crates and verify they work together without conflicts

