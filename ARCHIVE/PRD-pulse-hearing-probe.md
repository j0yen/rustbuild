# PRD: pulse-hearing-probe — turn the on-demand hearing check into a bus signal

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-audio
**Vision:** visions/pulse.md

## TL;DR

The companion's only proof that it can still hear is `wm-audio selftest` —
a one-shot CLI that returns an exit code and tells *no one* on the bus. This
PRD adds a `--emit` mode (and a library entry point) that runs the existing
real-inference self-check and **publishes a `wm.health.hearing` envelope** so
the rest of the fleet can watch the device's ears. It adds no new inference;
it gives voice to a check that already works.

## Why this exists

Phase-1 evidence (2026-06-13):

- `wintermute-audio/src/selftest.rs` already drives a fixture (or `--live`
  mic read) through the *real* wake+VAD inference path and asserts
  `wm.audio.wake` + `wm.audio.speech.{start,end}` appear, exposing an
  `exit_code()` contract (selftest.rs:56) and an `any_model_present()` helper
  (selftest.rs:204). It is invoked from `main.rs:51 "selftest"`.
- But it publishes **no `wm.health.*` envelope** — its result dies as a
  process exit code. Nothing on the bus learns the device went deaf.
- The `wm.health.*` envelope shape is already established:
  `wintermute-almanac/src/daemon.rs:147` publishes `wm.health.almanac`;
  `docket/src/digest.rs:84` documents the companion-degrade-compatible shape.
- No `wm.health.hearing` topic exists anywhere in `~/wintermute/*/src`.

pulse-watch (next PRD) needs a structured, periodic hearing signal to build a
liveness state machine on. This PRD mints it.

## What this builds

A `--emit` path on the existing selftest, plus a small published envelope.

- **`wm-audio selftest --emit`** — runs the existing self-check (fixture by
  default; honors the existing `--live` flag), then publishes one
  `wm.health.hearing` envelope to the agorabus socket before exiting with the
  same `exit_code()` contract it already has. `--emit` is additive; bare
  `selftest` keeps its current behavior.
- **`wm.health.hearing` envelope** (serde, companion-degrade/almanac shape):

  ```json
  {
    "msg_type": "wm.health.hearing",
    "state": "ok" | "degraded" | "deaf",
    "last_wake_age_s": 0,
    "detector_loaded": true,
    "model_present": true,
    "wake_seen": 1,
    "speech_seen": 1,
    "ts": "<rfc3339>"
  }
  ```

  - `state` derives from the self-check result: models present + detector
    loaded + expected events seen → `ok`; events missing but path intact →
    `degraded`; model absent (`any_model_present()` false) or detector
    unloaded → `deaf`.
  - `detector_loaded` / `model_present` come from the existing inference
    setup + `any_model_present()`; `wake_seen` / `speech_seen` from the
    counters selftest already tracks (selftest.rs:79–83).
- **Library entry point** — a `pub fn run_hearing_probe(...) -> HearingHealth`
  in (or beside) `selftest.rs` returning the envelope struct, so pulse-watch
  can call it in-process without shelling out. The CLI `--emit` is a thin
  wrapper over it.
- Reuse the crate's existing agorabus publish path (the same client selftest
  already connects with to subscribe); apply the self-emitted-topic filter so
  the probe never consumes its own publication.

No new dependencies. No second inference path. MSRV 1.85, no let-chains,
`sigpipe::reset()` already in main.

## Acceptance criteria

1. `wm-audio selftest --emit` runs the existing self-check and publishes
   exactly one `wm.health.hearing` envelope to the agorabus socket, then exits
   with the same code `selftest` returns without `--emit` (verify the exit-code
   contract is unchanged for pass and fail).
2. The published envelope deserializes to a `HearingHealth` struct with all
   documented fields; `msg_type == "wm.health.hearing"`; `ts` is valid RFC3339.
3. `state` is `deaf` when `any_model_present(prefix)` is false (point a
   `--prefix` at an empty dir in the test); `ok` when the fixture path sees the
   expected wake + speech events; the mapping is unit-tested for all three
   states with fixtures (no live mic).
4. A `pub fn run_hearing_probe` library entry point returns the same
   `HearingHealth` the CLI publishes; a test calls it in-process and asserts
   the struct without spawning a process.
5. Bare `wm-audio selftest` (no `--emit`) behavior is byte-for-byte unchanged:
   no envelope published, same stdout, same exit code (regression test).
6. `cargo test` green; `cargo build --release` clean. No new crate in
   `Cargo.toml` beyond what's already present.
