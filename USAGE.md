# Using ggml-rs as a Dependency

This guide explains how to use `ggml-rs` as a dependency in your Rust project.

## Adding the Dependency

### Option 1: Local Path (for development)

Add to your `Cargo.toml`:

```toml
[dependencies]
ggml-rs = { path = "../ggml-rs" }
```

### Option 2: Git Repository

```toml
[dependencies]
ggml-rs = { git = "https://github.com/joshatdia/ggml-rs.git", branch = "main" }
```

### Option 3: Published to crates.io (when available)

```toml
[dependencies]
ggml-rs = "0.1.0"
```

## Environment Variables Available

When you add `ggml-rs` as a dependency, Cargo automatically makes these environment variables available to your build script:

- **`DEP_GGML_RS_ROOT`** - Automatically created by Cargo (root directory of the dependency)
- **`DEP_GGML_RS_INCLUDE`** - Path to GGML include directory (exported by build.rs)
- **`DEP_GGML_RS_LIB_DIR`** - Path to GGML library directory (exported by build.rs)

## Using in Your Build Script

In your `build.rs`, you can access these variables:

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    // Get the include directory
    let include_dir = env::var("DEP_GGML_RS_INCLUDE")
        .expect("DEP_GGML_RS_INCLUDE not set. Make sure ggml-rs is in dependencies.");
    
    // Get the library directory
    let lib_dir = env::var("DEP_GGML_RS_LIB_DIR")
        .expect("DEP_GGML_RS_LIB_DIR not set. Make sure ggml-rs is in dependencies.");
    
    // Get the root directory (automatically set by Cargo)
    let root_dir = env::var("DEP_GGML_RS_ROOT")
        .expect("DEP_GGML_RS_ROOT not set. Make sure ggml-rs is in dependencies.");
    
    // Use in CMake configuration
    println!("cargo:rustc-link-search=native={}", lib_dir);
    println!("cargo:rustc-link-lib=dylib=ggml");
}
```

## Verification

To verify that `ggml-rs` is properly configured, run:

```bash
cargo run --bin verify_build
```

This will check:
- ✓ Crate name is correct
- ✓ GGML source directory exists
- ✓ Include directory exists
- ✓ Required headers exist
- ✓ CMakeLists.txt exists
- ✓ wrapper.h exists
- ✓ build.rs exists

## Troubleshooting

### Error: `DEP_GGML_RS_ROOT is not set`

This means `ggml-rs` is not properly added as a dependency. Check:

1. **Is it in Cargo.toml?** Make sure you have:
   ```toml
   [dependencies]
   ggml-rs = { path = "..." }  # or git = "..."
   ```

2. **Is the crate name correct?** The crate name must be exactly `ggml-rs` (with hyphen).

3. **Is the build script running?** Try `cargo clean` and rebuild.

### Error: `DEP_GGML_RS_INCLUDE is not set`

This means the `ggml-rs` build script didn't run successfully. Check:

1. **Does the ggml directory exist?** The `ggml-rs` crate needs a `ggml/` directory with the GGML source code.

2. **Did the build script fail?** Check the build output for errors.

3. **Is it a docs.rs build?** The build script exports variables even on docs.rs, but doesn't build the library.

## Example: Using with llama-cpp-sys-2

In your `llama-cpp-sys-2/Cargo.toml`:

```toml
[dependencies]
ggml-rs = { path = "../ggml-rs" }  # or git = "..."

[features]
use-shared-ggml = []
```

In your `llama-cpp-sys-2/build.rs`:

```rust
fn main() {
    #[cfg(feature = "use-shared-ggml")]
    {
        let root = env::var("DEP_GGML_RS_ROOT")
            .expect("use-shared-ggml feature is enabled but DEP_GGML_RS_ROOT is not set. Make sure ggml-rs is properly configured and the dependency is added to Cargo.toml.");
        
        let include = env::var("DEP_GGML_RS_INCLUDE")
            .expect("DEP_GGML_RS_INCLUDE not set");
        
        let lib_dir = env::var("DEP_GGML_RS_LIB_DIR")
            .expect("DEP_GGML_RS_LIB_DIR not set");
        
        // Configure CMake to use shared GGML
        // ... your CMake configuration ...
    }
}
```

