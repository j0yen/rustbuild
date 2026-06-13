# PRD: plumb-coverage — report which self-review B.5 probes have no oracle

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/plumb
Vision: visions/plumb.md

## TL;DR

plumb's end-state is that *every verdict-expressible self-review B.5
probe* has a registered independent oracle. Today `plumb list` returns 3
probes while self-review B.5 has many playbooks, and nothing reports the
gap — so "are all the watchers calibrated?" is answered by a human
reading SKILL.md against the registry each pass, which is exactly the
manual audit plumb exists to retire. This PRD adds `plumb coverage`: it
parses the B.5 playbook IDs and the `plumb_gate` probe→playbook table out
of the self-review SKILL.md, cross-references them against registered
probe IDs, and reports `registered` / `unregistered` / `orphan` so the
end-state becomes a measurable "N of M".

## Why this exists (Phase 1 evidence, 2026-06-13)

- `plumb list` (run live this session) returns exactly 3 probes:
  `memlog-active`, `ctrace-backfill-wired`, `adopt-report-exists`.
- `~/.claude/skills/self-review/SKILL.md` (read live) defines many B.5
  playbooks beyond those three — e.g. `memlog_group_awaiting_activation`,
  `ctrace_sessionend_resolve`, `ctrace_scribe_backfill`,
  `fleet-binary-staleness`, the warden/bpolicy inert playbook, the
  agorabus daemon-staleness playbook, the autobuilder-gate-promote
  playbook. The `plumb_gate` sub-step (SKILL.md:320) carries an explicit
  probe→playbook table that currently lists only the 3 registered probes.
- The vision's end-state (`visions/plumb.md`) states it plainly: "Every
  self-review B.5 probe that can be expressed as a verdict has a
  registered independent oracle." There is no command that measures how
  far from that we are, so coverage is tracked by hand — the same
  hand-audit that let the memlog and ctrace probes carry false readings
  for 9–13 runs (per 2026-06-09…06-13 journals).
- This pairs with [[feedback_verify_before_concluding]]: knowing which
  detectors are *unverified* is a precondition for not trusting their
  readings.

## What this builds

A new `coverage` subcommand on the existing `plumb` binary.

- New module `src/coverage.rs`:
  - `fn playbook_ids(skill_md: &str) -> BTreeSet<String>` — extract B.5
    playbook identifiers. Source of truth (drafted): the `plumb_gate`
    probe→playbook markdown table (`| probe-id | playbook |`) plus the
    `### Sub-step:` / playbook-id headings in Phase B.5. Parse defensively
    — unknown formats are skipped, never panic.
  - `fn classify_coverage(playbooks, gate_table, registered) -> Coverage`
    producing three lists:
    - `registered` — probe IDs present in `probes.toml` AND referenced by
      a playbook/gate entry.
    - `unregistered` — playbooks referenced by the gate table (or flagged
      verdict-expressible) with no matching registered probe.
    - `orphan` — registered probe IDs with no playbook reference.
- New CLI command `plumb coverage`:
  - Resolves the SKILL.md path: `--skill <path>` override, else default
    `~/.claude/skills/self-review/SKILL.md`.
  - `--format json` → `{registered:[…], unregistered:[…], orphan:[…],
    counts:{registered:N, unregistered:N, orphan:N, total_playbooks:M}}`.
  - Default human output: a summary line `coverage: N/M B.5 detectors
    calibrated` followed by the unregistered and orphan lists.
  - Guard: if the SKILL.md file is absent, emit a clear one-line message
    and exit nonzero (1) — never panic, never report false 100%.
  - SIGPIPE-safe (consistent with the rest of the toolkit,
    [[self_sigpipe_panic_toolkit]]).

Reuses `Registry` for the registered-probe set. No new runtime deps
(string parsing only; `toml`/`serde` already present).

## Acceptance criteria

1. `plumb coverage` against the live self-review SKILL.md and seed
   `probes.toml` lists `memlog-active` and `adopt-report-exists` under
   `registered` (both appear in the `plumb_gate` table) and reports a
   `coverage: N/M` summary line with M ≥ 3.
2. At least one known B.5 playbook that has no registered probe (e.g.
   `fleet-binary-staleness` or the agorabus daemon-staleness playbook)
   appears in the `unregistered` list.
3. `plumb coverage --format json` emits valid JSON with `registered`,
   `unregistered`, `orphan` arrays and a `counts` object whose
   `registered + ...` values match the array lengths.
4. `plumb coverage --skill /nonexistent/path` exits nonzero with a clear
   "skill file not found" message and does NOT report any coverage
   percentage (no false 100%).
5. A registered probe ID that no playbook references is reported under
   `orphan` (constructed test with a temp probes.toml + temp SKILL.md
   fixture).
6. `playbook_ids` unit test: given a SKILL.md fragment containing the
   `| probe-id | playbook |` table and a couple of `### Sub-step:`
   headings, it returns the expected identifier set and skips non-playbook
   headings.
7. `cargo test` green and `cargo build --release` clean on MSRV 1.85
   (edition 2021, no let-chains). `plumb coverage --help` documents the
   `--skill` and `--format` flags.
