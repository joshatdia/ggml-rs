#![allow(clippy::uninlined_format_args)]

extern crate bindgen;

use cmake::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    // Verify features are enabled
    println!("[BUILD] ========================================");
    println!("[BUILD] Starting ggml-rs build script");
    println!("[BUILD] ========================================");
    
    // TEST: Export a test variable immediately to verify exports work
    println!("cargo:TEST_VAR=test_value");
    println!("[BUILD] TEST: Exported cargo:TEST_VAR (should be DEP_GGML_RS_TEST_VAR)");
    
    // Also export test variables for both variants immediately (before any build)
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    println!("cargo:GGML_LLAMA_LIB_DIR={}", out_dir.join("llama").join("lib").display());
    println!("cargo:GGML_LLAMA_BIN_DIR={}", out_dir.join("llama").join("bin").display());
    println!("cargo:GGML_LLAMA_BASENAME=ggml_llama");
    println!("cargo:GGML_WHISPER_LIB_DIR={}", out_dir.join("whisper").join("lib").display());
    println!("cargo:GGML_WHISPER_BIN_DIR={}", out_dir.join("whisper").join("bin").display());
    println!("cargo:GGML_WHISPER_BASENAME=ggml_whisper");
    println!("[BUILD] TEST: Exported initial variant variables (will be overwritten after build)");
    
    println!("[BUILD] CARGO_MANIFEST_DIR: {:?}", env::var("CARGO_MANIFEST_DIR"));
    println!("[BUILD] OUT_DIR: {:?}", env::var("OUT_DIR"));
    println!("[BUILD] TARGET: {:?}", env::var("TARGET"));
    println!("[BUILD] PROFILE: {:?}", env::var("PROFILE"));
    println!("[BUILD] CUDA feature enabled: {}", cfg!(feature = "cuda"));
    println!("[BUILD] Metal feature enabled: {}", cfg!(feature = "metal"));
    println!("[BUILD] Vulkan feature enabled: {}", cfg!(feature = "vulkan"));
    println!("[BUILD] OpenBLAS feature enabled: {}", cfg!(feature = "openblas"));
    println!("[BUILD] HIPBLAS feature enabled: {}", cfg!(feature = "hipblas"));
    println!("[BUILD] Intel-SYCL feature enabled: {}", cfg!(feature = "intel-sycl"));
    
    println!("[BUILD] Building BOTH variants (llama and whisper) unconditionally");
    println!("[BUILD] This ensures both sets of libraries are available regardless of which dependent crate builds first");
    
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
        .clang_arg(format!("-I{}", manifest_path.display()))
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

    // Export variables even on docs.rs so dependent crates can find them
    // (We still need to export INCLUDE even if we don't build the library)
    // Exporting INCLUDE creates DEP_GGML_RS_INCLUDE for dependent crates
    let ggml_include = ggml_root.join("include");
    println!("cargo:INCLUDE={}", ggml_include.display());
    
    // Stop if we're on docs.rs (don't build the library, but export placeholder variables)
    if env::var("DOCS_RS").is_ok() {
        println!("[BUILD] Running on docs.rs - exporting placeholder variables");
        // Export placeholder variables so dependent crates don't fail
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        println!("cargo:GGML_LLAMA_LIB_DIR={}", out_dir.join("llama").join("lib").display());
        println!("cargo:GGML_LLAMA_BIN_DIR={}", out_dir.join("llama").join("bin").display());
        println!("cargo:GGML_LLAMA_BASENAME=ggml_llama");
        println!("cargo:GGML_WHISPER_LIB_DIR={}", out_dir.join("whisper").join("lib").display());
        println!("cargo:GGML_WHISPER_BIN_DIR={}", out_dir.join("whisper").join("bin").display());
        println!("cargo:GGML_WHISPER_BASENAME=ggml_whisper");
        return;
    }

    // Export common include directory (same for both variants) - ALWAYS export this
    println!("cargo:INCLUDE={}", ggml_root.join("include").display());
    println!("[BUILD] Exported cargo:INCLUDE (becomes DEP_GGML_RS_INCLUDE)");
    
    // Build BOTH variants unconditionally (llama and whisper)
    // This ensures both sets of libraries are available regardless of which dependent crate builds first
    println!("[BUILD] Building both GGML variants (llama and whisper)...");
    
    // Pre-allocate paths based on OUT_DIR so we can export them even if build fails
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let llama_lib_dir_fallback = out_dir.join("llama").join("lib");
    let llama_bin_dir_fallback = out_dir.join("llama").join("bin");
    let whisper_lib_dir_fallback = out_dir.join("whisper").join("lib");
    let whisper_bin_dir_fallback = out_dir.join("whisper").join("bin");
    
    let llama_result = build_ggml_variant(&ggml_root, "ggml_llama", "llama");
    let whisper_result = build_ggml_variant(&ggml_root, "ggml_whisper", "whisper");
    
    // Export environment variables for both variants so consumers can find them
    // Consumers will link to their own variant using these variables
    // Note: Cargo automatically prefixes these with DEP_GGML_RS_, so:
    // cargo:GGML_LLAMA_LIB_DIR becomes DEP_GGML_RS_GGML_LLAMA_LIB_DIR
    let (llama_lib_dir, llama_bin_dir) = match llama_result {
        Ok((lib_dir, bin_dir)) => {
            println!("[BUILD] ✓ Llama variant built successfully");
            (lib_dir, bin_dir)
        }
        Err(e) => {
            eprintln!("cargo:warning=Failed to build llama variant: {}", e);
            eprintln!("cargo:warning=Using fallback paths for llama variant");
            (llama_lib_dir_fallback, llama_bin_dir_fallback)
        }
    };
    
    let (whisper_lib_dir, whisper_bin_dir) = match whisper_result {
        Ok((lib_dir, bin_dir)) => {
            println!("[BUILD] ✓ Whisper variant built successfully");
            (lib_dir, bin_dir)
        }
        Err(e) => {
            eprintln!("cargo:warning=Failed to build whisper variant: {}", e);
            eprintln!("cargo:warning=Using fallback paths for whisper variant");
            (whisper_lib_dir_fallback, whisper_bin_dir_fallback)
        }
    };
    
    // ALWAYS export variables, even if build failed (use fallback paths)
    // Use explicit format! to ensure proper formatting
    use std::io::{self, Write};
    
    println!("[BUILD] Exporting llama variant variables:");
    println!("[BUILD]   GGML_LLAMA_LIB_DIR={}", llama_lib_dir.display());
    println!("[BUILD]   GGML_LLAMA_BIN_DIR={}", llama_bin_dir.display());
    
    // Export using cargo: prefix - Cargo will make these available as DEP_GGML_RS_*
    io::stdout().flush().ok();
    println!("cargo:GGML_LLAMA_LIB_DIR={}", llama_lib_dir.display());
    io::stdout().flush().ok();
    println!("cargo:GGML_LLAMA_BIN_DIR={}", llama_bin_dir.display());
    io::stdout().flush().ok();
    println!("cargo:GGML_LLAMA_BASENAME=ggml_llama");
    io::stdout().flush().ok();
    
    println!("[BUILD] Exporting whisper variant variables:");
    println!("[BUILD]   GGML_WHISPER_LIB_DIR={}", whisper_lib_dir.display());
    println!("[BUILD]   GGML_WHISPER_BIN_DIR={}", whisper_bin_dir.display());
    
    io::stdout().flush().ok();
    println!("cargo:GGML_WHISPER_LIB_DIR={}", whisper_lib_dir.display());
    io::stdout().flush().ok();
    println!("cargo:GGML_WHISPER_BIN_DIR={}", whisper_bin_dir.display());
    io::stdout().flush().ok();
    println!("cargo:GGML_WHISPER_BASENAME=ggml_whisper");
    io::stdout().flush().ok();
    
    println!("[BUILD] Build script completed. Variables exported:");
    println!("[BUILD]   DEP_GGML_RS_GGML_LLAMA_LIB_DIR={}", llama_lib_dir.display());
    println!("[BUILD]   DEP_GGML_RS_GGML_LLAMA_BIN_DIR={}", llama_bin_dir.display());
    println!("[BUILD]   DEP_GGML_RS_GGML_LLAMA_BASENAME=ggml_llama");
    println!("[BUILD]   DEP_GGML_RS_GGML_WHISPER_LIB_DIR={}", whisper_lib_dir.display());
    println!("[BUILD]   DEP_GGML_RS_GGML_WHISPER_BIN_DIR={}", whisper_bin_dir.display());
    println!("[BUILD]   DEP_GGML_RS_GGML_WHISPER_BASENAME=ggml_whisper");
    println!("[BUILD]   DEP_GGML_RS_INCLUDE={}", ggml_root.join("include").display());
    
    // IMPORTANT: Do NOT emit cargo:rustc-link-lib here
    // Each consumer crate (llama-cpp-rs, whisper-rs) will link to its own variant
}

