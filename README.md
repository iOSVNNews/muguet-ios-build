# Muguet (iOSVN Rebuild)

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-iOS%20%7C%20iPadOS%20%7C%20macOS-lightgrey.svg)](https://github.com/iOSVNNews/muguet)
[![Architecture](https://img.shields.io/badge/Arch-ARM64%20Mach--O-orange.svg)](https://github.com/iOSVNNews/muguet)
[![Latest Release](https://img.shields.io/badge/Release-Build%208.7-brightgreen.svg)](https://github.com/iOSVNNews/muguet/releases/tag/iosvn-runtime8.7-20261012)

An open-source native Mach-O ARM64 port of Grand Theft Auto V for iOS, iPadOS, and macOS. Powered by WebAssembly ahead-of-time (AOT) compilation and Apple Metal graphics.

> **Project Origin & Attribution**: This project is an open-source rebuild fork of [muguet](https://github.com/c22dev/muguet), originally created and developed by Constantin Clerc ([@c22dev](https://github.com/c22dev)), licensed under the **GNU General Public License v3.0 (GPL-3.0)**.

---

## Core Features & Capabilities

### 1. Native ARM64 Execution (No JIT Required)
- **Ahead-of-Time (AOT) Compiled**: Game bytecode is precompiled into a native Mach-O ARM64 executable segment (`__GAME`).
- **No Debugger / Tethering Needed**: Runs at full native machine speed without requiring JIT debugging entitlements, Xcode tethering, or external developer tools.
- **Compatible with iOS 15.0+**: Supports 64-bit iOS devices running iOS/iPadOS 15.0 or newer.

### 2. Apple Metal Graphics & 120 FPS Activation
- **Direct Metal Pipeline**: Graphics commands route directly through Apple Metal via `wgpu`.
- **Zero Web Overhead**: Bypasses browser sandboxes, WebGPU layers, and WebKit compositor overhead, minimizing draw-call latency and memory consumption.
- **120 FPS ProMotion Support**: Supports 30 FPS, 60 FPS, and unlocks full 120 FPS with `CADisplayLink` frame synchronization.
- **1-Click 120 FPS Performance Preset**: Instantly optimizes resolution scale (50%), textures, ProMotion V-Sync, and Deep Optimization for maximum sustained 120 FPS without thermal throttling.

### 3. Customizable Touchscreen HUD
- **In-Game Layout Editor**: Visual layout customizer accessible from the launcher and pause menu.
- **Free Drag & Repositioning**: Reposition every on-screen action button, virtual stick, or HUD cluster anywhere on screen.
- **Independent Scaling & Opacity**: Resize individual control elements between `0.5×` and `2.2×` scale with granular nudging controls and opacity adjustments.
- **Action Clusters**: Mobile-first controls for shooting, aiming, driving, vehicle entry/exit, horn, cover, handbrake, and weapon wheel.
- **Configurable Aiming**: Toggle or hold aiming modes with customizable touch look sensitivity curves.

### 4. Pause Menu & Map Touch Navigation
- **Direct Touch Map Selection**: Tap directly on pause menu tabs (`MAP`, `BRIEF`, `STATS`, `SETTINGS`, `GAME`).
- **Pan & Pinch-to-Zoom**: Fluid one-finger map dragging and two-finger pinch-to-zoom across Los Santos.
- **One-Tap Waypoints**: Tap anywhere on the world map to set instant GPS navigation waypoints.
- **Optional Navigation Cluster**: Toggleable on-screen directional buttons (TAB ◀/▶, arrows, OK, BACK) for quick menu browsing.

### 5. Multiplayer UDP Lobby & 32/64 Player Server Support
- **Low-Latency Relay Networking**: High-performance UDP networking layer syncing player positions, vehicle states, and animations.
- **32 / 64 Player Capacity**: Configurable room limit supporting standard 32 players or expanded 64-player servers.
- **Custom Room Codes**: Isolated lobbies via 6-character room codes for private or community sessions.
- **Gameplay Modes**: Support for separated Story Mode co-op and Sandbox free-roam sessions with host-configurable PvP rules.

### 6. Save State Persistence
- **Sandbox Storage**: Persistent local storage for standard GTA V save files (`SGTA50000`).
- **File Sharing Access**: Import 100% completion saves or backup campaign saves via the iOS Files app (`On My iPhone > Muguet > saves`) or iTunes / Finder file sharing.

### 7. Memory Safeguards & 64-bit Virtual Addressing
- **Extended Virtual Addressing**: Built with `com.apple.developer.kernel.extended-virtual-addressing` entitlement, providing full 64-bit virtual memory space to prevent `mmap` allocation failures.
- **Increased Memory Limit**: Includes `com.apple.developer.kernel.increased-memory-limit` entitlement to access extended RAM budgets.
- **Adaptive Memory Management**: Automatically adjusts heap reservations to device hardware (4 GB, 6 GB, 8 GB+ RAM).

### 8. Interactive GTA V Cheat Menu
- **Comprehensive Cheat Library**: Invincibility (PAINKILLER), Max Health & Armor (TURTLE), All Weapons (TOOLUP), Clear Wanted Level (LAWYERUP), Super Jump, Moon Gravity, Weather Control, and vehicle spawning (Buzzard attack chopper, Duke O'Death, Comet, PCJ-600, Kraken submarine, etc.).
- **One-Tap In-Game Activation**: Browse and activate cheats easily from the launcher or directly mid-game via the in-game menu (offline story mode only).

---

## System Requirements

| Specification | Minimum | Recommended |
|---|---|---|
| **OS** | iOS / iPadOS 15.0+ | iOS / iPadOS 16.0+ |
| **SoC** | Apple A12 Bionic | Apple A15 Bionic, A16, A17 Pro, M1+ |
| **RAM** | 4 GB | 6 GB to 8 GB+ |
| **Storage** | ~4 GB free | 8 GB+ free |

---

## Download & Installation

Precompiled IPA binaries are available on the [Releases page](https://github.com/iOSVNNews/muguet/releases/tag/iosvn-runtime8.7-20261012):

- **Build 8.7 IPA**: [Muguet-v8.7.ipa](https://github.com/iOSVNNews/muguet/releases/download/iosvn-runtime8.7-20261012/Muguet-v8.7.ipa)
- **SHA-256**: `b5d70b6144145d8a2b676ce6a0f5e48bc868056dfc1aa31c0b9b4b2bbcbecd4f`
- **Release Mirror**: [iOSVNNews/muguet-ios-build](https://github.com/iOSVNNews/muguet-ios-build/releases)

### Sideloading Methods
- **TrollStore (Recommended)**: Permanent installation with unrestricted entitlements and no periodic resign requirements.
- **ESign / Scarlet / Feather**: Sign with your Enterprise or Developer P12 certificate. Ensure `extended-virtual-addressing` and `increased-memory-limit` entitlements are preserved during signing.
- **AltStore / SideStore / Sideloadly**: Standard 7-day profile sideloading supported.
- **Xcode**: Deploy directly with your Apple Developer account.

---

## Game Assets Installation

Muguet is an open-source game engine reimplementation and does **not** contain any proprietary game assets, audio files, or textures.

### Method 1: Direct Folder Copy via Files App (Fastest & Recommended)
1. Extract your `game.zip` asset package on your computer or directly on your device.
2. Open the iOS **Files** app (`Tệp`) and navigate to **On My iPhone / iPad > Muguet** (`muguet`).
3. Create a new folder named **`game`** (all lowercase).
4. Copy and paste the extracted **`data`** and **`b`** folders inside the **`game`** folder (`.../Muguet/game/data` and `.../Muguet/game/b`).
5. Open Muguet — the game will detect the assets instantly and launch without waiting for in-app decompression.

### Method 2: In-App Archive Import
- Launch Muguet and tap **Import game data .zip** to select your `game.zip` archive directly through the iOS document picker.

---

## Building from Source

### Prerequisites (macOS)
- macOS 14+ on Apple Silicon
- Xcode 26+ with iOS SDK and Command Line Tools
- Rust toolchain 1.98.1 with `aarch64-apple-ios` target:
  ```bash
  rustup toolchain install 1.98.1
  rustup target add aarch64-apple-ios --toolchain 1.98.1
  ```
- XcodeGen:
  ```bash
  brew install xcodegen
  ```

### Build Commands
```bash
# Build iOS IPA with game code
./make-ios.sh <path/to/game.zip> --ipa

# Build macOS Application
./make-app.sh
```

---

## Credits & Upstream Attribution

- **Original Creator & Upstream Project**: Constantin Clerc ([@c22dev](https://github.com/c22dev)) - [c22dev/muguet](https://github.com/c22dev/muguet)
- **Rebuild & Enhancements**: iOSVN (Mobile touch HUD editor, pause map touch gestures, UDP multiplayer room system, extended memory entitlements, and CI packaging)
- **Community Contributors**:
  - [Kagander](https://github.com/Kagander): Minimap projection & blip rendering fixes
  - [inj-src](https://github.com/inj-src): Multi-threaded BC block texture decompression
  - [Dbhardwaj99](https://github.com/Dbhardwaj99): Filesystem storage and disk space reporting
  - [dre4moff](https://github.com/dre4moff): Background state and controller input release

---

## License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)**. See the [LICENSE](LICENSE) file for the full license text.
