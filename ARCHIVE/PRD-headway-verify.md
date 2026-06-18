# PRD: headway-verify — prove the daemon flipped fresh, or surface a contradiction

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/headway
Vision: visions/headway.md

## TL;DR

A rebuild-and-reload loop that reports success on exit 0 is a false-green
waiting to happen: the build can succeed, the install can land, the daemon can
bounce, and the running image can *still* be `behind-head` (wrong dest, race,
stale unit). This PRD closes the loop honestly — after install + reload,
re-run `binstale` on the daemon and assert the verdict flipped
`behind-head → fresh`; emit a receipt, and when it did not flip, surface a
`contradicted` verdict for triage instead of false-closing.

## Why this exists

Phase-1 inspection, 2026-06-18:

- `binstale check --format json <PID>` returns the live verdict for a single
  running process (`fresh | deleted-exe | inode-drift | prov-stale |
  behind-head`); `binstale/src/fleet.rs` computes `behind-head` from build-ts vs
  newest source commit. This is the ground-truth oracle the loop can re-query.
- The fleet has a standing precedent against false-closing: the
  answerable/threshold work forbids silently dropping a `contradicted` verdict
  (recorded in gossip for vision-threshold, 2026-06-18: "its ACs forbid silently
  dropping a `contradicted` verdict (warrant/assay false-close class)").
- The recurring `behind-head` item that survives self-reviews is exactly the
  failure a verify step guards against: something *looked* remediated but the
  verdict never actually changed. Without a re-check, headway would inherit the
  same blind spot.
- `headway-build` (foundation PRD) emits a `BuildVerdict` but stops at "artifact
  built/installed"; it does not prove the *running daemon* advanced.

## What this builds

Extend `~/wintermute/headway`.

- A `verify(daemon, expected_repo_head) -> VerifyReceipt` that, after a
  rebuild + install + reload, re-runs `binstale check <pid> --format json`
  against the (new) daemon pid and classifies:
  - `confirmed` — verdict is now `fresh`; the loop converged.
  - `contradicted` — verdict is still `behind-head` (or any non-fresh staleness);
    the remediation did not take. This is reported, never swallowed.
  - `inconclusive` — binstale returned `unknown` (no daemon / unreadable /proc).
- A `VerifyReceipt { daemon, pid_before, pid_after, verdict_before,
  verdict_after, outcome, ts }`, emitted as JSON, suitable for a self-review or
  docket consumer.
- CLI: `headway verify <daemon> [--format json|table]`, and a `headway run`
  that composes build → install/reload → verify into one receipt (the
  end-to-end "bring this daemon into headway" command).
- Exit codes: 0 = confirmed, 1 = contradicted, 2 = inconclusive/error — so a
  cron or self-review caller can gate on convergence.

## Acceptance criteria

1. `verify` re-runs `binstale check <pid> --format json` (subprocess, stubbable
   via env for tests) and maps `fresh → confirmed`, `behind-head → contradicted`,
   `unknown → inconclusive`.
2. A `contradicted` outcome is always reported in the `VerifyReceipt` and yields
   exit 1 — there is no code path that downgrades a still-`behind-head` daemon to
   a success. (Test: fixture binstale returns `behind-head` after reload → receipt
   `outcome=contradicted`, exit 1.)
3. A `confirmed` outcome (fixture binstale returns `fresh`) yields a receipt with
   `verdict_before=behind-head`, `verdict_after=fresh`, `outcome=confirmed`,
   exit 0.
4. An `unknown`/unreachable binstale yields `outcome=inconclusive`, exit 2, and a
   structured error — never confirmed.
5. `headway run <daemon>` composes build (headway-build) → install/reload →
   verify and returns a single receipt; on a stubbed happy path it ends
   `outcome=confirmed`.
6. `VerifyReceipt` JSON includes `pid_before`/`pid_after` so a reader can confirm
   the daemon actually bounced (pids differ on a real reload).
7. All pre-existing headway tests stay green; `cargo test` green and no new
   clippy warnings over the repo baseline; `headway verify --help` and
   `headway run --help` work on the freshly-built binary.
