#!/usr/bin/env bash
# Offline check that the install.sh scripts/package-linux.sh puts in the tarball
# writes a launcher entry with absolute paths plus the icon, idempotently, under
# a throwaway HOME. Nothing touches the network, systemd, or the real home.
#
# Usage: scripts/test-linux-desktop-entry.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
VERSION=9.9.9
fail() { echo "FAIL: $*" >&2; exit 1; }

# A fake release: the tarball layout package-linux.sh produces, minus the real
# binary.
PKG="space-$VERSION-linux-x86_64"
mkdir -p "$WORK/pkg/$PKG" "$WORK/tmp"
printf '#!/bin/sh\nexit 0\n' >"$WORK/pkg/$PKG/space"
chmod 755 "$WORK/pkg/$PKG/space"
cp "$ROOT/dist/space.desktop" "$WORK/pkg/$PKG/space.desktop"
printf 'not-really-a-png' >"$WORK/pkg/$PKG/space.png"

# The tarball's install.sh lives in a heredoc inside package-linux.sh.
sed -n "/<<'INSTALL'/,/^INSTALL\$/p" "$ROOT/scripts/package-linux.sh" | sed '1d;$d' \
  | sed "s/__VERSION__/$VERSION/" >"$WORK/pkg/$PKG/install.sh"
chmod 755 "$WORK/pkg/$PKG/install.sh"

# Each run gets its own TMPDIR, which must be empty again afterwards.
run_tarball() {
  local home="$1"; shift
  env -i HOME="$home" USER=tester PATH="/usr/bin:/bin" TMPDIR="$WORK/tmp" "$@" \
    bash "$WORK/pkg/$PKG/install.sh" >"$WORK/out.log" 2>&1 \
    || { cat "$WORK/out.log" >&2; fail "tarball installer exited non-zero"; }
  [ -z "$(ls -A "$WORK/tmp")" ] || fail "tarball installer left files in TMPDIR: $(ls -A "$WORK/tmp")"
}
# check HOME DATA_HOME
check() {
  local home="$1" data="$2" entry="$2/applications/space.desktop"
  [ -f "$entry" ] || fail "missing $entry"
  [ -f "$data/icons/hicolor/1024x1024/apps/space.png" ] || fail "missing hicolor icon"
  # `$(...)` strips nothing needed here: paths in these tests have no newlines.
  grep -qxF "TryExec=$home/.space/app/current/space" "$entry" || fail "TryExec: $(grep '^TryExec' "$entry")"
  grep -qxF "Icon=$home/.space/app/current/space.png" "$entry" || fail "Icon: $(grep '^Icon' "$entry")"
  grep -qxF "StartupWMClass=space" "$entry" || fail "StartupWMClass changed"
  [ "$(grep -c '^\[Desktop Entry\]' "$entry")" = 1 ] || fail "duplicated entry"
  [ "$(grep -c '^Exec=' "$entry")" = 1 ] || fail "Exec lines"
  [ -z "$(find "$data" -name '.space*')" ] || fail "temp files left behind"
  # A user-level icon cache is only ever refreshed, never created.
  [ ! -e "$data/icons/hicolor/icon-theme.cache" ] || fail "created a hicolor icon cache"
  if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate "$entry" || fail "desktop-file-validate"
  fi
}
run() { run_tarball "$@"; }

# Default XDG location, then a re-run (how updates are installed) is stable.
home="$WORK/tarball-a/home"; mkdir -p "$home"
run "$home"
check "$home" "$home/.local/share"
grep -qxF "Exec=$home/.space/app/current/space %u" "$home/.local/share/applications/space.desktop" \
  || fail "tarball: Exec line"
before="$(cat "$home/.local/share/applications/space.desktop")"
run "$home"
check "$home" "$home/.local/share"
[ "$before" = "$(cat "$home/.local/share/applications/space.desktop")" ] || fail "tarball: re-run changed the entry"

# XDG_DATA_HOME wins when absolute; a relative value is ignored per the spec.
home="$WORK/tarball-b/home"; mkdir -p "$home"
run "$home" XDG_DATA_HOME="$WORK/tarball-b/xdg"
check "$home" "$WORK/tarball-b/xdg"
[ ! -e "$home/.local/share/applications" ] || fail "tarball: wrote outside XDG_DATA_HOME"
home="$WORK/tarball-c/home"; mkdir -p "$home"
run "$home" XDG_DATA_HOME=relative/dir
check "$home" "$home/.local/share"

# A home with a space and characters the Exec key must quote and escape.
home="$WORK/tarball-d/h o\$me\"x%y"; mkdir -p "$home"
run "$home"
check "$home" "$home/.local/share"
# Spec: quote the argument, `\` before " and $ (doubled again for the file's
# string escaping), and `%%` for a literal `%`.
want="Exec=\"$WORK/tarball-d/"'h o\\$me\\"x%%y'"/.space/app/current/space\" %u"
grep -qxF "$want" "$home/.local/share/applications/space.desktop" \
  || fail "tarball: Exec quoting: $(grep '^Exec=' "$home/.local/share/applications/space.desktop")"
echo "ok: tarball installer"
