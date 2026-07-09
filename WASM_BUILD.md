# WASM Build Guide

## Overview

TauWriter targets desktop-first (Tauri / Rust + GPUI). Web/WASM is a secondary target. This document describes how to build TauWriter for the browser.

## Prerequisites

```bash
# Install the wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown

# Optionally install wasm-pack (recommended for packaging)
cargo install wasm-pack
```

## Building

### Option A: cargo build (manual)

The LSP crate (`lsp/`) is **not** WASM-compatible by default — it depends on `tokio`, `tower-lsp`, and other OS-specific crates. To produce a WASM artifact you must either:

1. **Feature-gate non-WASM deps**, or
2. **Build the extension only** (pure Rust + `zed_extension_api` with `cdylib` crate-type).

#### Build the Zed extension for WASM

```bash
cd TauWriter/extension
cargo build --target wasm32-unknown-unknown --release
```

Output: `target/wasm32-unknown-unknown/release/tauwriter_extension.wasm`

This binary is suitable for loading in a Zed web extension or any GPUI/GPU-powered renderer.

#### Build the LSP subset that compiles to WASM (optional)

To make `tauwriter-lsp` compile on wasm32, add feature flags or cfg-gate non-WASM deps:

```toml
# In lsp/Cargo.toml, gate OS-specific dependencies:
tokio = { version = "1.0", features = ["full"], optional = true }
tower-lsp = { version = "0.20", optional = true }
```

Then build with only WASM-compatible features:

```bash
cd TauWriter/lsp
cargo build --target wasm32-unknown-unknown --release --no-default-features \
    --features wasm
```

### Option B: wasm-pack (packaged)

```bash
cd TauWriter/extension
wasm-pack build --target web --release
```

Output directory: `pkg/`

## Current Limitations

| Component | WASM-compatible? | Notes |
|-----------|-----------------|-------|
| `tauwriter-extension` (zed_extension_api) | ✅ Yes | Pure Rust, cdylib crate-type |
| `tauwriter-lsp` (tower-lsp, tokio) | ❌ No | OS-specific networking / async runtime |
| Tree-sitter C bindings (cc build-dep) | ⚠️ Manual | Requires wasi SDK or prebuilt `.wasm` WASI modules |

## Automation (Suggested)

Add a CI job similar to `build-lsp` that builds the extension for wasm32 and publishes it:

```yaml
# In .github/workflows/dev-bundle.yml, add:
  build-wasm:
    name: Build WASM Extension
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: wasm32-unknown-unknown
      - name: Build WASM
        run: |
          cd extension
          cargo build --target wasm32-unknown-unknown --release
      - name: Upload WASM artifact
        uses: actions/upload-artifact@v4
        with:
          name: tauwriter-extension-wasm
          path: extension/target/wasm32-unknown-unknown/release/tauwriter_extension.wasm
```

Then commit the `.wasm` file to `extension/bin/` alongside native binaries.

## References

- [TauWriter Design — Target Platform](./TauWriterMD/TauWriterDesign.md): Desktop-first (Tauri / GPUI)
- [Status — Web/Mobile Notes](./Status.md): "Web and mobile are secondary targets."
