# PRD: reach-silence-nudge — surface "haven't heard from Mom today" as its own gentle nudge

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/wintermute-reach
Vision: visions/kin.md
Depends: none (extends shipped wintermute-reach v0.2.0; consumes the
  wm.presence.silence event the presence daemon already emits).

## TL;DR

The kin vision's fourth promise — *"Silence is surfaced, gently… you get a single
soft nudge"* — is half-built. The presence daemon already emits `wm.presence.silence`
when Mom hasn't interacted within her waking-hours window, and reach already
*notices* it — but only by setting a flag in the daily digest body
(`src/digest.rs:56 record_silence`, line 12: silence "does NOT trigger" anything).
If the daily digest is disabled (it defaults OFF), a silent day produces no signal
at all. This PRD adds a standalone, opt-in, debounced silence nudge: a single soft
delivery to jsy when silence fires, independent of the digest, so a quiet day is
surfaced even when no digest is configured.

## Why this exists

Phase-1 evidence (2026-06-06, direct grep of shipped repos):

- **Silence is detected but not delivered.** `wintermute-reach/src/digest.rs:3-12`
  documents that the digest "subscribes to `wm.presence.summon` and
  `wm.presence.silence`" and that silence is "reflected in the digest body but does
  NOT trigger" any delivery; `digest.rs:56 record_silence` only sets
  `silence_flagged = true`, surfaced via `format_digest_body`
  (`digest.rs:94`: "(Note: a silence window was flagged today.)").
- **The digest defaults OFF, so silence can vanish entirely.** `visions/kin.md`
  Notes-for-build: "Presence/silence/digest default OFF in config." A caregiver who
  enabled silence-detection but not the daily digest gets *nothing* on a silent day
  — the one day the vision says they most want a nudge.
- **The vision asked for a distinct signal, not a digest line.** End-state #4:
  *"If she hasn't interacted at all within her waking-hours window… you get a single
  soft nudge — not an alarm, a 'haven't heard from Mom today.'"* That is a delivery
  event, debounced to once per window, distinct from the batched digest.
- **The presence emitter already debounces at the source.** `wintermute-presence`
  README AC: silence is emitted "one signal per window, debounced, never outside
  waking hours." So reach can treat each `wm.presence.silence` as authoritative and
  needs only its own re-delivery guard (don't nudge twice if the daemon restarts and
  replays, and never escalate a nudge into an alarm).
- **Distress must stay first-class above this.** `digest.rs:11`: "Distress is always
  instant — the digest never delays distress delivery." The silence nudge is the
  opposite end of the urgency scale; it must use the gentle phrasing and never
  borrow the distress priority path.

## What this builds

A `silence_nudge` path in `wintermute-reach`, default OFF, reusing the existing
transport layer the digest and family deliveries already use.

- **`SilenceNudgeConfig { enabled, contact_name, debounce }`** in config; default
  `enabled = false`. Read from `/etc/wintermute/conf.d/` with `WM_REACH_SILENCE_*`
  env fallbacks, matching the existing config pattern.
- **A subscriber path** on `wm.presence.silence`: when enabled, format a single
  gentle body — e.g. `"Haven't heard from {contact_name} today."` — and deliver it
  through the existing transport (the same one the digest/family messages use), at
  normal (non-distress) priority.
- **Debounce / idempotency** — a per-window guard (last-nudged window key persisted
  to the reach state file, mirroring how the digest's daily counter survives restart
  per the presence README's round-trip AC) so a daemon restart replaying a silence
  event does not produce a second nudge for the same window.
- **Digest coexistence** — when both the digest and the silence nudge are enabled,
  silence is still flagged in the digest body *and* the standalone nudge fires; they
  are independent opt-ins, not mutually exclusive (document this; it's a deliberate
  belt-and-suspenders for a safety-adjacent signal).
- **Never an alarm** — the nudge is a single delivery; it does not repeat, does not
  escalate, and does not use the distress transport-priority path.
- Version bump to **v0.4.0** (or next free minor if built after inbound-imap).

## Acceptance criteria

1. With `SilenceNudgeConfig::enabled = false` (default), a received
   `wm.presence.silence` produces zero deliveries — behaviour identical to v0.2.0
   (regression: existing digest tests still green).
2. With the nudge enabled, one `wm.presence.silence { since_ts, window }` produces
   exactly one transport delivery whose body contains the configured contact name
   and the gentle phrasing (no "alarm"/"emergency" wording — asserted).
3. A second `wm.presence.silence` for the *same* window (e.g. after a daemon
   restart/replay) produces no additional delivery — the debounce guard holds.
4. A `wm.presence.silence` for a *new* window after a prior nudged window produces a
   new delivery (the guard is per-window, not permanent).
5. The debounce state survives a daemon restart (state-file round-trip test, mirror
   of the presence daily-counter AC).
6. The silence nudge is delivered at normal priority and never pre-empts or shares
   the distress fast-path (assert a queued distress is still delivered ahead of /
   independent of a nudge).
7. When both digest and nudge are enabled, a silent day yields both the digest's
   silence note and one standalone nudge (independence test).
8. `cargo test` green; `cargo clippy` clean under the crate's existing `-D`-grade
   lints; release-gate receipts produced per autobuilder.

## Out of scope

- Re-nudging / escalation if silence persists across multiple windows (would turn a
  gentle nudge into the alarm the vision explicitly rejects). One nudge per window.
- Changing the presence daemon's emission logic — this PRD only consumes the event
  reach already receives.
