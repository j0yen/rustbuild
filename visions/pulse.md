# Vision: pulse — a deaf box looks exactly like a quiet room, until it doesn't

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-13
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection. companion and kin are
both **shipped end-to-end** (voice TURN live since 2026-06-04;
`wintermute-reach` v0.5.0, `wintermute-presence`, `wintermute-family-enroll`
all built). Probing for the genuine remaining seam surfaced kin's own
Fleet-2 evidence: the safety story has a hole nobody is watching.

## TL;DR

The companion sits on a desk at jsy's mother's home, always-listening, no
keyboard, unattended for days. kin makes it reach jsy when she *speaks* —
"tell Joe," "I've fallen." But **every safety guarantee in companion and
kin assumes the device can still hear.** Today nothing watches that
assumption. If `wm-audio` wedges, the mic vanishes, the ONNX detector
silently stops firing, or the model dir gets clobbered, the box goes deaf
— and a deaf box is **indistinguishable from a quiet, healthy one.** She
says "hey wintermute, I need help" and nothing happens; she has no way to
know it's broken, and — the load-bearing failure — **jsy has no way to know
either.** kin's silence-nudge (`reach/src/silence_nudge.rs`) fires
"haven't heard from Mom today" on `wm.presence.silence`, which it cannot
tell apart from "the device can't hear Mom." The one alert that does fire
points the wrong way.

pulse closes the hole. It turns the on-demand hearing check rouse shipped
(`wm-audio selftest`) into a **continuous, caregiver-facing liveness
attestation**: the device proves on a cadence that its ears work, escalates
a *sustained* deaf state to jsy through the same durable transport kin uses
for distress, and — critically — **gates the silence-nudge on device-alive**
so "haven't heard from Mom" only fires when the box was provably listening.
Silence becomes absence again, not deafness. Built honestly on shipped
pieces; nothing here re-invents the voice loop or the transport.

## What's actually there today (Phase 1 evidence, 2026-06-13)

- **The hearing self-check exists but is on-demand only.**
  `wintermute-audio/src/selftest.rs` injects a fixture (or `--live` reads
  the mic) through the *real* inference path and asserts `wm.audio.wake` +
  `wm.audio.speech.{start,end}` appear, with an `exit_code()` contract
  (selftest.rs:56). It is a one-shot CLI — it **does not publish a
  `wm.health.*` envelope** any daemon can consume, and nothing runs it on a
  cadence. The primitive is built; it just never speaks to the fleet.
- **A `wm.health.*` envelope convention already exists.**
  `wintermute-almanac/src/daemon.rs:147` publishes `wm.health.almanac`;
  `docket/src/digest.rs:84` documents the "`wm.health.*`-compatible
  envelope (matches companion-degrade shape)." pulse mints
  `wm.health.hearing` in that established shape — net-new topic, known
  pattern.
- **No `wm.health.hearing` topic exists anywhere.** grep across
  `~/wintermute/*/src/*.rs` returns the almanac/docket health envelopes and
  nothing about hearing or ear-liveness. This is honestly net-new on the bus.
- **The silence-nudge conflates dead with quiet.**
  `wintermute-reach/src/silence_nudge.rs` fires a gentle delivery on
  `wm.presence.silence` with no concept of device-alive. kin's own Fleet-2
  note already flagged the nudge as half-built; pulse identifies the deeper
  bug — it can't tell which failure it's reporting.
- **The durable escalation ladder is already built.**
  `wintermute-reach/src/distress_delivery.rs:41 run_distress_ladder()` does
  bounded retry + multi-transport fallback for distress. A deaf-device alert
  is a safety event of the same class; pulse reuses this ladder rather than
  inventing delivery.
- **`wintermute-presence`** (state.rs, daemon.rs, silence.rs) already tracks
  interaction counts and a waking-hours window and persists per-window
  state. It is the natural home for an active hearing-liveness check keyed to
  the same window — it already knows "did anything happen today"; pulse adds
  "could anything have happened today."

## End-state

When this vision is fulfilled:

1. **The device attests its hearing on a cadence.** A `wm.health.hearing`
   envelope (ok/degraded/deaf + last-wake-age + detector-loaded + model-present)
   is published periodically, derived from the *real* inference path, not a
   liveness ping of the process.
2. **A sustained deaf state reaches jsy fast, and durably.** N consecutive
   failed hearing checks cross the device to DEAF and fire a deaf-device alert
   to jsy over the distress ladder (retry + fallback transport), debounced so a
   flapping mic doesn't spam. "Wintermute at Mom's hasn't been able to hear for
   12 minutes" is a different, louder message than "haven't heard from Mom."
3. **Silence means absence, not deafness.** The silence-nudge only fires when
   the device was *confirmed hearing* across the window. A window the device
   spent deaf suppresses the (misleading) silence-nudge and is covered by the
   deaf-escalation instead. The two alerts never contradict each other.
4. **She can ask, and be reassured.** (Next-pass bullet — see Open questions.)
   "Are you listening?" / "Can you hear me?" runs a live hearing probe and the
   companion says "Yes, I'm here and listening" or, honestly, "I'm having
   trouble hearing — I've let Joe know." The reassurance verb the always-on
   device owes a non-technical elder.

## Components (PRD-sized pieces)

In dependency order. Components 1–4 drafted this pass; 5 left as a vision
bullet pending the dialog Health-branch design.