/// Build a single GGML variant with the specified namespace
fn build_ggml_variant(ggml_root: &PathBuf, namespace: &str, tag: &str) -> Result<(PathBuf, PathBuf), Box<dyn std::error::Error>> {
    println!("[BUILD] Building {} variant with namespace: {}", tag, namespace);
    
    // Build ggml as shared library using CMake
    let mut config = Config::new(&ggml_root);

    // Use a separate install prefix for each variant to avoid conflicts
    // The cmake crate will manage build directories automatically
    let out_dir = env::var("OUT_DIR").unwrap();
    let variant_install_prefix = PathBuf::from(&out_dir).join(tag);
    
    config
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "ON")  // Build as shared library
        .define("GGML_ALL_WARNINGS", "OFF")
        .define("GGML_ALL_WARNINGS_3RD_PARTY", "OFF")
        .define("GGML_BUILD_TESTS", "OFF")  // Disable tests (directory doesn't exist)
        .define("GGML_BUILD_EXAMPLES", "OFF")  // Disable examples (directory doesn't exist)
        // Note: GGML_STANDALONE will be set to ON by CMakeLists.txt when building standalone
        // We've created ggml.pc.in to satisfy the configure_file requirement
        .define("CMAKE_INSTALL_PREFIX", variant_install_prefix.to_string_lossy().as_ref())  // Separate install directory
        .very_verbose(true)
        .pic(true);
    
    // Always set namespace for this variant
    config.define("GGML_NAME", namespace);
    println!("[BUILD] Setting GGML_NAME={} for {} variant", namespace, tag);
    println!("[BUILD] Using install prefix: {}", variant_install_prefix.display());

    if cfg!(target_os = "windows") {
        config.cxxflag("/utf-8");
    }
    
    let target = env::var("TARGET").unwrap();

    if cfg!(feature = "cuda") {
        println!("[BUILD] Configuring CUDA support");
        config.define("GGML_CUDA", "ON");
        config.define("CMAKE_POSITION_INDEPENDENT_CODE", "ON");
        config.define("CMAKE_CUDA_FLAGS", "-Xcompiler=-fPIC");
        println!("[BUILD] CUDA CMake flags set: GGML_CUDA=ON");
    } else {
        println!("[BUILD] CUDA feature NOT enabled - skipping CUDA build");
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

    println!("[BUILD] Starting CMake build...");
    let destination = config.build();
    println!("[BUILD] CMake build completed. Output directory: {}", destination.display());

    // Explicitly run CMake install to ensure libraries are installed
    // The build() function should run install automatically, but we'll verify
    use std::process::Command;
    let cmake_build_dir = destination.join("build");
    if cmake_build_dir.exists() {
        println!("[BUILD] Running CMake install step...");
        let install_output = Command::new("cmake")
            .arg("--build")
            .arg(&cmake_build_dir)
            .arg("--target")
            .arg("install")
            .arg("--config")
            .arg("Release")
            .output();
        
        match install_output {
            Ok(output) => {
                if output.status.success() {
                    println!("[BUILD] CMake install step completed successfully");
                } else {
                    eprintln!("cargo:warning=CMake install step failed with exit code: {:?}", output.status.code());
                    if !output.stdout.is_empty() {
                        eprintln!("cargo:warning=CMake install stdout: {}", String::from_utf8_lossy(&output.stdout));
                    }
                    if !output.stderr.is_empty() {
                        eprintln!("cargo:warning=CMake install stderr: {}", String::from_utf8_lossy(&output.stderr));
                    }
                }
            }
            Err(e) => {
                eprintln!("cargo:warning=Failed to run CMake install: {}", e);
            }
        }
    } else {
        println!("[BUILD] CMake build directory does not exist: {}", cmake_build_dir.display());
    }

    // Get library and binary directories from the install prefix
    // Since we set CMAKE_INSTALL_PREFIX, the libraries should be in the install directory
    let install_prefix = PathBuf::from(env::var("OUT_DIR").unwrap()).join(tag);
    let lib_dir_install = install_prefix.join("lib");
    let bin_dir_install = install_prefix.join("bin");
    
    // Also check the destination directory (cmake crate's default location)
    // Libraries might be in destination/lib or install_prefix/lib
    let lib_dir_dest = destination.join("lib");
    let bin_dir_dest = destination.join("bin");
    
    // Use whichever exists (prefer install prefix if both exist)
    let lib_dir = if lib_dir_install.exists() {
        lib_dir_install
    } else if lib_dir_dest.exists() {
        lib_dir_dest
    } else {
        lib_dir_install  // Return install prefix path even if it doesn't exist yet
    };
    
    let bin_dir = if bin_dir_install.exists() {
        bin_dir_install
    } else if bin_dir_dest.exists() {
        bin_dir_dest
    } else {
        bin_dir_install  // Return install prefix path even if it doesn't exist yet
    };
    
    // Verify libraries were built
    println!("[BUILD] {} variant build completed", tag);
    println!("[BUILD] Library directory: {}", lib_dir.display());
    println!("[BUILD] Binary directory: {}", bin_dir.display());
    
    if lib_dir.exists() {
        println!("[BUILD] Available libraries in {}:", lib_dir.display());
        if let Ok(entries) = std::fs::read_dir(&lib_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                println!("[BUILD]   - {}", file_name.to_string_lossy());
            }
        }
    }
    
    // Patch ggml-config.cmake to use namespaced library names
    patch_ggml_config_cmake(&destination, namespace);
    
    // Copy DLLs/shared libraries to variant-specific location
    // Consumers will copy from here to their target directory
    copy_runtime_libraries(&destination, &lib_dir, namespace);
    
    Ok((lib_dir, bin_dir))
}

