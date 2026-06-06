# PRD: anchor-probe — make the delta layer fail loud, not silent

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/anchor`
**Vision:** visions/anchor.md

## TL;DR

The deepest part of the watchman pain isn't that roots get lost — it's that
the loss is **silent**: `wchg since`/`reset` return empty or error, and "empty
delta" is indistinguishable from "nothing changed this week." **anchor-probe**
extends the `anchor` repo with a pure-read health check that reports lost
roots, stale clocks, and a dead watchman socket as structured JSON with a
non-zero exit, so self-review (and any wchg consumer) can gate on real watch
health instead of trusting a quiet zero.

## Why this exists

- **Silence is the actual bug.** Journal 06-03 Notable: *"`wchg since`/`reset`
  silently returned empty/errored until re-`watch`."* Journal 06-02 run #1:
  the delta was *"unavailable"* and only noticed by hand. A consumer that can't
  tell "watch lost" from "no change" will under-report exactly when the box
  changed most.
- **The mechanism recurs beyond reboot.** `recall:01KSGR03R1KS6EHKBMVD019BFX`:
  *"wchg-watchman socket teardown happens mid-session, not just at reboot."* A
  health probe is the only thing that catches a mid-session loss before a
  consumer trusts an empty result.
- **anchor-roots already computes the classification** (`Missing` / `Stale` /
  `Watched`) and owns the `WatchBackend` trait. The probe is a thin,
  read-only consumer of that surface plus a watchman-socket liveness check —
  small scope precisely because anchor-roots shipped the corpus first.

## What this builds

Extends `~/wintermute/anchor` (no new repo). Adds an `anchor probe` subcommand
and a `probe` module.

- **`ProbeReport { socket_alive: bool, roots: Vec<RootHealth>, worst:
  Severity, checked_at: i64 }`** where `RootHealth { path, status: RootStatus,
  clock: Option<String>, age_secs: Option<u64> }` reuses `RootStatus` from
  anchor-roots. `Severity` ∈ `Ok | Stale | Missing | SocketDown`.
- **Socket liveness** behind the existing `WatchBackend` (add a
  `fn ping(&self) -> Result<bool>` default-implemented via `watchman
  version`/`watch-list`): a dead or unreachable watchman socket is reported as
  `socket_alive: false`, `worst: SocketDown` — the single most important signal,
  because every root read is meaningless if the daemon is gone.
- **Pure-read by contract.** The probe NEVER calls `watch` or `reseed_cursor`
  (those are anchor-reconcile's `--apply` path). It only reads `live_roots` +
  `ping`. A test asserts no mutating backend method is invoked.
- **Stale detection** uses each root's `max_age_secs` (or a config default) vs
  an **injected** `now` (no `Date::now` — `self_*` clock-injection discipline),
  so the same fixture is deterministic on any box.
- **UX:** `anchor probe` → human table; `--format json` → `ProbeReport`. Exit
  code encodes severity: `0` all `Ok`, `1` any `Stale`, `2` any `Missing`,
  `3` `SocketDown` (highest wins) — so a hook can branch on watch health.
  `sigpipe::reset()` already in `main` (inherited).

## Acceptance criteria

1. `cargo build` and `cargo test` succeed offline; the probe runs entirely
   against a `FakeBackend` with **zero** real watchman/network calls (asserted).
2. `ProbeReport` and `RootHealth` are public and `serde`-(de)serializable;
   `anchor probe --format json` round-trips to the documented schema.
3. A `FakeBackend` reporting the socket down yields `socket_alive: false`,
   `worst: SocketDown`, and exit code `3`, regardless of root state.
4. With the socket up: a declared-but-absent root → `Missing` (exit `2`); a
   watched root whose clock age (vs injected `now`) exceeds `max_age_secs` →
   `Stale` (exit `1`); all-fresh → `Ok` (exit `0`). The highest severity sets
   the exit code; covered by integration cases.
5. A test asserts the probe path invokes **only** read methods of
   `WatchBackend` (`live_roots`, `ping`) and never `watch`/`reseed_cursor`.
6. The stale threshold is driven by the manifest `max_age_secs` (anchor-roots
   `RootsConfig`), not a hardcoded constant; a test with two roots at different
   thresholds classifies each independently.
7. `anchor probe | head -1` does not panic on SIGPIPE.
8. The integration test entry file `tests/probe.rs` appears in `cargo test`
   output (`self_orphaned_mock_tests` guard — a mocks subdir without a
   top-level entry compiles to nothing and false-greens).
9. README documents `anchor probe`, the exit-code contract, and how a
   SessionStart hook should branch on it.
