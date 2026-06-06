# PRD: coda-audit — the live read of summary debt

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/coda`
**Vision:** visions/coda.md

## TL;DR

coda-sweep gives a pure classifier and a `FakeStore`, but nothing yet looks at
the *real* sessions directory. **coda-audit** adds the live read: a real
`FsStore` over `~/.cache/ctrace/sessions/`, an active-log resolver that reads
`ctrace status`, and a `coda audit` command that scans, classifies via
`sweep`, and reports the summary debt as a human table and `--format json`.
It is strictly **read-only** — it observes and counts; it never renders a
summary (that is coda-close). This is the surface self-review reads instead of
re-deriving the gap by hand each run.

## Why this exists

- **The gap is re-derived by hand today.** Each self-review counts orphaned
  logs and decides whether to run `scribe backfill` ad hoc — rendering 50 on
  06-03, 2 on 06-02, 0 on 06-01 (journals). A single read-only `coda audit`
  replaces that toil with one deterministic number.
- **The active log must be excluded correctly.** The 06-01 journal notes *"the
  only summary-less ndjson is the active session log, correctly excluded"* —
  the live ctrace log (`ctrace status` → `.log`, observed 2026-06-05:
  `claude-20260605T230000.ndjson`, still being written) must never count as
  debt. coda-audit resolves it from `ctrace status` rather than guessing by
  mtime.
- **The debt is large enough to need a real reader.** 623 orphaned logs over
  1874 total (2026-06-05) — a `coda audit` that streams the directory once and
  reports the tally is the legible entry point before any repair runs.
- coda-sweep has shipped: its `LogStore` trait + `SweepPlan` are the contract
  this PRD implements against (the rust-extend rule — do not start until the
  corpus repo exists).

## What this builds

Extends the `coda` crate; no new repo.

**`FsStore` — the real `LogStore`.** Walks `sessions_dir` for `*.ndjson`,
records for each: path, whether the sibling `*.summary.md` exists, and the
`mtime` (seconds). One directory pass; no per-file content read (the
has-summary check is a stat on the sibling path, the age is the ndjson mtime).
Honors `CodaConfig.sessions_dir`.

**Active-log resolver.** Shells `ctrace status` (JSON), parses `.log`, and
passes that path to `sweep` as the active log so it is classed `Active`/`NoOp`.
If `ctrace status` is unavailable or not running (no active session), the
resolver returns `None` and every closed log is eligible — a fixture covers
the no-active-session case so coda-audit works in headless/cron contexts.

**`coda audit`** — the read command:

- scans via `FsStore`, resolves the active log, computes `now` from the system
  clock *once* at the call boundary (passed into the pure `sweep`), classifies.
- Human output: a tally line (`N orphaned, M fresh, K settled / T total`) plus,
  with `--verbose`, the per-log table; oldest-orphan age called out.
- `--format json`: the full `SweepPlan` + per-log `SessionLog` list.
- `--orphaned-only`: print just the orphaned log paths (one per line), the
  feed coda-close consumes.
- Exit non-zero when ≥1 `Orphaned` log exists (same gate contract as `coda
  plan`), zero otherwise.

**Strict read-only guarantee.** coda-audit calls only `LogStore::logs` and the
active-log resolver — never `LogStore::render`. An integration test asserts a
`render`-counting store records **zero** render calls across a full `coda
audit` run.

**Deps:** reuse coda-sweep's; add nothing beyond what shelling `ctrace status`
needs (`std::process::Command`). MSRV 1.85, no let-chains. `sigpipe::reset()`
already in `main` (`coda audit --orphaned-only | head` must not coredump).

## Acceptance criteria

1. `cargo build` / `cargo test` succeed offline; the suite includes a
   `tests/audit.rs` entry file that appears in cargo test output
   (`self_orphaned_mock_tests` guard).
2. `FsStore::logs` over a temp fixture dir (a mix of `*.ndjson` with and
   without sibling `*.summary.md`, varied mtimes) returns the expected
   `RawLog`s — correct `has_summary` and `mtime_secs` per file.
3. The active-log resolver parses a `ctrace status` JSON fixture and yields the
   `.log` path; a malformed / not-running fixture yields `None` without error.
4. `coda audit` over a fixture dir excludes the resolver-reported active log
   from debt (classed `Active`) even though it has no summary; remaining
   missing-summary logs older than `grace` count as `Orphaned`.
5. `coda audit --format json` emits the `SweepPlan` tallies + per-log list
   matching coda-sweep's schema; `--orphaned-only` prints exactly the orphaned
   paths, one per line, and nothing else.
6. **Read-only:** an integration test drives `coda audit` against a store whose
   `render` increments a counter and asserts the counter is **0**.
7. `coda audit` exits non-zero with ≥1 orphaned log, zero with none; two
   integration cases cover both.
8. `coda audit --orphaned-only | head -1` does not panic (SIGPIPE).
9. **[deferred — laptop-only]** `coda audit` run against the real
   `~/.cache/ctrace/sessions/` reports an orphan count matching `ls *.ndjson`
   minus `ls *.summary.md` minus the active log. (deferred_acs:[9] — the cloud
   build box has no ctrace session history.)