/// Patch ggml-config.cmake to use namespaced library names
fn patch_ggml_config_cmake(destination: &PathBuf, namespace: &str) {
    use std::fs;
    use std::io::Write;
    
    // ggml-config.cmake can be in multiple locations:
    // 1. build/ggml-config.cmake (before install)
    // 2. lib/cmake/ggml/ggml-config.cmake (after install)
    let possible_paths = vec![
        destination.join("build").join("ggml-config.cmake"),
        destination.join("lib").join("cmake").join("ggml").join("ggml-config.cmake"),
    ];
    
    for config_path in possible_paths {
        if !config_path.exists() {
            continue;
        }
        
        println!("[PATCH] Found ggml-config.cmake at: {}", config_path.display());
        
        // Read the file
        let content = match fs::read_to_string(&config_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("cargo:warning=Failed to read ggml-config.cmake: {}", e);
                continue;
            }
        };
        
        // Replace library names with namespaced versions
        let mut patched = content.clone();
        
        // Replace main library: find_library(GGML_LIBRARY ggml -> find_library(GGML_LIBRARY ggml_llama
        patched = patched.replace(
            &format!("find_library(GGML_LIBRARY ggml\n"),
            &format!("find_library(GGML_LIBRARY {}\n", namespace)
        );
        patched = patched.replace(
            &format!("find_library(GGML_LIBRARY ggml "),
            &format!("find_library(GGML_LIBRARY {} ", namespace)
        );
        
        // Replace base library: find_library(GGML_BASE_LIBRARY ggml-base -> find_library(GGML_BASE_LIBRARY ggml_llama-base
        patched = patched.replace(
            &format!("find_library(GGML_BASE_LIBRARY ggml-base\n"),
            &format!("find_library(GGML_BASE_LIBRARY {}-base\n", namespace)
        );
        patched = patched.replace(
            &format!("find_library(GGML_BASE_LIBRARY ggml-base "),
            &format!("find_library(GGML_BASE_LIBRARY {}-base ", namespace)
        );
        
        // Replace backend libraries - use a simpler approach: replace "ggml-" with "{namespace}-"
        // but preserve "ggml::" (target namespace) and other contexts
        // This handles all backend libraries generically
        
        // First, protect "ggml::" by temporarily replacing it
        let protected_marker = "___GGML_TARGET_NAMESPACE___";
        patched = patched.replace("ggml::", protected_marker);
        
        // Now replace all "ggml-" with "{namespace}-"
        patched = patched.replace("ggml-", &format!("{}-", namespace));
        
        // Also replace standalone "ggml" (the main library) but be careful
        // Only replace "ggml" when it's a library name, not in other contexts
        // Pattern: find_library(GGML_LIBRARY ggml -> find_library(GGML_LIBRARY {namespace}
        // We already handled this above, but let's also handle any remaining cases
        patched = patched.replace(
            &format!(" ggml\n"),
            &format!(" {}\n", namespace)
        );
        patched = patched.replace(
            &format!(" ggml "),
            &format!(" {} ", namespace)
        );
        patched = patched.replace(
            &format!(" ggml)"),
            &format!(" {})", namespace)
        );
        
        // Restore "ggml::"
        patched = patched.replace(protected_marker, "ggml::");
        
        // Note: Target names (ggml::ggml-base) stay unchanged for compatibility.
        // The IMPORTED_LOCATION will point to the namespaced library file because
        // we've patched the find_library calls above.
        
        // Check if anything changed
        if patched != content {
            // Write the patched content back
            match fs::File::create(&config_path).and_then(|mut f| f.write_all(patched.as_bytes())) {
                Ok(_) => {
                    println!("[PATCH] ✓ Successfully patched ggml-config.cmake with namespace: {}", namespace);
                }
                Err(e) => {
                    eprintln!("cargo:warning=Failed to write patched ggml-config.cmake: {}", e);
                }
            }
        } else {
            println!("[PATCH] No changes needed in ggml-config.cmake");
        }
        
        // Only patch the first file found
        break;
    }
}

