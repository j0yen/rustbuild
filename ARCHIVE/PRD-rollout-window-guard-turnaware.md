# PRD: rollout-window-guard-turnaware — never bounce a voice daemon mid-utterance, using the real turn signal

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/rollout
Vision: visions/vigil.md

## TL;DR

`rollout`'s only protection against killing the voice loop mid-sentence
is a coarse `--window` time sample of `wm.dialog.turn.*`. The vigil
vision deferred a *precise* guard (Fleet 2 `rollout-window-guard`)
because it needed "a reliable turn-in-progress signal" that did not yet
exist. It exists now: `wm.dialog.turn.{user,system}` and
`wm.brain.session.{start,end}` are live bus events. This PRD makes the
guard subscribe to those events and refuse to restart any voice daemon
while a turn or session is actually in flight — and extends the voice
set to include `wm-audio` (the mic pipeline), which the current guard
wrongly excludes.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The deferral's blocker is now resolved.** vigil Fleet 2: *"refuse/
  defer restart of a voice daemon while a dialog turn is in flight.
  Depends on a reliable 'turn in progress' signal, which the
  continuity-of-conversation vision … is about to mint. Draft once that
  session-boundary event exists; until then rollout uses a coarse
  `--window` time guard only."* The events are live: grepping
  `wintermute-dialog`/`wintermute-brain` source yields
  `wm.dialog.turn.user`, `wm.dialog.turn.system`, `wm.brain.session.start`,
  `wm.brain.session.end`. The dependency is satisfied; the PRD is honest.
- **The current guard is coarse.** `health.rs` documents the `--window`
  guard as "a short `agorabus subscribe` sample to detect recent
  `wm.dialog.turn.*` activity" — a fixed-duration sample, not a true
  in-flight test. It can both false-allow (sample lands in a gap between
  turns) and false-block (idle chatter).
- **wm-audio is wrongly outside the voice set.** `health.rs`
  `VOICE_SET_PATTERN = r"^wm-(dialog|stt|tts)$"` — it excludes
  `wm-audio`, yet `wm-audio` is "the mic pipeline" (`wm-audio.service`
  description). Restarting it drops microphone capture mid-turn just as
  destructively as restarting STT. The guard must cover it.
- **This is the daemon set that goes stale.** binstale reports exactly
  `wm-audio|dialog|stt|tts` as `behind-head` — the daemons a real
  rollout will want to bounce are precisely the voice set this guard
  protects.

## What this builds

In `~/wintermute/rollout/` (`health.rs`):

1. **A turn/session liveness probe.** Subscribe (bounded) to
   `wm.dialog.turn.` and `wm.brain.session.` on agorabus. Maintain a
   simple in-flight state: a `session.start` without a matching
   `session.end`, or a `turn.user`/`turn.system` seen within a short
   idle window, means "in flight." Expose
   `voice_activity_in_flight(timeout) -> bool`.
2. **Block voice-daemon restarts while in flight.** Before restarting a
   voice daemon under `apply`, if `voice_activity_in_flight` is true,
   **defer**: either wait (bounded, for `session.end` / idle) and retry,
   or skip that daemon this run with a clear `deferred: voice active`
   verdict — never SIGTERM/restart through an active turn. Non-voice
   daemons (e.g. `recalld`, `agorabus`) are unaffected by this guard.
3. **Extend the voice set to include `wm-audio`.** Change
   `VOICE_SET_PATTERN` to `^wm-(audio|dialog|stt|tts)$` and adjust the
   doc comment; add a test pinning that `wm-audio` is now classified as
   a voice daemon.
4. **Graceful degradation.** If agorabus is unreachable, follow the
   existing conservative posture in `health.rs` (do not hard-fail the
   run on a probe error) — but for a *voice* daemon, an unreachable bus
   should **defer**, not proceed, because we cannot prove the loop is
   idle. Document this explicitly.

No change to non-voice restart timing. The coarse `--window` flag may
remain as a manual override but is no longer the only protection.

## Acceptance criteria

1. `health.rs` exposes a function that returns whether a voice turn /
   session is in flight, driven by `wm.dialog.turn.*` /
   `wm.brain.session.*` subscriptions (not a fixed time sample alone).
2. With a simulated in-flight session (`session.start` and no
   `session.end`), an `apply` against a voice daemon **defers** and does
   not issue a restart; a unit test asserts no restart call is made.
3. With no activity (or a clean `session.end`), the same `apply` is
   permitted to proceed.
4. `is_voice_daemon("wm-audio")` returns `true`; `VOICE_SET_PATTERN`
   includes `audio`. Tests cover `wm-audio|dialog|stt|tts` true and a
   non-voice name (e.g. `recalld`, `agorabus`) false.
5. When agorabus is unreachable, a voice-daemon restart **defers** with
   a verdict explaining the bus was unreachable, rather than proceeding
   blind; non-voice daemons follow the existing conservative path.
6. The deferral is reported in the rollout result (`deferred` /
   `skipped: voice active`) so a `rollout apply` that skipped a voice
   daemon says so, exits in a way self-review can detect, and does not
   falsely claim the daemon was refreshed.
7. `cargo build --release` + `cargo test` pass; clippy `-D warnings`
   clean for `health.rs` (crate baseline permitting).

## Build note for /build

Modifies the rollout crate alongside **rollout-apply-systemd**.
**Serialize** the two /build cycles (or worktree-isolate) — both touch
`health.rs`/`restart.rs`. Ship `rollout-apply-systemd` first if
parallelism is unavailable; this PRD's deferral hooks into the
systemd-aware restart branch that PRD introduces.
