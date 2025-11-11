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
    
    // Stop if we're on docs.rs (don't build the library, but variables are already exported)
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    // Build ggml as shared library using CMake
    let mut config = Config::new(&ggml_root);

    config
        .profile("Release")
        .define("BUILD_SHARED_LIBS", "ON")  // Build as shared library
        .define("GGML_ALL_WARNINGS", "OFF")
        .define("GGML_ALL_WARNINGS_3RD_PARTY", "OFF")
        .define("GGML_BUILD_TESTS", "OFF")  // Disable tests (directory doesn't exist)
        .define("GGML_BUILD_EXAMPLES", "OFF")  // Disable examples (directory doesn't exist)
        // Note: GGML_STANDALONE will be set to ON by CMakeLists.txt when building standalone
        // We've created ggml.pc.in to satisfy the configure_file requirement
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

    // Explicitly run CMake install to ensure libraries are installed
    // The build() function should run install automatically, but we'll verify
    use std::process::Command;
    let cmake_build_dir = destination.join("build");
    if cmake_build_dir.exists() {
        let install_status = Command::new("cmake")
            .arg("--build")
            .arg(&cmake_build_dir)
            .arg("--target")
            .arg("install")
            .arg("--config")
            .arg("Release")
            .status();
        
        if let Ok(status) = install_status {
            if !status.success() {
                eprintln!("cargo:warning=CMake install step failed, but continuing...");
            }
        }
    }

    // Export the library path for CMake to find
    let lib_dir = destination.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    
    // Export library path as environment variable for CMake find_package
    // Cargo automatically creates DEP_GGML_RS_ROOT for crate "ggml-rs"
    // Exporting INCLUDE creates DEP_GGML_RS_INCLUDE (without double prefix)
    // Exporting GGML_RS_INCLUDE would create DEP_GGML_RS_GGML_RS_INCLUDE (redundant)
    // We export INCLUDE to match what dependent crates expect: DEP_GGML_RS_INCLUDE
    println!("cargo:LIB_DIR={}", lib_dir.display());
    println!("cargo:INCLUDE={}", ggml_root.join("include").display());
    
    // Link to shared libraries (not static)
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
        // Check if ggml-cuda library exists before linking
        // On Windows, we need the .lib import library file for linking
        let cuda_lib_name = "ggml-cuda";
        
        // Check if the library file exists in install directory
        // On Windows, we need the .lib file (import library) for linking
        // On Unix, we need the .so/.dylib file
        let cuda_lib_file = if cfg!(target_os = "windows") {
            lib_dir.join(format!("{}.lib", cuda_lib_name))
        } else if cfg!(target_os = "macos") {
            lib_dir.join(format!("lib{}.dylib", cuda_lib_name))
        } else {
            lib_dir.join(format!("lib{}.so", cuda_lib_name))
        };
        
        // Also check build directory (library might be built but not installed)
        let build_lib_file = if cfg!(target_os = "windows") {
            destination.join("build").join("src").join("Release").join(format!("{}.lib", cuda_lib_name))
        } else if cfg!(target_os = "macos") {
            destination.join("build").join("src").join(format!("lib{}.dylib", cuda_lib_name))
        } else {
            destination.join("build").join("src").join(format!("lib{}.so", cuda_lib_name))
        };
        
        // Debug: Show what we're looking for
        eprintln!("cargo:warning=[CUDA] Looking for CUDA library at: {}", cuda_lib_file.display());
        eprintln!("cargo:warning=[CUDA] Library directory: {}", lib_dir.display());
        eprintln!("cargo:warning=[CUDA] Library directory exists: {}", lib_dir.exists());
        
        // Also check for .dll file (on Windows)
        if cfg!(target_os = "windows") {
            let cuda_dll_file = lib_dir.join(format!("{}.dll", cuda_lib_name));
            eprintln!("cargo:warning=[CUDA] Looking for CUDA DLL at: {}", cuda_dll_file.display());
            eprintln!("cargo:warning=[CUDA] CUDA DLL exists: {}", cuda_dll_file.exists());
        }
        
        // List all files in lib_dir for debugging
        if lib_dir.exists() {
            eprintln!("cargo:warning=[CUDA] Files in lib_dir:");
            if let Ok(entries) = std::fs::read_dir(&lib_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let metadata = entry.metadata().ok();
                    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    eprintln!("cargo:warning=[CUDA]   - {} ({} bytes)", file_name.to_string_lossy(), size);
                }
            }
        } else {
            eprintln!("cargo:warning=[CUDA] ERROR: Library directory does not exist!");
        }
        
        // Also check parent directory (in case libraries are in a subdirectory)
        if let Some(parent) = lib_dir.parent() {
            eprintln!("cargo:warning=[CUDA] Checking parent directory: {}", parent.display());
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    if file_name.to_string_lossy().contains("cuda") {
                        eprintln!("cargo:warning=[CUDA]   Found CUDA-related file in parent: {}", file_name.to_string_lossy());
                    }
                }
            }
        }
        
        // Only link if the library exists (check both install and build directories)
        if cuda_lib_file.exists() {
            println!("cargo:rustc-link-lib=dylib={}", cuda_lib_name);
            eprintln!("cargo:warning=[CUDA] Successfully linking to ggml-cuda (found in install directory)");
        } else if build_lib_file.exists() {
            // Library exists in build directory but not installed - add build directory to link search
            println!("cargo:rustc-link-search=native={}", build_lib_file.parent().unwrap().display());
            println!("cargo:rustc-link-lib=dylib={}", cuda_lib_name);
            eprintln!("cargo:warning=[CUDA] Successfully linking to ggml-cuda (found in build directory)");
        } else {
            // If library doesn't exist, warn but don't fail
            // This can happen if CUDA wasn't properly configured during build
            eprintln!("cargo:warning=[CUDA] ERROR: ggml-cuda library not found at {} or {}, skipping link. Make sure CUDA is properly configured and GGML_CUDA=ON was set during CMake build.", cuda_lib_file.display(), build_lib_file.display());
        }
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

