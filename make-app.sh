#!/bin/sh
# muguet - make-app.sh
# Constantin Clerc (c22dev) - 0.1

set -eu
cd "$(dirname "$0")"
APP="$PWD/build/Muguet.app"
GAME="${1:-}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$HOME=/home --remap-path-prefix=$PWD=/muguet"

cargo build --release
BIN=target/release/muguet
EMBEDDED=false
if [ -n "$GAME" ]; then
  mkdir -p target/mac-game
  case "$GAME" in
    *.zip)
      ENTRY="$(unzip -Z1 "$GAME" | grep -E '(^|/)b/[^/]+/game\.wasm$' | head -1)"
      [ -n "$ENTRY" ] || { echo "$GAME has no b/<build>/game.wasm"; exit 1; }
      unzip -p "$GAME" "$ENTRY" > target/mac-game/game.wasm ;;
    *) cp "$GAME" target/mac-game/game.wasm ;;
  esac
  ./target/release/muguet --precompile target/mac-game/game.wasm target/mac-game/game.cwasm
  rm target/mac-game/game.wasm
  MUGUET_GAME_CODE="$PWD/target/mac-game/game.cwasm" cargo build --release --no-default-features --target-dir target/embed
  BIN=target/embed/release/muguet
  EMBEDDED=true
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" target/release/muguet-server "$APP/Contents/MacOS/"
swiftc -O -parse-as-library -target arm64-apple-macos14 launcher/Launcher.swift -o "$APP/Contents/MacOS/Launcher"
cp assets/*.mp3 "$APP/Contents/Resources/"

xcrun actool assets/muguet.icon --compile "$APP/Contents/Resources" --app-icon muguet --platform macosx \
  --minimum-deployment-target 14.0 --output-partial-info-plist "$(mktemp -d)/icon.plist" >/dev/null

cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Muguet</string>
  <key>CFBundleDisplayName</key><string>Muguet</string>
  <key>CFBundleIdentifier</key><string>ch.cclerc.muguet</string>
  <key>CFBundleExecutable</key><string>Launcher</string>
  <key>CFBundleIconFile</key><string>muguet</string>
  <key>CFBundleIconName</key><string>muguet</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>0.1</string>
  <key>CFBundleVersion</key><string>1</string>
  <key>LSMinimumSystemVersion</key><string>14.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.games</string>
  <key>GCSupportsGameMode</key><true/>
  <key>NSLocalNetworkUsageDescription</key><string>Multiplayer connects to a game server on your local network.</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><false/>
  <key>MuguetEmbeddedCode</key><EMBEDDED/>
</dict>
</plist>
PLIST
sed -i '' "s|<EMBEDDED/>|<$EMBEDDED/>|" "$APP/Contents/Info.plist"

codesign --force --deep --sign - "$APP"
echo "built: $APP ($(du -sh "$APP" | cut -f1))"
