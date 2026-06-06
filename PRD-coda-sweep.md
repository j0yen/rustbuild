# PRD: coda-sweep — the summary-debt model and the plan to close it

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/coda`
**Vision:** visions/coda.md

## TL;DR

Session ctrace logs get a one-page `.summary.md` only when the SessionEnd
hook fires — and headless timer ticks are SIGKILLed before it does, so their
summaries are silently lost. There is no canonical model of *which* logs are
in debt, *how old* the debt is, or *what to do* about each. **coda-sweep**
creates the `coda` workspace and the foundation the rest of the vision
extends: the shared types (`SessionLog` / `SummaryState` / `DebtClass` /
`SweepAction` / `SweepPlan`), the `LogStore` trait that abstracts the sessions
directory, a `FakeStore` for tests, and a **pure** `sweep(logs, active_log,
now, grace)` that classifies every log and emits a print-only action plan.
`coda plan` shows it at a glance.

## Why this exists

- **The debt is real and large.** `~/.cache/ctrace/sessions/` holds **1874
  `*.ndjson` vs 1251 `*.summary.md` → 623 orphaned (33%)** as of 2026-06-05.
  Every timer tick today (`claude-20260605T210001` … `…T230000`) is missing.
- **The cause is documented.** `ctrace-scribe/README.md`: *"Heavy headless
  sessions are SIGKILLed by cgroup teardown before the hook runs, leaving logs
  permanently un-summarized."* Confirmed live: today's `…T230000.ndjson` ends
  mid-`execve` with no end marker, no summary.
- **Open docket item `ctrace-sessionend-flake`** ("ctrace session-end event
  missing", first_seen 2026-05-30, still open) is exactly this symptom.
- **There is no model of the debt.** Today self-review re-derives the gap by
  hand and runs `scribe backfill` inconsistently (50 rendered 06-03, 2 on
  06-02, 0 on 06-01 — journals). The set of logs that *should* be summarized,
  and the decision for each, lives nowhere. It must become a typed, testable
  plan before anything can act on it.
- This is the FIRST PRD of the fleet: it creates the repo + shared types + the
  `LogStore` trait. relay/concord/quicken/keel/anchor all learned the hard
  rule — **no rust-extend starts until the corpus repo has SHIPPED** or
  extend-validate fails. coda-sweep is coda's corpus.

## What this builds

New cargo workspace at `~/wintermute/coda` with a `coda` binary crate.

**The classification model.** A log is in one of these states; `sweep` is the
pure function that assigns them. The signal is *not* an in-NDJSON end marker
(graceful logs don't carry one either — the SessionEnd hook is what renders
them). The signals are: (a) is this the **active** log ctrace is currently
writing, (b) does a `.summary.md` already exist, (c) how long since the log's
last write (`mtime`-derived age, injected as `now`).

- `SummaryState` enum: `Present`, `Missing`.
- `DebtClass` enum:
  - `Active` — the live ctrace log; never touch (excluded by path match).
  - `Fresh { age_secs }` — missing summary but younger than `grace`; the
    SessionEnd hook may still fire, so skip for now.
  - `Orphaned { age_secs }` — missing summary, older than `grace`; real debt,
    render it.
  - `Settled` — summary already present; nothing to do.
- `SessionLog { path: PathBuf, summary: SummaryState, age_secs: u64, is_active: bool }`
  — one log as seen through the store.
- `SweepAction` enum: `Render { path }`, `Skip { path, reason }`,
  `NoOp { path }`.
- `SweepPlan { actions: Vec<SweepAction>, orphaned: usize, fresh: usize,
  settled: usize, total: usize }` — the diff result; **declarative, no side
  effects** at this layer. `Render` actions are what coda-close will execute.

**The `LogStore` trait** — abstracts the sessions dir so `sweep` is pure and
testable, and so coda-audit/coda-close share one contract:

```rust
pub trait LogStore {
    fn logs(&self) -> Result<Vec<RawLog>>;          // (path, has_summary, mtime) per *.ndjson
    fn render(&self, path: &Path) -> Result<()>;    // shell `scribe render` (apply-only; unused here)
}
```

`RawLog { path, has_summary, mtime_secs }` is the store's raw observation;
`sweep` turns `&[RawLog]` + the active-log path + injected `now` + `grace`
into typed `SessionLog`s and a `SweepPlan`. coda-sweep ships a `FakeStore`
(in-memory fixtures) for tests; the real `FsStore` lands in coda-audit.

**The grace knob** — `~/.config/coda/coda.toml`, optional:

```toml
grace_secs = 120        # younger-than-this missing-summary logs are Fresh, not Orphaned
sessions_dir = "/home/jsy/.cache/ctrace/sessions"
```

`CodaConfig::load(path)` parses it; defaults (`grace_secs = 120`,
`sessions_dir = ~/.cache/ctrace/sessions`) ship in `config/coda.example.toml`.
Loading is pure — no filesystem scan, no `scribe` call.

**UX:** `coda plan` → table of `log | state | age | action`; `--format json`
for machines. Print-only — no `--apply` in this PRD (that's coda-close). Exit
non-zero when at least one log is `Orphaned` (so a hook can gate on it before
`--apply` exists).

**Deps:** minimal — `serde`/`serde_json`, `toml`, `clap`. MSRV 1.85, no
let-chains (`self_recall_baseline_gate_red` discipline). `sigpipe::reset()` as
the first line of `main()` (`self_sigpipe_panic_toolkit` — `coda plan | head`
must not coredump). Match the workspace shape of sibling toolkit repos
(vigil/quicken/keel/anchor): `clippy.toml`, `deny.toml`, `rust-toolchain.toml`,
`CHANGELOG.md`, `README.md`.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed offline; a test asserts `sweep`
   makes **zero** store calls (it operates only on the passed `&[RawLog]` +
   the active-log path + injected `now`/`grace`).
2. `SessionLog`, `SummaryState`, `DebtClass`, `SweepAction`, `SweepPlan` are
   public and `serde`-(de)serializable; a round-trip test covers each.
3. `CodaConfig::load` parses `config/coda.example.toml`; a fixture manifest
   yields the expected `grace_secs` + `sessions_dir`, and an absent file
   yields the documented defaults.
4. `sweep` classifies correctly against `FakeStore` fixtures: the active-log
   path → `Active` + `NoOp` (even if summary missing); a missing-summary log
   younger than `grace` → `Fresh` + `Skip`; a missing-summary log older than
   `grace` → `Orphaned` + `Render`; a log with a summary → `Settled` + `NoOp`.
5. `coda plan --format json` emits one entry per log plus the `SweepPlan`
   tallies (`orphaned`/`fresh`/`settled`/`total`); the schema matches the
   documented `SweepPlan`.
6. `coda plan` exits non-zero when ≥1 log is `Orphaned`, zero when none are;
   covered by two integration cases driving a `FakeStore`.
7. `coda plan | head -1` does not panic (SIGPIPE reset verified by a test that
   closes the read end early).
8. README documents the config format, the type surface, and the `LogStore`
   trait so coda-audit / coda-close / coda-boot have a contract to extend.
