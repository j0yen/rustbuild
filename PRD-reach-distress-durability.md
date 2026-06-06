# PRD: reach-distress-durability — a failed distress delivery must retry and escalate, not die as one nack

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/wintermute-reach
Vision: visions/kin.md
Depends: none (extends shipped wintermute-reach v0.2.0; the ntfy/webhook
  fallback transports are the crate's existing default-off Cargo features).

## TL;DR

The kin vision's load-bearing promise — *"Distress reaches you instantly"* — has no
delivery durability. When Mom says "I've fallen," the dialog fires
`wm.family.distress` and reach delivers it over a single transport. If that
transport fails, reach acks `delivered: false` and stops. AC5 of the shipped crate
deliberately makes a failure "not a panic, not a silent drop" — but for the *one*
event class that is genuinely a safety signal, a single nack that nobody is watching
*is* a silent drop. This PRD makes distress delivery durable: bounded retry with
backoff on the primary transport, then escalation to the next configured fallback
transport, before `delivered: false` is ever emitted.

## Why this exists

Phase-1 evidence (2026-06-06, direct grep of shipped repos):

- **Distress is delivered exactly once, over one transport.** `wintermute-reach/src/daemon.rs:35`
  models `EventKind::Distress { body }` and `daemon.rs:75,131` route it to a single
  delivery; there is no retry loop and no second-transport path in the daemon.
- **A failure currently ends at a nack.** The shipped AC5 (reach README): "A
  transport error yields `wm.family.ack { delivered: false }` — not a panic, not a
  silent drop." Good for an ordinary message. But the same handling applies to
  distress, and a `delivered:false` ack on the bus is not a person being reached —
  it is the safety loop terminating unobserved.
- **The vision rates this safety-critical.** `visions/kin.md` end-state #5 and the
  family-distress component both stress distress is "highest priority," "bypasses
  batching," fires on the "non-API path" so it works when the brain is unreachable.
  The same reasoning demands the *delivery* not hinge on one transport succeeding on
  the first try — the network blip that drops the email is exactly when Mom has
  fallen.
- **Fallback transports already exist, unused for escalation.** `wintermute-reach/Cargo.toml:20-21`
  ships `ntfy` and `webhook` as Cargo features (the vision: "wires email first and
  gates the rest behind Cargo features"). Today they are alternative *primary*
  transports; nothing chains them as distress fallbacks. The mechanism is present —
  it just needs to be a retry/escalation ladder for distress.
- **Scope is naturally bounded to distress.** `dialog/src/distress.rs` only fires
  `wm.family.distress` for `Severity::Hard` or a confirmed `Soft` (OQ#3 resolved).
  So every distress on the bus is already "must reach Joe" — this PRD doesn't need
  to re-classify; it hardens delivery for the whole (already-filtered) distress class
  and leaves ordinary `wm.family.message` delivery untouched.

## What this builds

A distress delivery ladder in `wintermute-reach`, scoped to the distress path only.

- **`DistressDelivery` policy** — `{ retries, backoff, fallbacks: Vec<TransportId> }`.
  On a distress event: attempt the primary transport; on failure, retry up to
  `retries` with bounded backoff; on continued failure, attempt each configured
  fallback transport in order. Emit `wm.family.ack { delivered: true, transport }`
  on the first success (naming which transport got through), or
  `wm.family.ack { delivered: false }` only after the whole ladder is exhausted.
- **Transport abstraction** — a small `Transport` trait the existing email plus the
  feature-gated `ntfy`/`webhook` backends implement, so the ladder iterates over a
  `Vec<Box<dyn Transport>>` built from config. (If a `Transport` trait already
  exists internally, extend it; do not duplicate.)
- **Ordinary messages unchanged** — `wm.family.message` keeps single-attempt
  semantics (AC5 of v0.2.0); only `wm.family.distress` runs the ladder. Assert this
  so message latency/behaviour does not regress.
- **Bounded, non-blocking** — the retry/backoff is bounded (config-capped, sane
  default e.g. 3 retries / fallbacks within a few seconds) and must not block the
  daemon's select loop from handling a *newer* distress; deliveries run as spawned
  tasks consistent with the existing daemon structure.
- **Observability** — each attempt logs transport + outcome (no body) so a missed
  escalation is auditable; the final ack names the transport that succeeded.
- Version bump to **v0.5.0** (or next free minor).

## Acceptance criteria

1. A distress whose primary transport fails once then succeeds on retry yields
   exactly one `wm.family.ack { delivered: true }` and no `delivered: false` — the
   retry is transparent.
2. A distress whose primary transport fails all retries but whose first fallback
   succeeds yields `wm.family.ack { delivered: true, transport: <fallback> }` naming
   the fallback, using a `FakeTransport` sequence (fail-primary, ok-fallback).
3. A distress that fails every configured transport yields exactly one
   `wm.family.ack { delivered: false }`, emitted only after the full ladder is
   exhausted (not after the first failure).
4. Retry/escalation is bounded by config (count + backoff cap); a pathological
   always-failing transport terminates the ladder within the cap, never loops
   unbounded (test asserts attempt count == cap).
5. An ordinary `wm.family.message` still uses single-attempt delivery — a message
   transport failure yields `delivered: false` immediately, with no retry/escalation
   (distress-only scoping; regression of v0.2.0 AC5).
6. A second distress arriving while the first is still retrying is not starved — both
   reach a terminal ack (the ladder runs off the select loop, not in it).
7. No distress body is logged; attempt logs carry transport id + outcome only.
8. ntfy and webhook fallbacks compile behind their existing Cargo features and are
   excluded from the default build; the ladder degrades to email-only-with-retry
   when no fallback feature is enabled.
9. `cargo test` green; `cargo clippy` clean under the crate's existing `-D`-grade
   lints; release-gate receipts produced per autobuilder.

## Out of scope

- A real second-channel (SMS gateway / self-hosted ntfy) deployment — this PRD makes
  the *ladder* real and proves it with fakes/feature-gated backends; which physical
  fallback jsy runs on his phone is the vision's OQ#1, a deployment decision.
- Persisting un-delivered distress across a full daemon crash for later replay — a
  durable on-disk distress outbox is a worthwhile follow-on (note it in the vision),
  but v1 hardens the in-flight delivery, not crash-recovery.
