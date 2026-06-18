# PRD: colophon-attribute — which session wrote this pile of files?

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/colophon
Vision: visions/colophon.md

## TL;DR

Disk-cruft sweeps on this laptop delete by mtime and guess at ownership. Now
that provfs stamps every file with its writing actor, this PRD adds
`colophon attribute <dir>`: walk a tree, read each file's provenance, and emit
a ranked report grouping files and bytes by the session/skill that wrote them.

## Why this exists

Phase-1 inspection, 2026-06-18:

- The recurring self-review headline is cruft of *unknown origin*: the
  2026-06-17 journal records `51 build-worktrees consuming 11G — /build creates
  but sometimes doesn't clean them up`, plus `65 dirty repos` and DISK at 99%.
  The sweeps that follow (ballast/careen/drydock) reap by size and age, never by
  *who made it*.
- provfs now makes origin a fact, not a guess. Across
  `~/wintermute/autobuilder/*.md`, grouping the live `comm-chain` heads already
  separates writers:
  `getfattr` over 20 files → distinct `comm-chain:Bun Pool N>claude-dream-he…`
  vs `comm-chain:Bun Pool N>zsh>xterm` actors. The data to answer "which skill
  leaked this" is sitting in the xattrs.
- provfs **skips** `target`, `.git`, `node_modules`, `/tmp`, `/proc` by design
  (`provfs_lsm.c` skip filter; confirmed live —
  `getfattr ~/wintermute/corpus-attest/target/debug/libcorpus_attest.d`
  returns no prov xattr). A naive walker would report all of `target/` as
  "unattributed" and drown the signal. Attribution must treat skip-prefixed
  paths as *skipped-by-design*, distinct from *stamped-but-unparseable*.
- colophon-parse (foundation PRD) already turns a single xattr into a
  `Provenance` + `originating_skill()`; this PRD is the tree-level aggregator on
  top of it.

## What this builds

Extends `~/wintermute/colophon/` (the crate from PRD-colophon-parse). No new
repo.

- **`Attribution` report model:** per-actor buckets — actor key (skill name, or
  comm-chain head + cwd when no skill resolves), `file_count`, `total_bytes`,
  `oldest_ts`, `newest_ts`, and a sample of paths. Plus a `skipped` bucket
  (count of skip-prefixed paths) and an `unstamped` bucket (walked, non-skipped,
  but no `user.prov.session` — e.g. files written before provfs booted).
- **`fn attribute(root, opts) -> Attribution`** — walk `root` (bounded depth,
  no symlink-follow), classify each entry: skip-prefixed → `skipped`; else
  `read_file` provenance via colophon-parse; group by `originating_skill()` or
  fallback key. Pure aggregation over the parse layer; the walk is the only I/O.
- **`colophon attribute <dir>`** subcommand: `--format text|json`, `--top N`
  (default 10 actors), `--min-bytes` to suppress noise, `--by skill|actor|cwd`
  to choose the grouping key. Text output is a ranked table; the headline line
  is human-readable ("11.0G across 412 files written by /build over 51
  sessions").
- Tests over a `tempdir` fixture tree with hand-set xattrs (and a skip-prefixed
  `target/` subdir) so aggregation, skip-handling, and the unstamped bucket are
  all deterministic without depending on live provfs.

Out of scope: stale detection (colophon-stale), the self-review digest block
(colophon-digest), mutating/deleting anything (attribute is strictly
read-only — it reports, the user/sweeper decides).

## Acceptance criteria

1. `cargo build`/`cargo test` green in `~/wintermute/colophon`; `cargo clippy`
   adds no new warnings over the autobuilder baseline; the `colophon` binary
   still installs and `colophon parse` still works (no regression to the
   foundation PRD).
2. Against a tempdir fixture with files carrying three distinct writing actors,
   `colophon attribute <dir> --format json` reports exactly three actor buckets
   with correct `file_count` and `total_bytes` per bucket.
3. A `target/` subdir of the fixture whose files carry no prov xattr is counted
   in the `skipped` bucket, NOT in `unstamped` and NOT as an actor — proving
   skip-by-design and missing-stamp are distinguished.
4. A non-skipped file with no `user.prov.session` lands in the `unstamped`
   bucket.
5. `--by skill` groups `claude-dream*` / `claude-build*` chains under their
   skill names; `--by cwd` groups by writing cwd; `--top 2` returns only the two
   largest buckets while the headline still reports the full total.
6. `colophon attribute` is read-only: an integration test asserts the fixture
   tree is byte-for-byte unchanged after the command runs.
7. `colophon attribute ~/wintermute/autobuilder --format text` run live on this
   machine produces a non-empty ranked report distinguishing /dream-written
   files from shell-written files (integration test, skipped with a logged note
   if provfs is absent).