1. **PRD-pulse-hearing-probe** (rust-extend → `wintermute-audio`) — the floor.
   A `wm-audio selftest --emit` mode (+ a library entry point) that runs the
   existing real-path self-check and **publishes a `wm.health.hearing`
   envelope** on the bus in the companion-degrade/almanac shape:
   `{state: ok|degraded|deaf, last_wake_age_s, detector_loaded, model_present, ts}`.
   Reuses selftest.rs's fixture/`--live` machinery and `any_model_present()`
   (selftest.rs:204); adds no new inference. Independent — ships first.

2. **PRD-pulse-watch** (rust-extend → `wintermute-presence`) — the watcher.
   A cadenced active hearing-liveness check (triggers the probe, or subscribes
   to `wm.health.hearing`) with a HEARING / DEGRADED / DEAF state machine: K
   consecutive failures → DEAF, one success → HEARING. Persists liveness
   per-window alongside the existing presence state (state.rs) and **emits
   `wm.health.hearing.fail` on the DEAF edge** (and `.ok` on recovery). Keyed
   to the same waking-hours window presence already tracks. Depends on
   component 1's envelope.

3. **PRD-pulse-deaf-escalation** (rust-extend → `wintermute-reach`) — the
   alert. Subscribe `wm.health.hearing.fail`; deliver a deaf-device alert to
   jsy via `run_distress_ladder()` (retry + fallback transport — a deaf box is
   a safety event), debounced/rate-limited so a flapping detector sends one
   alert per sustained outage, not one per check. A `.ok` recovery sends a
   single "hearing restored" note. Distinct message class from kin distress
   (device-initiated, not Mom-initiated). Depends on component 2 + shipped
   reach transport.

4. **PRD-pulse-silence-gate** (rust-extend → `wintermute-reach`) — the fix.
   Gate `silence_nudge.rs` on device-alive: read the per-window liveness state
   component 2 persists; only fire "haven't heard from Mom" when the device was
   confirmed hearing across the window. A deaf window suppresses the nudge
   (deaf-escalation owns that case) so the two alerts never contradict. Closes
   the conflation bug directly. Depends on components 2 and 3.

## Order

```
pulse-hearing-probe   (wm-audio: mint wm.health.hearing; ships first)
        │
        ▼
pulse-watch           (presence: liveness state machine; emits .fail/.ok)
        ├──► pulse-deaf-escalation  (reach: durable alert to jsy)
        └──► pulse-silence-gate     (reach: gate the nudge on device-alive)
```

- probe is the gate: it defines the `wm.health.hearing` envelope everything
  keys on. Ship it first.
- watch turns the point-in-time envelope into a debounced liveness state and
  the `.fail` edge the two reach PRDs consume.
- deaf-escalation and silence-gate both extend `wintermute-reach` and touch
  disjoint modules (a new escalation path vs. `silence_nudge.rs`), so they can
  build in parallel; sequentially they rebase onto each other's `version` bump.

## Open questions

1. **Active probe vs. passive inference, and its cost.** The honest liveness
   check injects a fixture through the *real* ONNX path (selftest's existing
   mode) — but doing that every N minutes spins inference on a 4-core box that
   already pins under load. Cadence and fixture-vs-`--live` are a real
   power/heat tradeoff. Drafted: low-frequency fixture probe (cheap, no mic
   contention) with `--live` reserved for the on-demand "are you listening?"
   verb. **Worth jsy's read on cadence.**
2. **What counts as DEGRADED vs DEAF.** Model present but detector cold?
   Mic enumerated but RMS flatlined? One missed wake vs. K missed? Drafted: K
   consecutive failed probes → DEAF; model-absent or detector-unloaded →
   immediate DEAF; transient single failure → DEGRADED (no alert). The exact K
   and the DEGRADED band want tuning against a real deployment.
3. **Does Mom hear about her own deaf device?** When the box knows it's deaf,
   should it *say so* ("I'm having trouble hearing")? It can still speak even
   when it can't hear — TTS is a separate path. Leaning yes, gently, but a box
   that announces its own faults could alarm a non-technical elder. Tied to
   component 5.
4. **Component 5 — the "are you listening?" reassurance verb** (rust-extend →
   `wintermute-dialog`). A Health branch in the FSM (sibling to the shipped
   Family/Distress branches): deterministic recognition of "are you there /
   can you hear me" → live hearing probe → spoken result. Left undrafted this
   pass because the dialog FSM Health-branch shape wants its own look (does it
   reuse the degrade phrase bank? does a deaf result auto-fire escalation?).
   Next /dream pass picks it up.

## Notes for /build

- **Reuse, don't reinvent.** probe extends `selftest.rs` (don't add a second
  inference path); escalation reuses `run_distress_ladder()` (don't write new
  delivery); watch extends presence's existing window/state machinery.
- `wm.health.hearing` is a plain agorabus string — keep it identical across
  the three repos that touch it (audio publishes, presence consumes+re-emits,
  reach consumes). No shared crate needed; match the strings to this doc.
- Every new `wm.health.*` / `wm.presence.*` publisher must apply the
  self-emitted-topic filter (the sibling pattern wm-tts/wm-audio established) —
  presence already does; the deaf-escalation subscriber must not loop on its
  own `.ok`.
- **Safety semantics:** deaf-escalation must NOT depend on the Claude API path
  — it is a deterministic bus-event → transport delivery, same reasoning kin's
  distress used. The brain being down is itself a reason the box might look
  deaf; the alert can't depend on the thing that may be broken.
- Default cadence conservative; default deaf-escalation ON (it's a safety
  signal, like kin distress), silence-gate change is pure correctness (no new
  opt-in). presence's waking-hours window already gates when checks matter.
- Red-baseline reality across all four: bar = compiles + `cargo test` green.
  MSRV 1.85, no let-chains, `sigpipe::reset()` already in each crate's main.
  agorabus is the path dep version 0.9.
