# Muguet iOS Builds

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/iOSVNNews/muguet/blob/main/LICENSE)
[![Latest Release](https://img.shields.io/badge/Release-Build%208.7-brightgreen.svg)](https://github.com/iOSVNNews/muguet-ios-build/releases/tag/iosvn-runtime8.7-20261012)

Precompiled release artifacts and CI orchestration for [Muguet (iOSVN Rebuild)](https://github.com/iOSVNNews/muguet), an open-source rebuild fork of [muguet](https://github.com/c22dev/muguet) by Constantin Clerc ([@c22dev](https://github.com/c22dev)) under the GNU General Public License v3.0 (GPL-3.0).

---

## Downloads

Latest release binaries (ARM64 Mach-O IPA) are available under [Releases](https://github.com/iOSVNNews/muguet-ios-build/releases):

- **Build 8.7 (Latest)**: [Muguet-v8.7.ipa](https://github.com/iOSVNNews/muguet-ios-build/releases/download/iosvn-runtime8.7-20261012/Muguet-v8.7.ipa)
- **SHA-256**: `b5d70b6144145d8a2b676ce6a0f5e48bc868056dfc1aa31c0b9b4b2bbcbecd4f`

For full source code, technical documentation, and setup instructions, refer to the main repository: [iOSVNNews/muguet](https://github.com/iOSVNNews/muguet).

---

## Core Build Specifications

The CI pipeline compiles the source wrapper from `iOSVNNews/muguet`, verifies the WebAssembly module hash, and ahead-of-time (AOT) compiles it into an ARM64 Mach-O binary (`__GAME` segment).

- **Target Architecture**: `aarch64-apple-ios` (iOS / iPadOS 15.0+)
- **Compiler**: Rust 1.98.1 + Xcode 26+ (`xcodegen`)
- **Memory Entitlements**: `extended-virtual-addressing` + `increased-memory-limit`
- **Required WASM input hash**: `11ca8d2c04c5e843d18ff4aea4899d72c86973c6b031df334e67c446b2ae83e0`

Game data and shaders (`game.zip` containing `data/manifest.json` and `b/<build>/shaders/index.json`) are imported separately by the user on-device.

---

## License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)**.
