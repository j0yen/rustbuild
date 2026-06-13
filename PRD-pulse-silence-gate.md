# PRD: pulse-silence-gate — silence means absence, not deafness

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-reach
**Vision:** visions/pulse.md

## TL;DR

kin's silence-nudge tells jsy "haven't heard from Mom today" when the presence
daemon reports a silent window — but it cannot tell a quiet, healthy device
from a deaf one. The one alert that fires today points the wrong way: a deaf
box, the worst case, produces a *reassuring-sounding* "haven't heard from Mom"
that implies the device is fine and Mom is just quiet. This PRD gates the
silence-nudge on the device-alive state pulse-watch persists: the nudge fires
only when the device was provably hearing across the window; a deaf window is
suppressed and left to pulse-deaf-escalation. The two alerts stop contradicting
each other.

## Why this exists

Phase-1 evidence (2026-06-13):

- `wintermute-reach/src/silence_nudge.rs` fires a single debounced delivery on
  `wm.presence.silence` with **no concept of device-alive** — it cannot
  distinguish "Mom was quiet" from "the device couldn't hear Mom." kin's own
  Fleet-2 notes flagged the nudge as half-built; this is the deeper defect.
- pulse-watch now persists `hearing_confirmed_in_window` per waking-hours
  window (true iff an `ok` `wm.health.hearing` envelope arrived inside it), so
  reach can read whether the device was provably hearing.
- pulse-deaf-escalation now owns the deaf case with a louder, durable alert.
  Without this gate, a deaf window fires *both* a deaf-escalation and a
  contradicting silence-nudge.

## What this builds

A device-alive gate around the existing silence-nudge.

- **Read the per-window liveness state** pulse-watch persists
  (`hearing_confirmed_in_window` + current liveness state). The two repos share
  the agreed state location/topic (plain agorabus strings / a documented state
  file path — no new shared crate); match the contract in visions/pulse.md.
- **Gate the nudge.** In `silence_nudge.rs`'s delivery path: fire "haven't
  heard from Mom" **only when `hearing_confirmed_in_window` is true** for the
  window the silence event refers to. If the device was not confirmed hearing
  (deaf or never-probed) in that window, **suppress** the nudge — the deaf case
  is covered by pulse-deaf-escalation; the never-probed case is ambiguous and a
  misleading nudge is worse than none.
- **Preserve existing debounce.** The gate is *in front of* the existing
  debounce guard, not a replacement — a gated-out window still must not later
  double-fire.
- **Observability.** When a nudge is suppressed because the device was deaf,
  emit a structured log line (`action:"silence_nudge_suppressed_deaf"`) so the
  decision is auditable; do not deliver anything to jsy from this path (the
  escalation already did).
- **Config.** No new opt-in — this is a correctness fix to an existing feature.
  Honor the existing silence-nudge enable flag; when disabled, behavior is
  exactly as today.

MSRV 1.85, no let-chains, `sigpipe::reset()` already in main.

## Acceptance criteria

1. A `wm.presence.silence` for a window where `hearing_confirmed_in_window` is
   **true** fires the silence-nudge exactly as today (regression-equivalent to
   the current behavior); tested.
2. A `wm.presence.silence` for a window where the device was **deaf** (liveness
   `Deaf`, `hearing_confirmed_in_window` false) delivers **no** silence-nudge
   and logs `silence_nudge_suppressed_deaf`; tested.
3. A `wm.presence.silence` for a window with **no hearing probe recorded**
   (ambiguous) suppresses the nudge (drafted policy: no confirmation → no
   nudge); tested. If jsy prefers fire-on-ambiguous, it is a one-line flag —
   note it but default to suppress.
4. The existing debounce guard is preserved: a suppressed window does not later
   double-fire when a subsequent event arrives; tested against the existing
   debounce-state test.
5. With the silence-nudge feature disabled, behavior is byte-for-byte as today
   (no gate evaluation, no new logs); regression test.
6. The shared liveness-state contract (location + field names) is documented in
   the PRD and matches what pulse-watch writes; an integration test writes the
   pulse-watch state and reads it through this gate.
7. `cargo test` green; `cargo build --release` clean; reach `version` bumped
   (rebased onto pulse-deaf-escalation's bump if built after it).
