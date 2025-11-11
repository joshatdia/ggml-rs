# Fix for regex-automata rlib Format Error

## Problem

When using `ggml-rs` as a dependency in other crates (like `llama-cpp-rs`), you may encounter this error:

```
error: crate `regex_automata` required to be available in rlib format, but was not found in this form
```

## Root Cause

The `regex-automata` crate is a transitive dependency through:
- `bindgen` → `regex` → `regex-automata`

When building as a dependency, Cargo needs `regex-automata` in rlib format, but the dependency resolution might not provide it in that format.

## Solution

Explicitly add `regex-automata` to `[build-dependencies]` in `Cargo.toml`:

```toml
[build-dependencies]
cmake = "0.1"
bindgen = "0.71"
cc = { version = "1.0", features = ["parallel"] }
regex-automata = "0.4"  # Explicitly added to ensure rlib format
```

## Verification

1. **Local build**: `cargo build` should succeed
2. **As dependency**: When used in `llama-cpp-rs`, the build should succeed without the rlib format error

## Why This Works

By explicitly adding `regex-automata` to `[build-dependencies]`, we ensure that:
- Cargo knows to build it in rlib format for build scripts
- The dependency resolution is explicit and unambiguous
- Other crates using `ggml-rs` can find `regex-automata` in the correct format

## Testing

To test that this fix works:

1. Build `ggml-rs` locally:
   ```bash
   cd ggml-rs
   cargo build
   ```

2. Use it as a dependency in another crate:
   ```toml
   [dependencies]
   ggml-rs = { path = "../ggml-rs" }
   ```

3. Build the dependent crate:
   ```bash
   cargo build
   ```

The build should succeed without the `regex_automata` rlib format error.

