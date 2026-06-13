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

## Fleet 2 — activation: the warm-swap is built but dormant (drafted 2026-06-13)

`/dream extend changeover`, bare interactive seed. Fleet 1
(`probe`/`warmswap`/`autoapply`) all **shipped** 2026-06-13 — yet the
journal that same evening *still* parks `fleet-binary-staleness` on
"daemon restarts drop subscribers; requires explicit approval." The
machinery exists and the fleet is still stale. This pass found why: the
warm-swap is **decorative**, switched off at three points, and nothing
ever closed the loop. Same spirit as [[quicken]] — "a built primitive
that never came alive isn't built" — but the axis here is the rollout
*actuator* and its producer-side prerequisites, not kernel primitives.

Caught live this pass (verbatim probes, 2026-06-13):

- **No daemon acquires a claim.** `warmswap.rs` waits for the *successor*
  to appear as an `agorabus claim list <path>` holder before stopping the
  predecessor. But `grep -rl claim_acquire` across
  `~/wintermute/wintermute-{audio,stt,tts}/src` → **0 files**;
  `wintermute-dialog`'s two `claim` hits are FSM-internal (transcript
  claiming), not the agorabus lease. So no successor ever becomes a
  holder → `wait_for_claim_holders` times out → warm-swap falls back to
  the exact hard restart it was built to replace. All four daemons depend
  on `agorabus = { path = "../agorabus" }`, whose client already exposes
  `claim_acquire` — the API is right there, unused.
- **No proof ledger exists.** `~/.config/rollout/proofs.json` is absent.
  `rollout apply --auto` (v0.7.0) refuses every daemon without a fresh
  green proof, and nothing has ever run `changeover probe` →
  `rollout record-proof`. The gate is permanently red because it was
  never seeded.
- **The cron never fires apply.** `self-review/SKILL.md` (lines 303, 852,
  864) makes "never run `rollout apply` autonomously" an *immutable*
  guardrail, justified by the *hard-restart* reality warm-swap was built
  to eliminate. The reconciliation is already drafted as the **blocked**
  `PRD-rollout-selfreview-apply.md` (classifier-blocked pending jsy's
  explicit approval) — Fleet 2 does not redraft it; it depends on it.

The fix is not more mechanism — it is **producer-side participation plus
seeding plus activation**: teach the daemons to hold their claim, mint
the first green proof, then turn the autonomous loop on with live
post-swap verification that voice actually survived.

### Components (Fleet 2)

- **changeover-claim-guard** (rust-extend `~/wintermute/agorabus`):
  an RAII `ClaimGuard` on the client — `client.hold_claim(path, ttl)`
  acquires the lease, auto-renews before the TTL, and releases on drop
  (and on SIGTERM via a shutdown hook). The reusable primitive the four
  daemons and any future peer share. Ships first; pure lib + fixture
  tests, no daemon surgery.

- **changeover-daemon-claims** (mixed rust-extend → the four
  `wintermute-{audio,dialog,stt,tts}` crates): each daemon, on startup
  after bus connect, holds `agorabus://daemon/<unit>` via the guard and
  releases on graceful shutdown. This is the load-bearing change that
  makes warm-swap real — without it the proof can never go green. The
  pattern is identical across all four; /build may fan out per-crate.
  Depends on claim-guard.

- **changeover-proof-seed** (rust-extend `~/wintermute/rollout`):
  a `rollout prove --daemon <unit>` convenience that runs `changeover
  probe`, feeds its JSON to `record-proof`, and a low-frequency
  systemd-user timer that keeps the ledger fresh (re-proving when a
  daemon's binary hash changes). Mints the first green entry per daemon.
  Depends on daemon-claims (the proof only goes green once a real
  warm-swap loses zero events).

- **changeover-activate** (rust-extend `~/wintermute/rollout` + config):
  the systemd-user timer that runs probe → record-proof → `apply --auto`
  (warm-swap only, turn-aware quiet window from v0.5.0), gated on
  `ROLLOUT_AUTO_ENABLED=1`, followed by **live post-swap verification** —
  assert all four daemons re-hold their claims and a synthetic voice turn
  round-trips, recording a receipt. Depends on proof-seed and on the
  blocked `PRD-rollout-selfreview-apply.md` reaching jsy's approval.

### Order (Fleet 2)

claim-guard → daemon-claims → proof-seed → activate.
claim-guard and daemon-claims are the producer half (make warm-swap
real); proof-seed mints the green ledger; activate turns the loop on.
activate must not ship before `PRD-rollout-selfreview-apply.md` is
unblocked — until then it lands the timer in a `ROLLOUT_AUTO_ENABLED=0`
dormant posture (proves itself in `--dry-run`, never restarts).

### Open questions (Fleet 2)

- **Mic-handoff still unsolved** (carried from Fleet 1): wm-audio owns the
  ALSA capture device; claim-guard closes the bus window but two
  processes still can't both hold the mic. Accept a sub-100ms audio gap on
  wm-audio specifically, or a separate SCM_RIGHTS fd-passing PRD? Left out
  until `changeover probe` quantifies the audio-specific gap post-claims.
- **Claim TTL vs renew cadence**: what TTL balances "successor can acquire
  quickly after predecessor dies" against "renew traffic on a quiet bus"?
  Drafted to TTL=30s, renew at 10s, but probe should tune it.
- **Should proof-seed live in rollout or changeover?** `rollout` already
  owns `record-proof`; `changeover` owns `probe`. Drafted into rollout so
  one binary runs the whole prove cycle, but a `changeover prove` that
  shells to `rollout record-proof` is equally defensible.
