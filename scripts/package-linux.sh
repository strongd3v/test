#!/usr/bin/env bash
# Linux packaging: build the release binary and produce
#   target/package/space-<version>-linux.tar.gz
# containing the binaries (the app and its background engine), the .desktop
# entry, and the icon, plus an install.sh
# that installs them into the self-updating ~/.space/app layout and links
# ~/.local (XDG) paths to it. The tarball is the internal-update payload the
# auto-updater and the curl installer fetch from the CDN — release runs carry
# a x86_64 AppImage instead:
#   target/package/space-<version>-linux.appimage
#
#
# Usage: scripts/package-linux.sh
# Env:   PROFILE=debug for a fast unoptimized package (CI smoke); default release.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
command -v cargo >/dev/null 2>&1 || PATH="$HOME/.cargo/bin:$PATH"
PROFILE="${PROFILE:-release}"
VERSION="$(grep -m1 '^version' "$ROOT/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
OUT_DIR="$ROOT/target/package"
STAGE="$OUT_DIR/space-$VERSION-linux"
TARBALL="$STAGE.tar.gz"

cd "$ROOT"
if [[ "$PROFILE" == "release" ]]; then
  cargo build --release -p space
  BIN="$ROOT/target/release/space"
else
  cargo build -p space
  BIN="$ROOT/target/debug/space"
fi
# The background engine the app starts: sessions keep running with the app
# closed, in a process of its own.
ENGINE_BIN="$(dirname "$BIN")/space-engine"

rm -rf "$STAGE" "$TARBALL"
mkdir -p "$STAGE"
install -m 755 "$BIN" "$STAGE/space"
install -m 755 "$ENGINE_BIN" "$STAGE/space-engine"
install -m 644 "$ROOT/dist/space.desktop" "$STAGE/space.desktop"
install -m 644 "$ROOT/dist/space.png" "$STAGE/space.png"
install -m 644 "$ROOT/LICENSE.md" "$STAGE/"
mkdir -p "$STAGE/licenses/fonts"
cp "$ROOT/crates/ui/assets/fonts/licenses/"* "$STAGE/licenses/fonts/"
cp "$ROOT/crates/voice/NOTICE.md" "$STAGE/licenses/parakeet-v3.txt"

cat >"$STAGE/install.sh" <<'INSTALL'
#!/usr/bin/env bash
# Install Space for this user (no root needed), in the layout the in-app
# updater manages: ~/.space/app/<version> behind a `current` symlink — the
# same layout `curl -fsSL https://space.pentomake.com/install.sh | sh` uses — with
# ~/.local/bin/space and the desktop entry pointing through it.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION="__VERSION__"
APP_ROOT="$HOME/.space/app"
DEST="$APP_ROOT/$VERSION"
mkdir -p "$APP_ROOT"
if [ ! -x "$DEST/space" ]; then
  # Copy beside the final name, then rename: an interrupted install never
  # leaves a half-copied version the updater would trust.
  STAGE="$(mktemp -d "$APP_ROOT/.install-$VERSION-XXXXXX")"
  cp -R "$HERE/." "$STAGE/"
  rm -rf "$DEST"
  mv "$STAGE" "$DEST"
fi
if ! "$DEST/space" --version >/dev/null; then
  echo "Space could not start; see the loader error above. Install the missing runtime libraries (including ALSA, libasound.so.2), then retry." >&2
  exit 1
fi
ln -sfn "$DEST" "$APP_ROOT/current"
mkdir -p "$HOME/.local/bin"
ln -sfn "$APP_ROOT/current/space" "$HOME/.local/bin/space"

