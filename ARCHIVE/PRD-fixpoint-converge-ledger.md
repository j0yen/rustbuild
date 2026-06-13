# PRD: fixpoint-converge-ledger — make the pipeline's convergence observable

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/fixpoint.md

## TL;DR

The not-current headline went 84/84 (2026-06-11) → 16/16 (2026-06-13) — a
5× change nobody noticed, because no instrument stores the count over time
or asserts it is shrinking. This PRD adds a convergence ledger: `adopt`
records `{total, behind, dirty-blocked, fallback}` per run, exposes
`adopt converge` to print the trend, and emits a `fixpoint-not-converging`
docket finding when the genuinely-behind count rises or fails to reach
zero after a configured number of runs.

## Why this exists

Phase-1 / recall evidence, 2026-06-13:

- Self-review reflective memories report a flat "N/N not-current" every
  run (84/84 on 2026-06-11, 16/16 today) with no delta, no trend, no
  "are we winning?" signal. The 5× drop passed entirely unremarked —
  whether it was real progress or a tracking-set change, nothing can say.
- The pipeline is a control loop (build → install → mark → verify) with
  **no error signal**. `apply` runs every 6h; without a convergence
  record there is no way to detect the hamster-wheel failure mode this
  vision documents (reinstall forever, never converge) except by a human
  eyeballing journal prose.
- docket already tracks findings over time (`report_count`, `runs_seen`)
  but keyed on *qualitative* findings, not a *quantitative* burndown of
  the stale count.

## What this builds

In `~/wintermute/adopt` (new `src/converge.rs` + a `Converge` subcommand
in `src/cli.rs`):

- After a `verify` (or as part of `adopt report`), append a record to a
  convergence ledger under the adopt state dir
  (`~/.local/state/adopt/converge.jsonl`, append-only, O_APPEND,
  SIGPIPE-safe): `{run, ts, total, behind, dirty_blocked, fallback,
  lineage_current}`. `behind` is the `SourceNewer-behind` count from
  fixpoint-verify-resolution.
- `adopt converge` prints the trend (last N runs: total and behind, with a
  ▲/▼/= marker per column) in table and `--format json`.
- Assert convergence: if `behind` strictly increased vs. the previous run,
  OR `behind > 0` for more than `--stall-runs <N>` (default 4)
  consecutive runs, emit/refresh a `fixpoint-not-converging` docket
  finding with the offending counts; auto-resolve it when `behind`
  reaches 0 or resumes decreasing. Reuse the existing `adopt report` →
  docket binding path; do not hand-roll docket I/O.
- Read-only by default for `adopt converge`; only the
  record-append and the docket emit mutate state, and both are idempotent
  per run (keyed on `run` id, no double-append for the same RUN_ID).

Scope guard: depends on `SourceNewer-behind` from
fixpoint-verify-resolution — if that bucket is absent, fall back to total
not-current as `behind` and note the degradation, do not fail. MSRV 1.85,
no let-chains, `sigpipe::reset()` in `main`.

## Acceptance criteria

1. Running the convergence record step appends exactly one JSONL line per
   distinct `run` id to `~/.local/state/adopt/converge.jsonl` with fields
   `{run, ts, total, behind, dirty_blocked, fallback, lineage_current}`;
   a second run with the same `run` id does not duplicate the line.
2. `adopt converge` prints the last N runs with per-column trend markers;
   `--format json` emits the same data as an array.
3. Given a synthetic ledger where `behind` increases run-over-run, the
   convergence check emits a `fixpoint-not-converging` docket finding;
   given one where `behind` reaches 0, it resolves/does-not-emit.
4. The stall rule fires after `--stall-runs` consecutive runs with
   `behind > 0` and clears when `behind` hits 0.
5. `cargo test` green (ledger fixtures held-out, not co-written with the
   assertion logic), `cargo build --release` succeeds, and the new
   subcommand appears in `adopt --help`.
