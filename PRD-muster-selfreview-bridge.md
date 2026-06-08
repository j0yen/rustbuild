# PRD: muster-selfreview-bridge

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/muster
Vision: visions/muster.md

## TL;DR

The whole reason `muster` exists is that self-review keeps guessing at the live
session population. This PRD closes the loop: a `muster verdict --format
selfreview` emit and the self-review playbook edit that swaps the
`pgrep -af claude` + "may both be real" step for a definitive `muster` census.
After this lands, the journal prints a roster with verdicts, not a shrug.

## Why this exists

self-review's session check is `pgrep -af claude` with *"note duplicates, do not
kill"* (`self-review/SKILL.md:70`), and the resulting journal entries park
*"duplicate Claude sessions … may both be real"* under "Pending your call" on
both 2026-06-06 and 2026-06-07. census + verdict produce the answer self-review
needs; without wiring them in, the playbook keeps doing the ad-hoc `pgrep`
parse and the chronic non-finding persists. This PRD is the integration that
makes the prior three pay off in the one consumer that motivated them.

## What this builds

Two coordinated pieces (hence `mixed`):

- **`--format selfreview` emit** on `muster verdict` (extends the `muster`
  crate, `~/wintermute/muster`): a compact, deterministic block suitable for
  pasting into the journal's "Active Claude processes" line — one row per
  session (`role · origin · pid · uptime · verdict`), a one-line summary
  (`N sessions: a live, b duplicate, c orphan, d stale`), and, when any
  `orphan`/`stale` exist, the exact `muster reap` dry-run command to inspect
  them (never auto-run). Stable ordering so the journal diff is meaningful run
  to run.
- **self-review playbook edit**: replace the Phase that currently runs
  `pgrep -af claude` (SKILL.md ~line 70) with a step that runs
  `muster verdict --format selfreview` (falling back to the old `pgrep` path
  if the `muster` binary is absent, so the review never hard-fails on a machine
  without it). The "Active Claude processes" snapshot line and the
  "duplicate Claude sessions" Pending item are sourced from muster's verdict —
  duplicates/orphans are reported with muster's evidence, and the playbook keeps
  its existing "surface, do not auto-kill" discipline (reap stays a separate,
  manually-confirmed step).

The edit must preserve self-review's single-run gate and Phase structure; it
changes only the session-enumeration step's *source of truth*, not the
review's control flow.

Out of scope: auto-running `muster reap` (it stays manual/`--confirm`); any
change to reap's safety invariants.

## Acceptance criteria

1. `muster verdict --format selfreview` emits a deterministic block: one row per
   session (`role · origin · pid · uptime · verdict`) + a one-line count
   summary; re-running with an unchanged roster yields byte-identical output.
2. When ≥1 `orphan`/`stale` entry exists, the block includes the exact
   `muster reap` (dry-run) command to inspect them; when none exist, it does
   not, and the summary reads `… 0 orphan, 0 stale`.
3. The self-review SKILL's session-enumeration step invokes
   `muster verdict --format selfreview` and uses its output for the
   "Active Claude processes" snapshot and the duplicate/orphan Pending items.
4. If the `muster` binary is not on `PATH`, the self-review step falls back to
   the prior `pgrep -af claude` behavior and the review completes (no hard
   failure on a muster-less machine).
5. The playbook edit preserves the single-run gate and does not introduce any
   auto-kill; reaping remains a separate, `--confirm`-gated action a human runs.
6. A self-review dry-run (or a documented manual invocation) shows the journal's
   "Active Claude processes" line populated from muster, with verdicts, instead
   of a bare `pgrep` list.
7. `cargo test` green for the `selfreview` formatter (determinism + the
   has-orphans/has-none branches); rustc 1.85, no let-chains; census + verdict +
   reap tests still pass.