# Launchers list Space through a per-user .desktop entry. The one in the tarball
# says `Exec=space` and `TryExec=space`, which only resolve when ~/.local/bin is
# on the PATH of the desktop session (often not, e.g. a bare Wayland + fuzzel
# setup) and TryExec then hides the entry outright. So write it with absolute
# paths through the `current` symlink, which keeps working across updates. The
# icon is referenced by path too: the only artwork is 1024x1024, a size the
# hicolor theme doesn't index, so a name lookup alone can come up empty.
install_desktop_entry() {
  src="$1"
  app="$2"
  [ -f "$src/space.desktop" ] && [ -f "$src/space.png" ] || return 1
  case "${XDG_DATA_HOME:-}" in
    /*) data_home="$XDG_DATA_HOME" ;;
    *) data_home="$HOME/.local/share" ;;
  esac
  apps_dir="$data_home/applications"
  icon_dir="$data_home/icons/hicolor/1024x1024/apps"
  bin="$app/current/space"
  icon="$app/current/space.png"
  # Desktop Entry `Exec` quoting: double-quote an argument with reserved
  # characters, backslash-escape ", `, $ and \ inside, then double every
  # backslash again for the file's own string escaping. `%` must be `%%`.
  case "$bin" in
    *[!A-Za-z0-9_./-]*)
      exec_bin="\"$(printf '%s' "$bin" | sed -e 's/\\/\\\\\\\\/g' -e 's/["`$]/\\\\&/g' -e 's/%/%%/g')\""
      ;;
    *) exec_bin="$bin" ;;
  esac
  try_bin="$(printf '%s' "$bin" | sed 's/\\/\\\\/g')"
  icon_val="$(printf '%s' "$icon" | sed 's/\\/\\\\/g')"

  mkdir -p "$apps_dir" "$icon_dir" || return 1
  # Write beside the final name, then rename, so a launcher watching the
  # directory never reads a half-written entry (a leading dot is ignored).
  # Not `tmp`: sh has no `local`, and the curl installer's EXIT trap removes
  # its download dir through `$tmp`.
  entry_tmp="$apps_dir/.space.desktop.$$"
  while IFS= read -r line || [ -n "$line" ]; do
    case "$line" in
      Exec=*) printf 'Exec=%s %%u\n' "$exec_bin" ;;
      TryExec=*) printf 'TryExec=%s\n' "$try_bin" ;;
      Icon=*) printf 'Icon=%s\n' "$icon_val" ;;
      *) printf '%s\n' "$line" ;;
    esac
  done <"$src/space.desktop" >"$entry_tmp" || { rm -f "$entry_tmp"; return 1; }
  mv -f "$entry_tmp" "$apps_dir/space.desktop" || { rm -f "$entry_tmp"; return 1; }
  cp "$src/space.png" "$icon_dir/.space.png.$$" \
    && mv -f "$icon_dir/.space.png.$$" "$icon_dir/space.png" || return 1

  # Best-effort cache refresh; both tools are optional. The icon cache is only
  # refreshed, never created: a user-level hicolor cache nobody else maintains
  # would hide icons other apps later install there, and the entry above
  # references the icon by path anyway.
  command -v update-desktop-database >/dev/null 2>&1 \
    && update-desktop-database "$apps_dir" >/dev/null 2>&1 || true
  [ -f "$data_home/icons/hicolor/icon-theme.cache" ] \
    && command -v gtk-update-icon-cache >/dev/null 2>&1 \
    && gtk-update-icon-cache -q -t -f "$data_home/icons/hicolor" >/dev/null 2>&1 || true
  return 0
}
install_desktop_entry "$HERE" "$APP_ROOT" \
  || echo "warn: could not install the desktop entry — Space won't appear in application launchers"

echo "Installed Space $VERSION. It updates itself from now on."
case ":$PATH:" in
  *":$HOME/.local/bin:"*) ;;
  *) echo "Add ~/.local/bin to your PATH to run \`space\` from a terminal." ;;
esac
INSTALL
sed -i "s/__VERSION__/$VERSION/" "$STAGE/install.sh"
chmod 755 "$STAGE/install.sh"

tar -czf "$TARBALL" -C "$OUT_DIR" "$(basename "$STAGE")"
rm -rf "$STAGE"
echo "packaged: $TARBALL"

# The release AppImage (x86_64): a self-sufficient single file for the GitHub
# release. The tarball above stays the update payload; the AppImage is what
# new users download. `appimagetool --appimage-extract-and-run` needs no FUSE,
# which stock GitHub runners don't provide; override its download location
# with APPIMAGETOOL_URL if a mirror is preferred.
if [[ "$(uname -m)" == "x86_64" ]]; then
  APPDIR="$OUT_DIR/space-$VERSION-linux.AppDir"
  rm -rf "$APPDIR"
  mkdir -p "$APPDIR/usr/bin" \
    "$APPDIR/usr/share/applications" \
    "$APPDIR/usr/share/icons/hicolor/1024x1024/apps"
  install -m 755 "$BIN" "$APPDIR/usr/bin/space"
  # The background engine the app starts: sessions keep running with the app
  # closed, in a process of its own.
  install -m 755 "$ENGINE_BIN" "$APPDIR/usr/bin/space-engine"
  install -m 644 "$ROOT/dist/space.desktop" "$APPDIR/usr/share/applications/space.desktop"
  install -m 644 "$ROOT/dist/space.desktop" "$APPDIR/space.desktop"
  install -m 644 "$ROOT/dist/space.png" "$APPDIR/usr/share/icons/hicolor/1024x1024/apps/space.png"
  install -m 644 "$ROOT/dist/space.png" "$APPDIR/space.png"
  install -m 755 /dev/null "$APPDIR/AppRun"
  cat >"$APPDIR/AppRun" <<'APPRUN'
#!/bin/sh
APPDIR="$(dirname "$(readlink -f "$0")")"
exec "$APPDIR/usr/bin/space" "$@"
APPRUN
  chmod 755 "$APPDIR/AppRun"
  ln -sfn "space.png" "$APPDIR/.DirIcon"
  TOOL="${APPIMAGETOOL_URL:-https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage}"
  TOOL_FILE="$OUT_DIR/appimagetool-x86_64.AppImage"
  curl -fsSL "$TOOL" -o "$TOOL_FILE"
  chmod +x "$TOOL_FILE"
  "$TOOL_FILE" --appimage-extract-and-run -n "$APPDIR" "$OUT_DIR/space-$VERSION-linux.appimage"
  rm -rf "$APPDIR" "$TOOL_FILE"
  echo "packaged: $OUT_DIR/space-$VERSION-linux.appimage"
fi

tar -tzf "$TARBALL"
