# PRD: litmus-stuck-detector — `docket stuck` flags probe-suspect findings

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/docket
Vision: visions/litmus.md

## TL;DR

When a `docket` finding is reported run after run and never resolves,
there are two possibilities: the world is genuinely stuck, or the *probe*
that reports it is lying. Today nothing distinguishes them — a
false-by-construction probe (see the ctrace case below) accumulates
reports silently, and the suspicion "maybe the probe is wrong" lives only
in a human eventually noticing a pattern across journal entries. This PRD
adds `docket stuck`: a read-only subcommand that surfaces open findings
whose report/run counts cross a threshold without ever resolving, labeled
as **probe-suspect — audit the probe, not the world.** It converts a
two-week-latency human catch into a one-command-per-review signal.

## Why this exists (Phase 1 evidence, 2026-06-13)

- `docket show ctrace-sessionend-flake` (measured live) →
  `report_count: 12`, `runs_seen: 10`, open from `2026-05-30` until
  resolved today. Its resolution reason: *"grep pattern 'scribe backfill'
  is a false negative — hook is correct."* The finding was probe-suspect,
  not world-stuck, for the entire fortnight — and nothing said so.
- `docket list` already tracks the exact fields needed:
  `src/model.rs:118` `struct Finding` carries `report_count` (`:140`),
  `runs_seen` (`:136`), `consecutive_runs` (`:138`), `resolved_at`
  (`:142`), `resolve_reason` (`:144`). No schema change is required to
  compute "reported a lot, never resolved."
- The CLI surface is a clean `enum Command` at `src/cli.rs:52` with
  `Report/List/Show/Resolve/Ack/Digest/Sweep`; `Digest`
  (`src/digest.rs`) is the model for a read-only rollup that other tools
  consume. `stuck` slots in beside them.
- Multiple self-reviews independently re-flag the same handful of
  long-lived findings (`agentns-session-zeros` runs=13+,
  `adopt-scan-stale-binaries` 84→16 churn, the now-resolved ctrace one) —
  evidence that "old open finding that keeps getting re-reported" is a
  recurring, detectable shape worth a first-class query.

## What this builds

A new `Stuck` variant on `docket`'s `Command` enum and a `db::stuck`
query. Pure read; no migration, no mutation. MSRV 1.85, no let-chains,
`sigpipe::reset()` already in `main`.

- **`docket stuck`** — list open (and escalated) findings that are
  *probe-suspect*: reported many times, never resolved, no recent
  resolution progress. Default criteria (tunable via flags):
  `report_count >= --min-reports` (default 6) **and**
  `runs_seen >= --min-runs` (default 5) **and** `resolved_at is null`.
- **Flags:** `--min-reports N` (default 6), `--min-runs N` (default 5),
  `--format text|json` (default text), `--key <substr>` to filter. JSON
  emits an array of `{key, title, report_count, runs_seen,
  consecutive_runs, first_seen, last_seen, suspect_reason}` so the
  self-review bind PRD can parse it.
- **`suspect_reason`** is a short human string, e.g.
  `"reported 12× over 10 runs, never resolved — verify the probe matches
  reality before re-parking"`. It does not assert the probe IS wrong —
  only that the shape warrants an audit.
- **Ack-aware.** Findings already `ack`-ed (triaged/known-inert, e.g.
  `memlog`, `warden`) are excluded by default — an acked finding has
  already been judged, so it is not "suspect." A `--include-acked` flag
  overrides for completeness audits.
- **Read-only and ledger-safe.** `docket stuck` never writes the db;
  running it has no effect on streaks, runs, or resolution state.
- **Empty is success.** Zero probe-suspect findings prints a single
  `no probe-suspect findings` line and exits 0 (so a self-review banner
  reads clean when the field is healthy).

## Acceptance criteria

1. `docket stuck` lists every open finding with
   `report_count >= 6 AND runs_seen >= 5 AND resolved_at IS NULL`, and
   excludes findings below either threshold and any resolved finding.
   Tested against a temp `XDG_DATA_HOME` db seeded via repeated `docket
   report` (the `docket-bind-selftest.sh` isolation pattern).
2. A finding matching the historical ctrace shape (reported 12× across 10
   runs, unresolved) appears in `docket stuck` output with a
   `suspect_reason` naming the counts; once `docket resolve`d, a
   subsequent `docket stuck` omits it. End-to-end test.
3. `--min-reports` and `--min-runs` flags change the cutoff: a finding at
   `report_count 6 / runs_seen 5` appears at defaults, is excluded at
   `--min-reports 7`, and is excluded at `--min-runs 6`. Test covers both
   boundaries.
4. `--format json` emits a valid JSON array with the documented fields;
   `jq -e 'length'` parses it and the field set matches the spec. Text
   format is human-readable with one finding per stanza.
5. Acked findings are excluded by default and included under
   `--include-acked`. Test seeds an acked finding above threshold and
   asserts both behaviours.
6. `docket stuck` performs no writes: a byte/identical check of the db
   file (or a `report_count`/`runs_seen` re-read) before and after
   confirms the ledger is unchanged. Empty result prints
   `no probe-suspect findings` and exits 0.
7. `cargo build` clean, `cargo test` green, all new tests hermetic (temp
   XDG + in-process db, no live ledger, no network). Existing docket
   tests still pass.