fn copy_runtime_libraries(destination: &PathBuf, lib_dir: &PathBuf, namespace: &str) {
    use std::fs;
    
    println!("[COPY] Starting DLL copy process for {} variant...", namespace);
    println!("[COPY] Destination: {}", destination.display());
    println!("[COPY] Library directory: {}", lib_dir.display());
    
    // Get the target directory (where the executable will be)
    // OUT_DIR is like: target/debug/build/ggml-rs-xxx/out
    // We need: target/debug/ or target/release/
    // Structure: target/<profile>/build/<crate>-<hash>/out
    // Go up 4 levels: out -> <crate>-<hash> -> build -> <profile> -> target
    // Then join <profile> to get target/<profile>/
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let target_dir = out_dir
        .parent().unwrap()  // <crate>-<hash>/
        .parent().unwrap()  // build/
        .parent().unwrap()  // <profile>/
        .parent().unwrap()  // target/
        .join(&profile);    // target/<profile>/
    
    println!("[COPY] Target directory: {}", target_dir.display());
    
    // Create target directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&target_dir) {
        eprintln!("cargo:warning=Failed to create target directory {}: {}", target_dir.display(), e);
        return;
    }
    
    // Determine library extension based on platform
    let lib_ext = if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    
    // Use the namespace passed in
    let lib_base_name = namespace;
    
    // List of libraries to copy (using namespace-aware names)
    let mut libraries = vec![
        lib_base_name.to_string(),
        format!("{}-base", lib_base_name),
        format!("{}-cpu", lib_base_name),
    ];
    
    // Add optional libraries based on features (backend libraries use namespace prefix)
    if cfg!(feature = "cuda") {
        libraries.push(format!("{}-cuda", lib_base_name));
    }
    if cfg!(feature = "vulkan") {
        libraries.push(format!("{}-vulkan", lib_base_name));
    }
    if cfg!(feature = "hipblas") {
        libraries.push(format!("{}-hip", lib_base_name));
    }
    if cfg!(feature = "metal") {
        libraries.push(format!("{}-metal", lib_base_name));
    }
    if cfg!(feature = "openblas") || cfg!(target_os = "macos") {
        libraries.push(format!("{}-blas", lib_base_name));
    }
    if cfg!(feature = "intel-sycl") {
        libraries.push(format!("{}-sycl", lib_base_name));
    }
    
    // Copy libraries from install directory
    println!("[COPY] Libraries to copy: {:?}", libraries);
    for lib_name in libraries.iter() {
        println!("[COPY] Checking for library: {}", lib_name);
        let lib_file = if cfg!(target_os = "windows") {
            lib_dir.join(format!("{}.{}", lib_name, lib_ext))
        } else if cfg!(target_os = "macos") {
            lib_dir.join(format!("lib{}.{}", lib_name, lib_ext))
        } else {
            lib_dir.join(format!("lib{}.{}", lib_name, lib_ext))
        };
        
        println!("[COPY]   Checking install directory: {}", lib_file.display());
        if lib_file.exists() {
            let target_file = target_dir.join(lib_file.file_name().unwrap());
            if let Err(e) = fs::copy(&lib_file, &target_file) {
                eprintln!("cargo:warning=Failed to copy {} to {}: {}", lib_file.display(), target_file.display(), e);
            } else {
                println!("[COPY] ✓ Copied {} to {}", lib_file.display(), target_file.display());
            }
        } else {
            println!("[COPY]   Not found in install directory, checking build directory...");
            // Also check build directory (library might be built but not installed)
            let build_lib_file = if cfg!(target_os = "windows") {
                destination.join("build").join("src").join("Release").join(format!("{}.{}", lib_name, lib_ext))
            } else if cfg!(target_os = "macos") {
                destination.join("build").join("src").join(format!("lib{}.{}", lib_name, lib_ext))
            } else {
                destination.join("build").join("src").join(format!("lib{}.{}", lib_name, lib_ext))
            };
            
            println!("[COPY]   Checking build directory: {}", build_lib_file.display());
            if build_lib_file.exists() {
                let target_file = target_dir.join(build_lib_file.file_name().unwrap());
                if let Err(e) = fs::copy(&build_lib_file, &target_file) {
                    eprintln!("cargo:warning=Failed to copy {} to {}: {}", build_lib_file.display(), target_file.display(), e);
                } else {
                    println!("[COPY] ✓ Copied {} to {}", build_lib_file.display(), target_file.display());
                }
            } else {
                println!("[COPY] ✗ Library {} not found in build directory either", lib_name);
            }
        }
    }
    
    // Also check bin directory on Windows (DLLs might be installed there)
    if cfg!(target_os = "windows") {
        let bin_dir = destination.join("bin");
        println!("[COPY] Checking bin directory: {}", bin_dir.display());
        if bin_dir.exists() {
            println!("[COPY] Bin directory exists, checking for DLLs...");
            if let Ok(entries) = fs::read_dir(&bin_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    println!("[COPY]   Found in bin: {}", file_name.to_string_lossy());
                }
            }
            for lib_name in libraries.iter() {
                let dll_file = bin_dir.join(format!("{}.dll", lib_name));
                println!("[COPY]   Checking bin for: {}", dll_file.display());
                if dll_file.exists() {
                    let target_file = target_dir.join(dll_file.file_name().unwrap());
                    if let Err(e) = fs::copy(&dll_file, &target_file) {
                        eprintln!("cargo:warning=Failed to copy {} to {}: {}", dll_file.display(), target_file.display(), e);
                    } else {
                        println!("[COPY] ✓ Copied {} to {}", dll_file.display(), target_file.display());
                    }
                } else {
                    println!("[COPY]   Not found: {}", dll_file.display());
                }
            }
        } else {
            println!("[COPY] Bin directory does not exist: {}", bin_dir.display());
        }
        
        // Also check build/bin directory (DLLs might be in build output)
        let build_bin_dir = destination.join("build").join("bin");
        println!("[COPY] Checking build/bin directory: {}", build_bin_dir.display());
        if build_bin_dir.exists() {
            println!("[COPY] Build/bin directory exists, checking for DLLs...");
            if let Ok(entries) = fs::read_dir(&build_bin_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    println!("[COPY]   Found in build/bin: {}", file_name.to_string_lossy());
                }
            }
            for lib_name in libraries.iter() {
                let dll_file = build_bin_dir.join(format!("{}.dll", lib_name));
                println!("[COPY]   Checking build/bin for: {}", dll_file.display());
                if dll_file.exists() {
                    let target_file = target_dir.join(dll_file.file_name().unwrap());
                    if let Err(e) = fs::copy(&dll_file, &target_file) {
                        eprintln!("cargo:warning=Failed to copy {} to {}: {}", dll_file.display(), target_file.display(), e);
                    } else {
                        println!("[COPY] ✓ Copied {} to {}", dll_file.display(), target_file.display());
                    }
                }
            }
        }
        
        // Also check build/bin/Release directory (Windows Release build output)
        if cfg!(target_os = "windows") {
            let build_bin_release_dir = destination.join("build").join("bin").join("Release");
            println!("[COPY] Checking build/bin/Release directory: {}", build_bin_release_dir.display());
            if build_bin_release_dir.exists() {
                println!("[COPY] Build/bin/Release directory exists, checking for DLLs...");
                if let Ok(entries) = fs::read_dir(&build_bin_release_dir) {
                    for entry in entries.flatten() {
                        let file_name = entry.file_name();
                        println!("[COPY]   Found in build/bin/Release: {}", file_name.to_string_lossy());
                    }
                }
                for lib_name in libraries.iter() {
                    let dll_file = build_bin_release_dir.join(format!("{}.dll", lib_name));
                    println!("[COPY]   Checking build/bin/Release for: {}", dll_file.display());
                    if dll_file.exists() {
                        let target_file = target_dir.join(dll_file.file_name().unwrap());
                        if let Err(e) = fs::copy(&dll_file, &target_file) {
                            eprintln!("cargo:warning=Failed to copy {} to {}: {}", dll_file.display(), target_file.display(), e);
                        } else {
                            println!("[COPY] ✓ Copied {} to {}", dll_file.display(), target_file.display());
                        }
                    }
                }
            }
        }
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

