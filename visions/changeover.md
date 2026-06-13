# Vision: changeover — zero-loss fleet daemon restarts

## TL;DR

The fleet's voice daemons (`wm-audio`, `wm-dialog`, `wm-stt`, `wm-tts`) go
stale and need restarts, but `rollout apply` restarts them with a hard
`systemctl --user restart` — a kill-then-start with a deafness window between
SIGTERM and the successor re-subscribing on agorabus. That window is why
`rollout apply` is never auto-approved (every self-review since 2026-06-12
parks fleet-binary-staleness on "daemon restarts drop subscribers"). Nobody
has ever *measured* how long the window actually is or how many bus events it
drops. **changeover** measures the window first, then closes it with a warm
overlap handoff built on agorabus's existing claim-lease primitive, then wires
the measured proof into rollout's auto-apply gate so the stale fleet can roll
itself without a human in the loop — and without going deaf mid-conversation.

## End-state

When this is done:

- A `changeover probe` run reports, for any fleet daemon, the exact
  deafness window (ms) and the count of bus events lost across a
  `rollout apply` restart — a number, not a guess.
- `rollout apply` can run a **warm-swap**: start the successor, wait for it
  to subscribe and acquire the daemon's agorabus claim, then stop the
  predecessor — so a live producer/consumer always holds the topic.
- `rollout apply --auto` is gated on a current `changeover probe` proof that
  the warm-swap window is under a configured threshold (default: 0 events
  lost). The four stale voice daemons roll themselves on the self-review
  cron once the proof is green.
- The user's voice loop never goes silent because a daemon was updated.

## Why now (evidence)

- `~/wintermute/rollout/src/restart.rs` delegates to
  `systemctl --user restart <unit>` then `poll_healthcheck` — a hard
  kill→start→poll-for-reregistration. No overlap; the window is real.
- `~/wintermute/agorabus/src/protocol.rs` already exposes `Publish` /
  `Subscribe` and a TTL **claim lease** (`ClaimAcquire`/`ClaimRelease`,
  broadcasts on `claim.acquire`/`claim.release`) plus `DrainNotice` +
  `reconnect_subscribe`. The handoff primitive exists; nothing uses it for
  peer restarts.
- Journal 2026-06-13 (and every self-review 2026-06-08 → 13):
  fleet-binary-staleness parked because "daemon restarts drop subscribers;
  requires explicit approval." wm-audio/dialog/tts/stt all behind-head.
- [[self_agorabus_restart_kills_voice]]: a *crash* of these daemons once
  killed voice end-to-end (fixed with Restart=always). A *planned* restart
  has the same window; Restart=always recovers a crash but doesn't overlap.
- [[feedback_verify_before_concluding]]: don't assert the window is a problem
  — instrument the actual restart path and measure the value first. The probe
  PRD is deliberately ordered before any agorabus/rollout surgery.

## Components (PRD-sized)

1. **changeover-probe** (new rust-cli, `~/wintermute/changeover/`) —
   measure the deafness window + event loss across a real `rollout apply`
   restart of a fleet daemon. Subscribe to the daemon's topics, drive
   synthetic publishes, trigger the restart, count the gap. Measurement
   only; mutates nothing but the daemon it's told to probe.

2. **changeover-warmswap** (rust-extend `~/wintermute/rollout/`) —
   add an overlap restart strategy: start successor, wait for it to
   subscribe + acquire the daemon's agorabus claim, then stop predecessor.
   Uses the existing `ClaimAcquire`/`ClaimRelease` lease so exactly one
   instance is the active holder. Falls back to the current hard restart
   when a daemon declares no claim key.

3. **changeover-autoapply** (rust-extend `~/wintermute/rollout/`) —
   gate `rollout apply --auto` on a fresh `changeover probe` proof
   (window under threshold, default 0 events lost) recorded per daemon.
   Without a current green proof, auto refuses and falls back to
   plan-only. Wires into the self-review fleet-staleness finding so the
   stale voice daemons roll unattended once proven safe.

## Order

probe → warmswap → autoapply.

- **probe ships first and independently** — it needs nothing new and tells
  us whether warm-swap is even necessary (maybe systemd restart is already
  <50ms with zero loss for some daemons).
- **warmswap** depends on nothing in probe at the code level but is only
  worth building if probe shows a lossy window.
- **autoapply** depends on both: it records probe proofs and only flips the
  auto-gate for daemons that warmswap can handle.

## Open questions

- **Produce-side hardware gap.** wm-audio holds the ALSA capture device;
  two processes can't both own the mic. Warm-swap closes the *bus* window
  but not the brief moment where neither old nor new wm-audio holds the
  device. Is mic-handoff a separate PRD (SCM_RIGHTS fd passing?) or do we
  accept a sub-100ms audio gap on wm-audio specifically? Left out of the
  fleet until probe quantifies it.
- **Claim key per daemon.** Warm-swap needs each daemon to declare a claim
  path (e.g. `agorabus://daemon/wm-stt`). Add to `fleet.toml` recipes, or
  derive from the unit name? Drafted to derive-with-override.
- **Proof freshness.** How stale can a `changeover probe` proof be before
  autoapply distrusts it? Tie to the daemon's binary hash (proof invalid
  once the exe changes) vs a wall-clock TTL? Drafted to hash-bound.
