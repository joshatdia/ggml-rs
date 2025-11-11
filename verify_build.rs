//! Verification script to ensure ggml-rs is properly configured
//! Run with: cargo run --bin verify_build

use std::env;
use std::path::PathBuf;

fn main() {
    println!("Verifying ggml-rs build configuration...\n");

    // Check 1: Verify crate name
    let crate_name = env::var("CARGO_PKG_NAME").unwrap_or_default();
    println!("✓ Crate name: {}", crate_name);
    assert_eq!(crate_name, "ggml-rs", "Crate name must be 'ggml-rs'");

    // Check 2: Verify ggml directory exists
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let ggml_root = PathBuf::from(&manifest_dir).join("ggml");
    println!("✓ GGML root: {}", ggml_root.display());
    assert!(ggml_root.exists(), "GGML directory not found at: {}", ggml_root.display());

    // Check 3: Verify include directory exists
    let include_dir = ggml_root.join("include");
    println!("✓ Include directory: {}", include_dir.display());
    assert!(include_dir.exists(), "Include directory not found");

    // Check 4: Verify key headers exist
    let ggml_h = include_dir.join("ggml.h");
    let gguf_h = include_dir.join("gguf.h");
    println!("✓ Checking headers...");
    assert!(ggml_h.exists(), "ggml.h not found");
    assert!(gguf_h.exists(), "gguf.h not found");

    // Check 5: Verify CMakeLists.txt exists
    let cmake_lists = ggml_root.join("CMakeLists.txt");
    println!("✓ CMakeLists.txt: {}", cmake_lists.display());
    assert!(cmake_lists.exists(), "CMakeLists.txt not found");

    // Check 6: Verify wrapper.h exists
    let wrapper_h = PathBuf::from(&manifest_dir).join("wrapper.h");
    println!("✓ wrapper.h: {}", wrapper_h.display());
    assert!(wrapper_h.exists(), "wrapper.h not found");

    // Check 7: Verify build.rs exists
    let build_rs = PathBuf::from(&manifest_dir).join("build.rs");
    println!("✓ build.rs: {}", build_rs.display());
    assert!(build_rs.exists(), "build.rs not found");

    println!("\n✅ All checks passed! ggml-rs is properly configured.");
    println!("\nTo use this crate as a dependency, add to Cargo.toml:");
    println!("  [dependencies]");
    println!("  ggml-rs = {{ path = \"{}\" }}", manifest_dir);
    println!("\nOr from git:");
    println!("  ggml-rs = {{ git = \"https://github.com/joshatdia/ggml-rs.git\" }}");
    println!("\nDependent crates will have access to:");
    println!("  - DEP_GGML_RS_ROOT (automatically set by Cargo)");
    println!("  - DEP_GGML_RS_INCLUDE (exported by build.rs)");
    println!("  - DEP_GGML_RS_LIB_DIR (exported by build.rs)");
}

