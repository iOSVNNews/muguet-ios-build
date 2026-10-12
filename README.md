# Muguet iOS Builds

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://github.com/iOSVNNews/muguet/blob/main/LICENSE)
[![Latest Release](https://img.shields.io/badge/Release-Build%208.7-brightgreen.svg)](https://github.com/iOSVNNews/muguet-ios-build/releases/tag/iosvn-runtime8.7-20261012)

Precompiled release artifacts and distribution repository for [Muguet (iOSVN Rebuild)](https://github.com/iOSVNNews/muguet), an open-source rebuild fork of [muguet](https://github.com/c22dev/muguet) by Constantin Clerc ([@c22dev](https://github.com/c22dev)) under the **GNU General Public License v3.0 (GPL-3.0)**.

---

## Downloads

Latest release binaries (ARM64 Mach-O IPA) are available under [Releases](https://github.com/iOSVNNews/muguet-ios-build/releases):

- **Build 8.7 (Latest)**: [Muguet-v8.7.ipa](https://github.com/iOSVNNews/muguet-ios-build/releases/download/iosvn-runtime8.7-20261012/Muguet-v8.7.ipa)
- **SHA-256**: `b5d70b6144145d8a2b676ce6a0f5e48bc868056dfc1aa31c0b9b4b2bbcbecd4f`

For full source code and technical documentation, visit the main repository: [iOSVNNews/muguet](https://github.com/iOSVNNews/muguet).

---

## Key Features

- **Native ARM64 Execution (No JIT Required)**: Game bytecode is precompiled ahead-of-time (AOT) into an executable Mach-O segment (`__GAME`). Runs at full speed on iOS 15.0+ without JIT entitlements, debuggers, or tethering.
- **Apple Metal Graphics**: Direct hardware rendering through Apple Metal via `wgpu`, removing browser/WebKit overhead and supporting stable 60 FPS and 120 FPS on ProMotion devices.
- **Customizable Touchscreen HUD**: Built-in HUD layout editor allowing free dragging, repositioning, and individual scaling of action buttons and virtual sticks (`0.5×` to `2.2×`).
- **Interactive Pause Map**: Direct touch input for menu tabs, fluid map dragging, two-finger pinch-to-zoom, and instant GPS waypoint placement on the Los Santos map.
- **Low-Latency UDP Multiplayer**: Dedicated UDP networking layer with 6-character room codes for private lobbies and separated Story Mode or Sandbox PvP sessions.
- **Save State Persistence**: Native sandbox storage for GTA V save files (`SGTA50000`), accessible through the iOS Files app and iTunes / Finder file sharing.
- **Memory Safeguards**: Built with `extended-virtual-addressing` and `increased-memory-limit` entitlements to ensure stable 64-bit virtual memory allocations across devices with 4 GB+ RAM.

---

## Sideloading Methods

- **TrollStore (Recommended)**: Unrestricted memory limits without expiration or developer certificate revokes.
- **ESign / Scarlet / Feather**: Sign using your Enterprise or Developer P12 certificate. Make sure to keep `extended-virtual-addressing` and `increased-memory-limit` entitlements active in signing options.
- **AltStore / SideStore / Sideloadly**: Standard profile signing supported.
- **Xcode**: Deploy directly with your personal or paid Apple Developer account.

---

## License

This project is licensed under the **GNU General Public License v3.0 (GPL-3.0)**.
