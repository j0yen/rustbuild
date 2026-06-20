# PRD-fluid-brain-key-health

**Status:** Draft v0.1
**Vision:** visions/fluid-voice.md
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-brain

## TL;DR

A rotated or expired Anthropic API key currently causes every voice turn to
degrade to a nonsensical fallback phrase with no visible error. Add a proactive
key-health probe that detects 401 on startup, publishes a bus event, and routes
turns to the local tier instead of emitting degraded nonsense.

## Why this exists

`journalctl --user _PID=2778192` on 2026-06-20 shows every turn:

```
WARN  wm-brain ladder: cloud tier failed; climbing tier=sonnet
      err=anthropic api status 401: {"error":{"type":"authentication_error",
      "message":"invalid x-api-key"}}
ERROR wm-brain ladder: turn degraded (no tier could serve)
      reason=top tier error: anthropic api status 401
```

The brain climbs the ladder, hits 401 on both haiku and sonnet, exhausts the
ladder, and emits a `LadderOutcome::Degraded` phrase ("I have to think about
that" or similar) — which wm-tts speaks verbatim. The user hears nonsense;
nothing on the bus indicates a key problem.

`WM_BRAIN_SKIP_TIERS=local-8b` is set (bootstrap env) so the local 3b tier
is also skipped (or not present). On 401, the only correct path is to route
to whatever local tier IS available, and to surface the key failure visibly.

## What this builds

Extend `~/wintermute/wintermute-brain` — new module `src/key_health.rs` +
minimal changes to `daemon.rs` and `ladder.rs`:

**`src/key_health.rs`** — `KeyHealthProbe`:
- `KeyHealthProbe::check(api_key: &str) -> KeyHealthResult` — async fn that
  sends a minimal Anthropic API request (e.g. `POST /v1/messages` with
  `max_tokens: 1`, `model: claude-haiku-4-5-20251001`) and maps the response:
  - 2xx → `KeyHealthResult::Ok`
  - 401/403 → `KeyHealthResult::AuthError { code: u16 }`
  - other → `KeyHealthResult::NetworkError { msg: String }`
- `KeyHealthProbe::spawn_background(api_key, interval, bus_publisher)` — runs
  `check()` at daemon start and every `interval` seconds (default: 300 s),
  publishes `wm.brain.key_status { ok: bool, code: Option<u16>, ts: u64 }` on
  each check.

**`daemon.rs`** — wire the probe:
- On daemon start, run `KeyHealthProbe::check()` once synchronously (before
  accepting any turns). If `AuthError`, log `ERROR wm-brain: API key invalid`
  and set a `key_healthy: bool` flag in daemon state.
- Spawn `spawn_background(300 s)` after startup.
- In `handle_turn_user`: if `!key_healthy`, skip cloud rungs entirely (route
  straight to local) without attempting the ladder climb that produces 401 +
  degraded fallback.

**`ladder.rs`** — no changes; the `key_healthy` flag gates the cloud rungs
before the ladder is consulted.

## Acceptance criteria

1. **AC1 — startup probe fires.** Integration test (wiremock or httpmock):
   configure a mock Anthropic endpoint returning 401. Start a `KeyHealthProbe`;
   `check()` returns `KeyHealthResult::AuthError { code: 401 }`.
2. **AC2 — bus event published on bad key.** Test: run probe with mock 401
   endpoint + in-memory bus publisher; assert `wm.brain.key_status` event
   emitted with `ok: false, code: 401`.
3. **AC3 — bus event published on good key.** Test: run probe with mock 200
   endpoint; assert `wm.brain.key_status { ok: true }` emitted.
4. **AC4 — degraded fallback NOT emitted when key is bad.** Integration test:
   configure daemon with mock 401 endpoint; send a `wm.dialog.turn` event;
   assert `wm.brain.reply` is still emitted (from local tier), NOT
   `wm.brain.error` with `reason=turn degraded`. (Local tier must be available
   in the test — use a stub local backend that returns "ok".)
5. **AC5 — background probe re-checks.** Test with a probe interval of 1 s:
   first check returns 401 (`key_healthy = false`); mock endpoint then returns
   200; second check fires within 2 s and `key_healthy` becomes `true`.
6. **AC6 — cargo test green.** `cargo test` on `wintermute-brain` passes with
   no regressions.
