# Muguet iOS Builds

Precompiled release artifacts and CI orchestration for the [Muguet (iOSVN Rebuild)](https://github.com/iOSVNNews/muguet) project.

---

## Downloads

Latest release binaries (ARM64 Mach-O IPA) are available under [Releases](https://github.com/iOSVNNews/muguet-ios-build/releases):
- **Build 8.5 (Latest)**: [Muguet-v8.5.ipa](https://github.com/iOSVNNews/muguet-ios-build/releases/download/iosvn-runtime8.5-20261012/Muguet-v8.5.ipa) (SHA-256: `284df7f1a9f89332a84710f8301ae93d077d605f40bfd352390be56c19a9577c`)

For source code, technical documentation, and setup instructions, refer to the main repository: [iOSVNNews/muguet](https://github.com/iOSVNNews/muguet).

---

## Build Pipeline Overview

The CI pipeline compiles the source wrapper from `iOSVNNews/muguet`, verifies the WebAssembly module hash, and ahead-of-time (AOT) compiles it into an ARM64 Mach-O binary (`__GAME` segment).

- **Target Architecture**: `aarch64-apple-ios`
- **Compiler**: Rust 1.98.1 + Xcode 26+ (`xcodegen`)
- **Required WASM input hash**: `11ca8d2c04c5e843d18ff4aea4899d72c86973c6b031df334e67c446b2ae83e0`

Game data and shaders (`game.zip` containing `data/manifest.json` and `b/<build>/shaders/index.json`) are imported separately by the user on-device.
