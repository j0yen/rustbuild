# PRD: vellum-amend — typed in-place frontmatter edits, retiring the sed storm

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/vellum
**Vision:** visions/vellum.md

## TL;DR

Every /build tick mutates PRD frontmatter — bumps status, adds a blocker,
appends to iter_log — and today the model does this with free-form `sed -i` in
the build loop. This PRD adds `vellum amend <PRD>` with typed flags so those
edits go through one tool that can't corrupt the file and is byte-preserving
for everything it doesn't touch.

## Why this exists

The 2026-06-08 self-review measured **sed×97,812 in a single 79-minute build
session** — the top executed binary by a 9× margin over the next. That is the
build loop hand-rolling frontmatter edits. Free-form sed is also a known
corruption hazard: a mis-anchored substitution silently rewrites the wrong
line, and there is no test surface. Memory `feedback_use_local_toolkit` says to
reach for a typed tool before hand-rolling shell; this PRD gives the loop the
tool it's missing for its single most frequent shell operation.

## What this builds

- `vellum amend <PRD>` subcommand (rust-extend `vellum`) with flags:
  - `--set-status "<line>"` — replace the `Status:` / `**Status:**` line.
  - `--add-blocker "<text>"` / `--clear-blockers` — manage the `blockers:` list.
  - `--append-iter-log "<text>"` — append a timestamped-by-caller entry to
    `iter_log:` (vellum does not stamp time — `Date::now` is unavailable in this
    environment's tooling; caller passes the full line).
  - `--set <key>=<value>` — set a scalar frontmatter key (e.g.
    `build_version_bump=minor`).
- Writes via temp-file + atomic rename (never partial). Everything the flags
  don't address is preserved byte-for-byte (assert with a round-trip test).
- Idempotent where it makes sense: `--set-status` to the current value is a
  no-op; `--add-blocker` of an existing blocker doesn't duplicate.
- `--dry-run` prints the would-be diff to stdout and writes nothing.

## Acceptance criteria

1. `cargo build` / `cargo test` green; clippy no new warnings; verify a
   `Running` line for every test file (`self_orphaned_mock_tests`). MSRV 1.85,
   no let-chains.
2. `vellum amend <PRD> --set-status "Status: Blocked"` rewrites only the status
   line; a byte-diff of the file shows exactly that one line changed.
3. `--add-blocker` adds to the blockers list; a second `--add-blocker` of the
   same text is a no-op (no duplicate). `--clear-blockers` empties it.
4. `--append-iter-log "<line>"` appends one entry; the rest of the file is
   byte-identical (round-trip test asserts this).
5. The write is atomic: a fixture test (or an injected write failure) leaves the
   original file intact, never a truncated/partial file.
6. `--dry-run` writes nothing and prints the intended change; exit 0.
7. Amending a PRD then `vellum read`-ing it yields a model reflecting the edit
   (read/amend round-trip consistency).
