#!/usr/bin/env bash
# The Linux Rust test set as ONE build and ONE nextest run (separate
# invocations recompiled the same dependency graph with different feature sets).
#
# Coverage:
#   space-harness  every test binary (`--test pi_rpc` uses native-fixture; that
#                  only adds the pi_rpc fixture binaries, so it is on throughout)
#   space-preview  every test binary (preview-tests)
#   space-doc      lib + persistence and memory regressions + attachments_roundtrip
#   space-sync, space-update   lib only (session-sync-regressions)
#   space-engine   lib + the integration tests named below. Engine has many other
#                  integration binaries (live agents etc.) that no workflow ran, so
#                  they are listed instead of globbed to avoid compiling them.
#   voice (moved from voice-tests.yml offline-linux, which ran them as 14 separate
#   `cargo test` builds): space-voice-media, space-voice-session and space-orb
#                  lib, space-proto lib `voice` tests, space-mcp lib, space-rpc
#                  lib and engine voice_runtime. They need no audio
#                  stack, and harness already depends on all of them.
# nextest does not run doctests; these crates have none (their doc comments
# contain no Rust code blocks).
# Extra arguments are passed to nextest (the nightly job adds `--release`).
# NEXTEST_PROFILE=ci (pull requests and main) leaves out the slow and soak tests
# that scripts/ci/nextest.toml lists; the nightly runs use the default profile.
set -euo pipefail
cd "$(dirname "$0")/../.."

# Test binaries that run nothing on Linux are left out: they would still be
# compiled and linked. Windows-only files (#![cfg(windows)]), and files whose
# every test is #[ignore]d because it needs a live service or a real CLI.
skip=" codex_availability windows_native managed_install muse_account_live pi_live real_acp_lifecycle real_quiet_survey coordinator "
tests=()
for f in crates/harness/tests/*.rs crates/preview/tests/*.rs crates/doc/tests/*.rs; do
  name="$(basename "$f" .rs)"
  [[ "$skip" == *" $name "* ]] || tests+=(--test "$name")
done
# `misc` (harness, engine and preview): the small suites, one binary each.
# The orb geometry tests live in space-orb's `geometry`.
for t in misc geometry born_doc_race restart_resume side_chats message_queue voice_runtime; do
  tests+=(--test "$t")
done

exec cargo nextest run --config-file scripts/ci/nextest.toml --locked --no-fail-fast \
  -p space-harness -p space-engine -p space-sync -p space-update -p space-doc -p space-preview \
  -p space-mcp -p space-proto -p space-rpc -p space-orb -p space-voice-session -p space-voice-media \
  --features space-harness/native-fixture \
  --lib --bins "${tests[@]}" \
  -E 'not ((package(space-proto) & not test(voice)) | (package(space-rpc) & kind(lib)))' "$@"
