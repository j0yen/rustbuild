# PRD: vellum-manifest — typed manifest ops keyed correctly on slug

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/vellum
**Vision:** visions/vellum.md

## TL;DR

/build joins its scan output against `manifest.json` every tick to decide what
to dispatch, and today that join is hand-rolled jq. This PRD adds `vellum
manifest` — typed get/join/set ops keyed on `slug` — so the two manifest
bug classes /build has already hit become structurally impossible.

## Why this exists

Two proven dispatch bugs live in the jq manifest layer:

- `self_build_manifest_join_slug`: the manifest `.prds` is an array where many
  entries have `path:null`; joining on `path` re-selects already-tracked PRDs.
  The join MUST key on `slug`. (Cost a wrongly-dispatched branch 2026-06-03.)
- `self_build_jq_escape_reads_absent`: a bad escape in one manifest entry makes
  the per-slug jq join error out → every slug reads ABSENT → /build mis-selects
  a blocked/user-gated PRD.

Both are the same root cause: untyped jq string-munging over a structured file.
A typed tool that parses the manifest once, keys on slug, and isolates a
malformed entry to itself removes the class.

## What this builds

- `vellum manifest` subcommand group (rust-extend `vellum`), operating on
  `~/.claude/skills/build/state/manifest.json` (path overridable via
  `--manifest <path>`):
  - `vellum manifest get <slug>` — emit the manifest entry for a slug, or a
    JSON `null`/error if absent. Never errors the whole file on one bad entry.
  - `vellum manifest join <scan.json>` — join a `vellum scan` array against the
    manifest **on slug**, emitting per-slug `{slug, scanned, tracked, status}`
    so /build's Phase-1 diff consumes typed records, not jq output.
  - `vellum manifest set <slug> <key> <value>` — set a field on a manifest
    entry, atomic write (temp+rename).
- The join keys on `slug` ONLY; a manifest entry with `path:null` is matched by
  slug, never skipped or re-selected.
- A malformed manifest entry degrades to a per-entry `{slug, error}` record; the
  join/get for OTHER slugs still succeeds (no whole-file ABSENT).

## Acceptance criteria

1. `cargo build` / `cargo test` green; clippy no new warnings; `Running` line
   per test file. MSRV 1.85, no let-chains.
2. `vellum manifest get <slug>` returns the correct entry for a tracked slug and
   a JSON `null` (exit 0) for an untracked one.
3. `vellum manifest join <scan.json>` matches scan entries to manifest entries
   on `slug`; a manifest entry with `path:null` is still matched by its slug and
   NOT re-emitted as a fresh/untracked PRD. (Closes `self_build_manifest_join_slug`.)
4. With one deliberately malformed manifest entry, `get`/`join` for all other
   slugs still succeed and the bad entry is isolated to a per-entry error — the
   whole manifest never reads ABSENT. (Closes `self_build_jq_escape_reads_absent`.)
5. `vellum manifest set <slug> <key> <value>` updates only that field, atomic
   write; the rest of the manifest is byte-stable except the touched value.
6. A fixture test runs `join` against a checked-in scan+manifest pair and
   asserts the expected per-slug status records.
