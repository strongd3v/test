#!/usr/bin/env bash
# Build and run an isolated macOS development bundle. Space.app may remain
# open: this bundle has a separate LaunchServices/TCC identity, data directory,
# and engine IPC port.
# Set SPACE_DEV_BUILD_ONLY=1 to prepare the signed bundle without launching it.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v cargo >/dev/null 2>&1 || PATH="$HOME/.cargo/bin:$PATH"
VERSION="$(grep -m1 '^version' "$ROOT/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
DEV_ROOT="$ROOT/target/macos-dev"
APP="$DEV_ROOT/Space Dev.app"
CONTENTS="$APP/Contents"
DATA_DIR="${SPACE_DEV_DATA_DIR:-$DEV_ROOT/data}"
IPC_PORT="${SPACE_DEV_IPC_PORT:-49777}"
BUNDLE_ID="${SPACE_DEV_BUNDLE_ID:-com.pentomake.space.dev}"

if pgrep -f -x "$CONTENTS/MacOS/space" >/dev/null 2>&1; then
  echo "Space Dev is already running. Quit it before rebuilding the signed bundle." >&2
  exit 1
fi
# The app's background engine keeps running after the app quits. Rebuilding
# replaces its executable, so stop this bundle's engine (only this bundle's:
# matched by its exact path) — its sessions stop with it.
ENGINE="$CONTENTS/MacOS/Space Engine"
if pgrep -f -x "$ENGINE" >/dev/null 2>&1; then
  echo "Stopping Space Dev's background engine before rebuilding." >&2
  pkill -TERM -f -x "$ENGINE" || true
  for _ in $(seq 1 100); do
    pgrep -f -x "$ENGINE" >/dev/null 2>&1 || break
    sleep 0.1
  done
  if pgrep -f -x "$ENGINE" >/dev/null 2>&1; then
    echo "Space Dev's background engine didn't stop; quit it and retry." >&2
    exit 1
  fi
fi

cd "$ROOT"
cargo build -p space

mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources" "$DATA_DIR"
TARGET_DIR="$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
install -m 755 "$TARGET_DIR/debug/space" "$CONTENTS/MacOS/space"
install -m 755 "$TARGET_DIR/debug/space-engine" "$ENGINE"
sed "s/__VERSION__/$VERSION/g" "$ROOT/dist/macos/Info-dev.plist" >"$CONTENTS/Info.plist"
plutil -replace CFBundleIdentifier -string "$BUNDLE_ID" "$CONTENTS/Info.plist"
plutil -replace LSEnvironment.SPACE_DATA_DIR -string "$DATA_DIR" "$CONTENTS/Info.plist"
plutil -replace LSEnvironment.SPACE_IPC_PORT -string "$IPC_PORT" "$CONTENTS/Info.plist"
if [[ -n "${CODEX_EXECUTABLE:-}" ]]; then
  plutil -insert LSEnvironment.CODEX_EXECUTABLE -string "$CODEX_EXECUTABLE" "$CONTENTS/Info.plist"
fi


mkdir -p "$CONTENTS/Resources/licenses"
cp "$ROOT/crates/voice/NOTICE.md" "$CONTENTS/Resources/licenses"/parakeet-v3.txt

if [[ ! -f "$CONTENTS/Resources/space.icns" ]]; then
  ICONSET="$DEV_ROOT/space-dev.iconset"
  mkdir -p "$ICONSET"
  for size in 16 32 128 256 512; do
    sips -z "$size" "$size" "$ROOT/dist/macos/icon-1024.png" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    retina=$((size * 2))
    sips -z "$retina" "$retina" "$ROOT/dist/macos/icon-1024.png" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
  done
  iconutil -c icns "$ICONSET" -o "$CONTENTS/Resources/space.icns"
fi

# A real Apple Development identity gives TCC a stable signing requirement
# across rebuilds. Set SPACE_DEV_CODESIGN_IDENTITY explicitly when more than
# one identity is installed; otherwise fall back to an ad-hoc signature.
IDENTITY="${SPACE_DEV_CODESIGN_IDENTITY:-}"
if [[ -z "$IDENTITY" ]]; then
  IDENTITY="$(security find-identity -v -p codesigning 2>/dev/null | sed -n 's/.*"\(Apple Development:[^"]*\)".*/\1/p' | head -1)"
fi
# Inside out: the engine first, then the bundle that seals it.
if [[ -n "$IDENTITY" ]]; then
  codesign --entitlements "$ROOT/dist/macos/Space.entitlements" --force --sign "$IDENTITY" --identifier "$BUNDLE_ID.engine" "$ENGINE"
  codesign --entitlements "$ROOT/dist/macos/Space.entitlements" --force --sign "$IDENTITY" --identifier "$BUNDLE_ID" "$APP"
else
  codesign --entitlements "$ROOT/dist/macos/Space.entitlements" --force --sign - --identifier "$BUNDLE_ID.engine" "$ENGINE"
  codesign --entitlements "$ROOT/dist/macos/Space.entitlements" --force --sign - --identifier "$BUNDLE_ID" "$APP"
  echo "warning: no Apple Development signing identity found; macOS may ask for permissions again after a rebuild" >&2
fi

if [[ "${SPACE_DEV_BUILD_ONLY:-0}" == "1" ]]; then
  echo "built: $APP" >&2
  exit 0
fi

echo "running Space Dev (bundle $BUNDLE_ID, data $DATA_DIR, IPC $IPC_PORT)" >&2
# LaunchServices must own the process. Launching Contents/MacOS/space directly
# makes TCC attribute Screen Recording to the terminal (Warp, Terminal, etc.).
# -W keeps the script attached until the app exits. Runtime logs remain in the
# isolated data directory (`target/macos-dev/data/logs/space-headed.log`).
OPEN_ENV=(
  --env "SPACE_AUTO_UPDATE=0"
  --env "SPACE_DATA_DIR=$DATA_DIR"
  --env "SPACE_IPC_PORT=$IPC_PORT"
)
if [[ -n "${CODEX_EXECUTABLE:-}" ]]; then
  OPEN_ENV+=(--env "CODEX_EXECUTABLE=$CODEX_EXECUTABLE")
fi

if [[ -n "${SPACE_OPEN_ROUTE:-}" ]]; then
  OPEN_ENV+=(--env "SPACE_OPEN_ROUTE=$SPACE_OPEN_ROUTE")
fi
exec open -W "${OPEN_ENV[@]}" "$APP" --args "$@"
