# PRD: abide-ack-state — docket findings get an acknowledged lifecycle

Status: shipped
build_target: rust-extend
build_into: /home/jsy/wintermute/docket
output_repo_path: /home/jsy/wintermute/docket
output_repo_url: https://github.com/j0yen/docket
Vision: visions/abide.md

## TL;DR

`docket` has three lifecycle states — `Open`, `Escalated`, `Resolved`
(`src/model.rs:50`) — and no way to say *"I have seen this finding, I have
decided to carry it, stop surfacing it until it changes."* Self-review uses
exactly that semantic in prose every run ("acked — will not re-escalate until
pkgrel changes"), but the ledger cannot store it, so two human-triaged findings
(`memlog-activation`, `warden-enforcer-inert`) keep showing as `[open] warn` in
every digest and banner. This PRD adds an **acknowledged** flag — orthogonal to
the recurrence `Status` — that keeps a finding fully tracked but marks it as
carried, and that **auto-clears the moment its evidence fingerprint changes** so
an ack can never hide a real state change.

## Why this exists

Phase 1 evidence (2026-06-08, this `/dream` pass — see `visions/abide.md`):

- `docket --help` (live) lists `report/list/show/resolve/digest/sweep` — **no
  `ack`**. `src/model.rs:50` confirms `enum Status { Open, Escalated, Resolved }`.
- `docket list --open` (live) shows `memlog-activation` (runs_seen:3,
  consecutive:2) and `warden-enforcer-inert` (runs_seen:3, consecutive:2) both
  `[open] warn`, both flagged "acked" in the 2026-06-08 self-review prose
  (`recall 01KTK2KF17SQYDZ4MZ3PQCK5T1`, items (4) and (5)).
- The acknowledgement is durable in the human's head and re-typed as prose
  every run (2026-06-06, -07, -08 journals all carry it). It belongs in the
  ledger that already exists for exactly this "findings get a memory" purpose
  (`visions/docket.md`).
- The carry-condition is already measured each run: memlog's is the installed
  pkgrel, warden's is the bpolicy `loaded` bool. The change-signal an ack should
  resurface on exists; it just has nowhere to live.

## What this builds

`rust-extend` into `~/wintermute/docket` (modules `model.rs`, `db.rs`,
`cli.rs`):

- **Schema (additive, idempotent migration in `db.rs`).** Four nullable columns
  on the `findings` table: `acknowledged_at TEXT`, `ack_reason TEXT`,
  `ack_fingerprint TEXT`, `ack_until_run TEXT`. Migration runs on open, is a
  no-op if the columns already exist (mirror the existing column-add pattern;
  `PRAGMA table_info` guard). **No `Status` variant is added** — ack is
  orthogonal to `Open`/`Escalated`/`Resolved`, so streak and escalation logic
  are untouched.
- **`Finding` struct (`model.rs`).** Add the four fields as
  `Option<String>`/`Option<i64>` with `#[serde(skip_serializing_if =
  "Option::is_none")]`. Add a `#[must_use] pub fn is_acked(&self) -> bool`
  (`acknowledged_at.is_some()`).
- **`docket ack <key>` subcommand.** Flags: `--reason <TEXT>` (optional),
  `--until-change <FINGERPRINT>` (optional opaque string), `--runs <N>`
  (optional; records `ack_until_run` = current run-id + N). Sets
  `acknowledged_at` to now (RFC3339), stores reason/fingerprint. Errors with a
  clear message if `<key>` does not exist or is already `Resolved` (acking a
  resolved finding is meaningless). Idempotent: re-acking updates
  reason/fingerprint without resetting `acknowledged_at` unless `--reset`.
- **`docket unack <key>` subcommand.** Clears all four columns; no other data
  touched. No-op (exit 0, note printed) if the finding was not acked.
- **Auto-clear in `report`'s upsert path (`db.rs`).** When a `report` upserts a
  finding that is currently acked: if the incoming evidence fingerprint differs
  from the stored `ack_fingerprint` (string inequality; a `None` stored
  fingerprint means "carry regardless" and never auto-clears on fingerprint),
  **OR** the current run crosses `ack_until_run`, clear the ack columns so the
  finding resurfaces. Same fingerprint and within run-window → ack persists.
  The fingerprint arrives via a new optional `--fingerprint <FP>` on `report`
  (absent → fingerprint comparison skipped, only `ack_until_run` can clear).
- **JSON + text output.** `docket show`/`list` include the ack fields when
  present; `format_text` (`model.rs:170`) renders an `acked:` line (reason +
  fingerprint + until-run) when `is_acked()`.

Deps: no new crates (rusqlite, serde, clap already in tree). MSRV 1.85, no
let-chains (per `self_recall_baseline_gate_red` house rules).

## Acceptance criteria

1. **Migration is additive and idempotent.** Opening a pre-existing docket DB
   adds the four columns; opening it again is a no-op. A `cargo test` covering
   open→reopen on a temp DB passes; no existing column or row is altered.
2. **`docket ack <key> --reason R --until-change FP` sets the columns.**
   `docket show <key> --json` then reports `acknowledged_at` (non-null),
   `ack_reason == R`, `ack_fingerprint == FP`. The finding's `status`,
   `runs_seen`, and `consecutive_runs` are **unchanged** by the ack.
3. **`docket ack` errors on unknown or resolved key** with a non-zero exit and a
   message naming the key.
4. **`docket unack <key>` clears all four ack columns** and leaves every other
   field intact; unack of a non-acked finding exits 0 with a note.
5. **Report with the same fingerprint keeps the ack.** Ack a finding with
   `--until-change pkgrel:5`, then `docket report <key> ... --fingerprint
   pkgrel:5`: `is_acked()` remains true.
6. **Report with a changed fingerprint clears the ack.** After the same setup,
   `docket report <key> ... --fingerprint pkgrel:11` clears the ack columns and
   the finding is `is_acked() == false`; `runs_seen` still increments normally.
7. **`ack_until_run` crossing clears the ack** even when the fingerprint is
   unchanged (covered by a test that advances the run-id past the recorded
   `ack_until_run`).
8. **Existing docket test suite stays green** (`cargo test` in
   `~/wintermute/docket` — confirm `Running` lines for all existing test files,
   per `self_orphaned_mock_tests`) and `cargo build` is clean.
