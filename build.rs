#![allow(clippy::uninlined_format_args)]

extern crate bindgen;

use cmake::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    // Verify features are enabled
    println!("[BUILD] Starting ggml-rs build script");
    println!("[BUILD] CUDA feature enabled: {}", cfg!(feature = "cuda"));
    println!("[BUILD] Metal feature enabled: {}", cfg!(feature = "metal"));
    println!("[BUILD] Vulkan feature enabled: {}", cfg!(feature = "vulkan"));
    println!("[BUILD] OpenBLAS feature enabled: {}", cfg!(feature = "openblas"));
    println!("[BUILD] HIPBLAS feature enabled: {}", cfg!(feature = "hipblas"));
    println!("[BUILD] Intel-SYCL feature enabled: {}", cfg!(feature = "intel-sycl"));
    
    // Determine namespace based on features
    let namespace = if cfg!(feature = "namespace-llama") {
        Some("ggml_llama")
    } else if cfg!(feature = "namespace-whisper") {
        Some("ggml_whisper")
    } else {
        None  // Default: no namespace (for backward compatibility)
    };
    
    if let Some(ns) = namespace {
        println!("[BUILD] Using GGML namespace: {}", ns);
    } else {
        println!("[BUILD] No namespace specified - using default GGML symbols");
        println!("[BUILD] WARNING: If using with both llama.cpp and whisper.cpp, enable namespace-llama or namespace-whisper");
    }
    
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
    
    // Set namespace if specified
    if let Some(ns) = namespace {
        config.define("GGML_NAME", ns);
        println!("[BUILD] Setting GGML_NAME={}", ns);
    }

    if cfg!(target_os = "windows") {
        config.cxxflag("/utf-8");
    }

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
    
    // Determine library base name based on namespace (for linking)
    let link_base_name = if cfg!(feature = "namespace-llama") {
        "ggml_llama"
    } else if cfg!(feature = "namespace-whisper") {
        "ggml_whisper"
    } else {
        "ggml"  // Default: no namespace
    };
    
    // Link to shared libraries (not static) - using namespace-aware names
    println!("cargo:rustc-link-lib=dylib={}", link_base_name);
    println!("cargo:rustc-link-lib=dylib={}-base", link_base_name);
    println!("cargo:rustc-link-lib=dylib={}-cpu", link_base_name);
    
    if cfg!(target_os = "macos") || cfg!(feature = "openblas") {
        println!("cargo:rustc-link-lib=dylib={}-blas", link_base_name);
    }
    
    if cfg!(feature = "vulkan") {
        println!("cargo:rustc-link-lib=dylib={}-vulkan", link_base_name);
    }

    if cfg!(feature = "hipblas") {
        println!("cargo:rustc-link-lib=dylib={}-hip", link_base_name);
    }

    if cfg!(feature = "metal") {
        println!("cargo:rustc-link-lib=dylib={}-metal", link_base_name);
    }

    if cfg!(feature = "cuda") {
        // Check if ggml-cuda library exists before linking
        // On Windows, we need the .lib import library file for linking
        let cuda_lib_name = format!("{}-cuda", link_base_name);
        
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
        
        // Debug: Show what we're looking for (always print to stdout)
        println!("[CUDA DEBUG] Looking for CUDA library at: {}", cuda_lib_file.display());
        println!("[CUDA DEBUG] Library directory: {}", lib_dir.display());
        println!("[CUDA DEBUG] Library directory exists: {}", lib_dir.exists());
        println!("[CUDA DEBUG] Build library path: {}", build_lib_file.display());
        println!("[CUDA DEBUG] Build library exists: {}", build_lib_file.exists());
        
        // Also check for .dll file (on Windows)
        if cfg!(target_os = "windows") {
            let cuda_dll_file = lib_dir.join(format!("{}.dll", cuda_lib_name));
            println!("[CUDA DEBUG] Looking for CUDA DLL at: {}", cuda_dll_file.display());
            println!("[CUDA DEBUG] CUDA DLL exists: {}", cuda_dll_file.exists());
        }
        
        // List all files in lib_dir for debugging
        if lib_dir.exists() {
            println!("[CUDA DEBUG] Files in lib_dir:");
            if let Ok(entries) = std::fs::read_dir(&lib_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let metadata = entry.metadata().ok();
                    let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                    println!("[CUDA DEBUG]   - {} ({} bytes)", file_name.to_string_lossy(), size);
                }
            }
        } else {
            println!("[CUDA DEBUG] ERROR: Library directory does not exist!");
        }
        
        // Also check build directory for files
        let build_src_dir = destination.join("build").join("src");
        if build_src_dir.exists() {
            println!("[CUDA DEBUG] Checking build/src directory: {}", build_src_dir.display());
            if let Ok(entries) = std::fs::read_dir(&build_src_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    if file_name.to_string_lossy().contains("cuda") {
                        println!("[CUDA DEBUG]   Found CUDA-related file in build/src: {}", file_name.to_string_lossy());
                    }
                }
            }
        }
        
        // Also check parent directory (in case libraries are in a subdirectory)
        if let Some(parent) = lib_dir.parent() {
            println!("[CUDA DEBUG] Checking parent directory: {}", parent.display());
            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    if file_name.to_string_lossy().contains("cuda") {
                        println!("[CUDA DEBUG]   Found CUDA-related file in parent: {}", file_name.to_string_lossy());
                    }
                }
            }
        }
        
        // Only link if the library exists (check both install and build directories)
        if cuda_lib_file.exists() {
            println!("cargo:rustc-link-lib=dylib={}", cuda_lib_name);
            println!("[CUDA DEBUG] SUCCESS: Linking to {} (found in install directory)", cuda_lib_name);
        } else if build_lib_file.exists() {
            // Library exists in build directory but not installed - add build directory to link search
            println!("cargo:rustc-link-search=native={}", build_lib_file.parent().unwrap().display());
            println!("cargo:rustc-link-lib=dylib={}", cuda_lib_name);
            println!("[CUDA DEBUG] SUCCESS: Linking to {} (found in build directory)", cuda_lib_name);
        } else {
            // If library doesn't exist, warn but don't fail
            // This can happen if CUDA wasn't properly configured during build
            println!("[CUDA DEBUG] ERROR: {} library not found at {} or {}", cuda_lib_name, cuda_lib_file.display(), build_lib_file.display());
            println!("[CUDA DEBUG] Make sure CUDA is properly configured and GGML_CUDA=ON was set during CMake build.");
        }
    }

    if cfg!(feature = "openblas") {
        println!("cargo:rustc-link-lib=dylib={}-blas", link_base_name);
    }

    if cfg!(feature = "intel-sycl") {
        println!("cargo:rustc-link-lib=dylib={}-sycl", link_base_name);
    }

    // Copy DLLs/shared libraries to target directory for runtime
    // On Windows, DLLs must be in the same directory as the executable
    // On Unix, we can use rpath, but copying ensures they're available
    copy_runtime_libraries(&destination, &lib_dir);
}

fn copy_runtime_libraries(destination: &PathBuf, lib_dir: &PathBuf) {
    use std::fs;
    
    println!("[COPY] Starting DLL copy process...");
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
    
    // Determine library base name based on namespace
    let lib_base_name = if cfg!(feature = "namespace-llama") {
        "ggml_llama"
    } else if cfg!(feature = "namespace-whisper") {
        "ggml_whisper"
    } else {
        "ggml"  // Default: no namespace
    };
    
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

