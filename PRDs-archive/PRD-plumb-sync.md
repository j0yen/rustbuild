# PRD: plumb-sync — detect when a registered verdict drifts from its live probe

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/plumb
Vision: visions/plumb.md

## TL;DR

Every verdict command in `~/.config/plumb/probes.toml` is a hand-copy of
a probe that lives in self-review's SKILL.md — the config comments say so
("The BROKEN probe from SKILL.md:186"). When plumb-selfreview-bind later
*fixes* that live probe, the registered verdict still holds the old
broken form, so plumb starts calibrating stale logic and its agreement
verdict becomes meaningless — a calibrator silently drifting from the
thing it claims to measure. This PRD adds an optional `source` pointer to
each probe and a `plumb sync` command that reads the live probe text at
that pointer and reports whether the registered verdict still matches it.

## Why this exists (Phase 1 evidence, 2026-06-13)

- The live seed `~/.config/plumb/probes.toml` (read this session) embeds
  copies of SKILL.md probe logic, annotated with the source line:
  `# The BROKEN probe from ~/.claude/skills/self-review/SKILL.md:186`
  and `# Original probe from SKILL.md:644 — direct grep`.
- plumb-selfreview-bind (shipped 2026-06-13, per the build changelog in
  CLAUDE_SELF.md: "plumb_gate in self-review Phase B.5, memlog probe fix")
  is mandated by the plumb vision to *fix the live memlog probe at
  SKILL.md:186 as the first proof*. The moment that fix lands, the
  registered `memlog-active` verdict — a verbatim copy of the broken
  form — no longer mirrors the live probe. plumb would then keep checking
  the obsolete broken command against its oracle and report agreement or
  disagreement about logic that no longer runs anywhere.
- Nothing in plumb today ties a registered verdict back to its origin, so
  this drift is undetectable. This is the freshness sibling of
  [[feedback_letter_vs_stack]] / [[feedback_verify_before_concluding]]:
  a stored description of state that was true once and is trusted as
  still-true without re-checking the live source.

## What this builds

An optional provenance field plus a `sync` subcommand on `plumb`.

- Extend `ProbeEntry` in `src/registry.rs` with an optional field
  `source: Option<String>` — a pointer of the form `<file>:<line>` or
  `<file>:#<anchor>`, where `<anchor>` is a literal comment marker
  expected to appear in the source file (e.g. `# plumb-probe:memlog-active`).
  Backward compatible: existing entries without `source` parse fine and
  are reported as `source-missing` by `sync`.
- New module `src/sync.rs`:
  - `fn extract_live(source: &str) -> Result<Option<String>>` — resolve
    the file, then for `:#anchor` read the shell command block immediately
    following the anchor comment line (until the next blank line or
    non-continuation line); for `:line` read the command starting at that
    1-based line (honoring `\`-continuation and `'''`-style multiline
    blocks as they appear in SKILL.md probe snippets). Missing file or
    anchor → `Ok(None)`.
  - `fn normalize_cmd(s: &str) -> String` — collapse whitespace, strip
    trailing comments, trim, so cosmetic reflows aren't reported as drift.
  - `fn compare(registered_verdict: &str, live: Option<&str>) -> SyncState`
    where `SyncState ∈ { InSync, Drifted{registered, live}, SourceMissing,
    NoSourcePointer }`.
- New CLI command `plumb sync`:
  - `plumb sync` / `--all` — check every probe with a `source`.
  - `plumb sync <probe-id>` — check one.
  - `--format json` → array of `{id, state, registered?, live?}`.
  - Default human output: per-probe `in-sync` / `DRIFTED` / `source-missing`
    / `no-source`, and for drift a unified before/after of the two
    normalized commands.
  - Exit nonzero (2) when any checked probe is `Drifted`; 0 otherwise
    (a `no-source`/`source-missing` probe is reported but not a hard
    failure on its own — drift is the failure). SIGPIPE-safe.
- Add `source` pointers to the three seed probes in the committed
  `probes.toml` template shipped with this build (anchor form preferred),
  and — where this PRD's build can do so safely — add the matching
  anchor comments next to the live SKILL.md probes. If editing SKILL.md
  is out of scope for the build, the PRD still ships with `:line`
  pointers and documents the follow-on to add durable anchors.

## Acceptance criteria

1. `ProbeEntry` parses a `probes.toml` entry both with and without a
   `source` field; an entry lacking `source` is reported by `plumb sync`
   as `no-source` (unit test on `Registry::load`).
2. Given a temp source file containing a known command at a known line
   and a probes.toml whose registered verdict matches it, `plumb sync
   <id>` reports `in-sync` and exits zero.
3. Given the same setup but with the source file's command edited to
   differ, `plumb sync <id>` reports `DRIFTED`, prints both normalized
   forms, and exits nonzero.
4. `plumb sync <id>` with a `source` pointing at a missing file or a
   missing `#anchor` reports `source-missing` (not a panic, not `in-sync`).
5. `normalize_cmd` makes a purely cosmetic reflow (added line-continuation,
   extra spaces, trailing comment) compare `in-sync`, while a genuine
   token change compares `DRIFTED` (unit test).
6. `plumb sync --all --format json` emits valid JSON with one object per
   probe carrying at least `id` and `state`.
7. `cargo test` green and `cargo build --release` clean on MSRV 1.85
   (edition 2021, no let-chains). `plumb sync --help` documents the flags,
   and `plumb check`/`trust`/`list` behavior is unchanged (regression:
   their existing tests still pass).
