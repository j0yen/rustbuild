# PRD: vellum-scan — drop-in replacement for scan-prds.sh's array output

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/vellum
**Vision:** visions/vellum.md

## TL;DR

`scan-prds.sh` emits a JSON array describing every PRD, consumed by /build's
Phase-1 diff every tick. This PRD adds `vellum scan <dir>` — the same array,
emitted from the typed `vellum read` core, with a malformed PRD degrading to a
per-entry error object instead of aborting the whole scan.

## Why this exists

`scan-prds.sh` builds its array by piping each PRD's fields through `jq` with
`--arg`/`--argjson`. A bad escape in ONE entry makes the per-slug jq join error
out, so EVERY slug reads ABSENT and /build mis-selects a blocked/user-gated PRD
(`self_build_jq_escape_reads_absent`, which cost a wrongly-dispatched branch
2026-06-03). The 2026-06-08 self-review also measured jq×2,696 in one session —
the array assembly is a hot path. A typed scanner over the `vellum read` core
removes the whole-scan-aborts-on-one-bad-entry failure mode and the jq
dependency.

## What this builds

- `vellum scan <dir>` subcommand (rust-extend the `vellum` crate). Walks `<dir>`
  for `PRD-*.md`, EXCLUDING any `PRDs-archive/` subdir, exactly as
  `scan-prds.sh` does.
- For each PRD, reuses the `vellum read` parser and emits the object shape
  `scan-prds.sh` emits: `{slug, path, build_auto, build_target, build_priority,
  build_into, build_version_bump, deferred_acs, deferred_ac_reasons,
  status_line, size_bytes, mtime_iso}`. `build_auto` is hardcoded `true` per
  user instruction 2026-05-27 (`feedback_always_commit_push`), matching the
  current script.
- A PRD that fails to parse emits `{slug, path, error}` in its array slot; the
  scan continues and exits 0. (The whole-scan never aborts on one bad file.)
- `--legacy-compat` flag (default on) guarantees the key set and types match
  `scan-prds.sh` byte-for-byte for well-formed PRDs, so it's a literal drop-in.

## Acceptance criteria

1. `cargo build` / `cargo test` green; clippy adds no new warnings. Verify a
   `Running` line for EVERY existing vellum test file in cargo output before
   trusting green (`self_orphaned_mock_tests`). MSRV 1.85, no let-chains.
2. `vellum scan ~/wintermute/autobuilder` emits a JSON array with one object per
   non-archived `PRD-*.md`; `PRDs-archive/` entries are excluded.
3. Each object carries the full `scan-prds.sh` key set with matching types;
   `build_auto` is `true` for every entry.
4. For a corpus where one PRD is deliberately malformed, the scan still exits 0
   and emits a per-entry `error` object for that PRD while every other entry is
   well-formed. (Closes `self_build_jq_escape_reads_absent`.)
5. On the live corpus, `vellum scan` and `scan-prds.sh` produce the same set of
   slugs with the same `build_target`/`status_line`/`build_into` values for all
   well-formed PRDs (parity check — assert in a test or a checked-in golden).
6. `deferred_acs` is populated from all three forms (inheriting vellum-read's
   parser), so block-form PRDs no longer scan to `[]`.
