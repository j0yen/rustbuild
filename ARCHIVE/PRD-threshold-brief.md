# PRD: threshold-brief — one synthesized arrival briefing, not ten raw dumps

Status: Draft v0.1
build_target: rust-cli
Vision: visions/threshold.md

## TL;DR

A fresh Claude session opens to ~21 KB of unsynthesized text from ten
independent SessionStart hooks. This PRD builds `threshold`, a new Rust CLI whose
`threshold brief` gathers those scattered signals and emits ONE prioritized
arrival briefing — text for a human, `--format json` for a machine. It is the
foundation crate the rest of the `threshold` fleet extends.

## Why this exists

Phase-1 inspection, 2026-06-18:

- The SessionStart hook bundle for this session measured **20,888 bytes**
  (`wc -c` on the captured hook stdout). It is a firehose: raw recall hits, a
  raw agorabus 10-peer list, a raw self-review docket dump, a lint-failed
  CLAUDE_SELF header, an agentns-activation-blocked banner — none synthesized,
  none prioritized.
- **Ten** SessionStart hooks are wired (`jq '.hooks.SessionStart' settings.json`:
  check-review-due, ctrace-session-start, recall-session-start,
  scratch-tools-start, claude-self-start, agorabus-session-start,
  learning-candidates-start, agentns-activation-check, roundtable-sessionstart,
  peon). Each emits independently; nothing consolidates them.
- No arrival/briefing/orientation tool exists: `ls ~/.local/bin | grep -iE
  'brief|arriv|orient|welcom|onboard|sitrep'` → none; no matching repo in
  `~/wintermute/`.
- Adjacent tools serve the *ending* session (`coda` summary-debt, `scribe`
  backfill, `ember`) or the *structural/cross-node* self (`corpus-introspect`,
  `cogito`) — none serve the *beginning* session's orientation.
- The signals a briefing needs already exist as queryable sources: `recall list
  --kind reflective`, `~/wintermute/autobuilder/notes/gossip.md`, the build
  manifest (`~/.claude/skills/build/state/manifest.json`), `git status` across
  `~/wintermute/*`, `docket` open findings, the self-review-due flag. The work is
  synthesis, not new data collection.

## What this builds

New repo `~/wintermute/threshold/` (Rust workspace, MSRV 1.85, edition 2021;
follows the autobuilder scaffold incl. `sigpipe::reset()` first line of `main`
per the standing SIGPIPE-panic lesson).

- **Model:** a `Signal` enum/struct (kind, title, body, priority, source,
  freshness) and a `Briefing` (ordered, sectioned: *Mid-flight*, *Owed to you*,
  *Changed since last session*, *Don't redo*). Pure data.
- **`SignalSource` trait** with concrete sources behind it:
  `RecallSource` (reflective tail), `GossipSource` (gossip.md tail),
  `BuildManifestSource` (in-flight / blocked PRDs), `GitSource` (dirty +
  unpushed under `~/wintermute/*`), `DocketSource` (open findings),
  `ReviewDueSource` (self-review flag). Each source is individually fallible and
  degrades to empty on error (a missing source must never sink the brief).
- **`FakeSource`** + fixtures for deterministic tests.
- **Pure synthesizer:** `fn synthesize(signals: Vec<Signal>) -> Briefing` —
  dedup, prioritize, section, cap length. No I/O; fully unit-testable.
- **`threshold brief`** subcommand: gather → synthesize → render. `--format
  text|json`, `--max-items N`, `--source-root <path>` for testing.
- README + install script (`~/.local/bin/threshold`).

Out of scope (later PRDs): claim verification (threshold-verify), the
ask/answer ledger (threshold-ledger), hook wiring (threshold-hook).

## Acceptance criteria

1. `cargo build` and `cargo test` are green; `cargo clippy` adds no new warnings
   over the autobuilder baseline; binary installs to `~/.local/bin/threshold`.
2. `threshold brief --format json` against a `FakeSource` fixture set produces a
   `Briefing` with the documented sections and a stable, documented JSON schema
   (validated by a test).
3. The synthesizer is a pure function (no I/O in its signature) and is covered by
   unit tests for: dedup of identical signals, priority ordering, section
   assignment, and the `--max-items` cap.
4. Each real `SignalSource` degrades to an empty contribution (not an error and
   not a panic) when its backing data is missing or malformed — proven by a test
   that points a source at a nonexistent path.
5. `threshold brief` (text mode) run live on this box produces a non-empty,
   sectioned briefing under a documented size budget (target: ≤ 4 KB, i.e. < 1/5
   of today's 20,888-byte firehose) and completes in < 500 ms cold.
6. First line of `main()` is `sigpipe::reset()`; `threshold brief | head` does
   not panic (regression guard for the known SIGPIPE-panic class).
7. README documents every source, the JSON schema, and the `--source-root`
   testing seam.
