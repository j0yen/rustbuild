# PRD: homeward-ingestd-install — build and install the missing ingest daemon binary

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
build_version_bump: patch
build_priority: high

## TL;DR

`homeward-ingest.service` is installed and linked, but the binary it calls
(`~/.local/bin/homeward-ingestd`) has never been built or installed. The
homeward fleet cannot run without it — `homeward up` brings up the embed
sidecar and report server, but the ingest daemon fails on start because
the binary is absent. The source is complete at
`homeward-ingest/src/main.rs`. This PRD builds it via cloudbuild, installs
it to `~/.local/bin/homeward-ingestd`, and verifies the service can start.

## Why this exists

homeward v0.18.0 ships POST /search wired to the embed sidecar. But the
vector gallery is empty: the ingest daemon is what reads shelter sources,
deduplicates PetRecords, enrolls photos into the DINOv2 store, and detects
departures. Without `homeward-ingestd` running, search always returns zero
matches. The code is written and compiles; the gap is purely the missing
install step.

Root cause: the `cargo build --release` that installed `homeward-reportd`
did not also install `homeward-ingestd` (they live in different workspace
crates: `homeward-report` vs `homeward-ingest`). The deploy `install.sh`
only links unit files; it does not build or install Rust binaries.

## What this builds

1. **Build**: `cargo build --release -p homeward-ingest` (via cloudbuild).
   Route all cargo invocations through cloudbuild.
2. **Install**: `install -m755 target/release/homeward-ingestd ~/.local/bin/homeward-ingestd`
3. **Smoke-test**: `homeward-ingestd stats` (exits 0 even with an empty
   database; confirms the binary is present and the DB path is writable).
4. **Service check**: `systemctl --user start homeward-ingest.service &&
   systemctl --user is-active homeward-ingest.service` — verify the daemon
   comes up. If it fails, capture `journalctl --user -u homeward-ingest --lines 30`
   and leave last_error in the manifest.
5. **Bump**: workspace version 0.18.0 → 0.18.1 in root Cargo.toml and
   homeward-ingest/Cargo.toml (patch; no API change, install gap closure).
6. **Changelog**: prepend `## v0.18.1` entry to homeward/CHANGELOG.md.

## Acceptance criteria

1. `~/.local/bin/homeward-ingestd` exists and `homeward-ingestd --help`
   exits 0.
2. `homeward-ingestd stats` exits 0 (empty DB is valid; error exit is not).
3. `systemctl --user is-active homeward-ingest.service` exits 0 after start.
4. `homeward/CHANGELOG.md` has a `## v0.18.1` section.
5. `cargo test --release -p homeward-ingest` passes (via cloudbuild).
