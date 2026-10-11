# Muguet private iOS build

This repository contains build orchestration only. The source wrapper is
checked out from the public `iOSVNNews/muguet` fork at an explicit revision.
Game inputs and compiled IPA outputs are not committed to this repository.

Run **Build Muguet IPA with game code** manually. The private artifact contains
the arm64 IPA and its SHA-256 checksum. The build checks the original WASM hash,
precompiles it for iOS, and verifies the resulting `__GAME` executable segment.

The original 63,201,802-byte WASM input is downloaded in validated HTTP ranges
to avoid the mirror's interrupted large responses. Its required SHA-256 is
`11ca8d2c04c5e843d18ff4aea4899d72c86973c6b031df334e67c446b2ae83e0`.

Game data and shaders are separate inputs. Import a compatible `game.zip`
containing `data/manifest.json` and `b/<build>/shaders/index.json` after installing
the application. No on-device execution or gameplay result is claimed by this
build job.
