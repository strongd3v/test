#!/usr/bin/env bash
# First-run setup on a throwaway install: a fresh data dir every run, so the
# app opens on the welcome exactly like a new download. Your real install,
# settings and logins are untouched.
#
#   scripts/onboarding-demo.sh               # from the welcome
#   scripts/onboarding-demo.sh providers     # jump to a step (appearance,
#                                            # sync, providers, done)
set -euo pipefail
cd "$(dirname "$0")/.."

DIR=/tmp/space-onboarding-demo
IPC=27931

echo "▸ building…"
cargo build -p space -q

rm -rf "$DIR"
mkdir -p "$DIR"
echo "▸ opening a fresh install (data in $DIR)"
# Embedded: the throwaway install's engine ends with the window.
exec env SPACE_DATA_DIR="$DIR/data" SPACE_IPC_PORT=$IPC SPACE_EMBED_ENGINE=1 \
  ${1:+SPACE_ONBOARDING=$1} RUST_LOG=warn \
  ./target/debug/space
