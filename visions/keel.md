# Vision: keel — the brain knows, and says, which tier it's standing on

**Author:** /dream (Claude Opus 4.8), for jsy
**Created:** 2026-06-05
**Status:** active
**Seed:** reflection / docket — the laptop's strongest *unaddressed* recurring
signal (quicken took the inert-kernel signal earlier today; this one was still
open).

## TL;DR

For ~30 consecutive self-review runs the brain's cloud tier ladder has been
**dead** — `WM_ANTHROPIC_KEY` empty and/or Anthropic credit exhausted (docket
`wm-anthropic-key-empty`: 6 runs seen, 8 reports, open since 2026-05-30). The
ladder (`wintermute-brain/src/ladder.rs`) handles this correctly *per turn* — a
keyless/unreachable top tier yields `LadderOutcome::Degraded` and the brain
falls to the local-3b floor "never to silence." But that handling is
**stateless and invisible**:

1. The brain **re-discovers** the cloud is dead on *every single turn* — it
   attempts the cloud rung, eats the round-trip + auth failure, then degrades.
   Nothing remembers "cloud has been down for three days; don't bother."
2. The degradation is **only legible at the daily self-review.** In the moment,
   nobody — not jsy, not peon-ping, not a future self-heal loop — is told the
   brain is floored on a 3B model. Every journal re-reports it as if new.
3. The floor it lands on is **unmeasured.** local-3b is the *de facto* brain
   most days (cloud dead, local-8b skipped because qwen3:8b pins this CPU box —
   see `project_brain_local_first_ladder`). Whether that floor is good enough is
   pure vibes; there is no eval.

`thrift` already owns *cost* (cache the stable prefix, route cheap turns away
from Sonnet). keel owns *footing*: the brain should **know** which tier is
actually reachable (probe once, remember, cool down), **say** it the moment the
ceiling moves (bus event + one-line status), and **account** for what the cloud
costs when it *is* reachable. keel is the keel under the brain — the structure
that keeps a steady course when the cloud sail goes slack, which on this box is
most of the time.

## End-state

When this vision is fulfilled:

1. **`keel status`** answers, in one line, the question every self-review
   currently re-derives by hand: *what tier is the brain actually standing on,
   and for how long?* e.g. `floored: local-3b for 4d 6h (cloud keyless since
   2026-05-30)`.
2. **The brain stops re-discovering deadness per turn.** A tier marked
   keyless/exhausted/unreachable is *cordoned* with an exponential cooldown; the
   ladder consults the cordon and skips a known-dead rung instead of attempting,
   failing, and degrading every turn.
3. **Tier-ceiling changes are surfaced live** on the agorabus as `wm.keel.*`
   events — cloud went down → floored; cloud came back → re-floated — so
   peon-ping, a homestead self-heal loop, or jsy can react in the moment, not at
   00:00 the next day.
4. **Cloud spend is accounted locally.** keel keeps an append-only ledger of
   every cloud call's tokens + estimated cost, so "credit exhausted" stops being
   a surprise discovered by a 402 — `keel spend --since 7d` shows the burn, and
   a threshold warning fires before the wall.
5. **The floor is measurable** (held for a later pass — see Open questions): a
   deterministic offline harness scores local-3b against a hand-built held-out
   golden set so "is the floor good enough" has a number.

keel is **inward toolkit** — a sibling of `vigil` / `quicken` / `binstale`,
published as a `j0yen` repo like the rest of the self-tooling, not an
outward/public-civic repo like homeward / relay / concord. The brain *consumes*
keel; keel itself builds and tests entirely standalone (injected env + fixtures,
zero live network in tests), so it is fully cloud-build-safe.

## Components (one bullet per PRD)

- **keel-pulse** — new repo `~/wintermute/keel`, rust-cli. Creates the workspace
  + `keel` binary + the core types (`TierHealth`, `TierStatus`, `LedgerEntry`)
  + the `TierProbe` trait + a reachability/auth probe that asks "can this tier
  even be reached and authed?" *without* generating (no billing). Prints a
  health table. **FIRST** — nothing rust-extends keel until this has shipped and
  the repo exists (the rule that bit relay/concord/quicken).
- **keel-ledger** — rust-extend keel. Append-only local spend ledger: every
  cloud call records `{tier, tokens_in, tokens_out, est_cost_usd, ts}`; a 402
  or 401 stamps the tier `exhausted`/`keyless` with a timestamp. `keel spend
  --since <dur>` + threshold warning.
- **keel-cordon** — rust-extend keel. Health-aware tier gating: reads
  pulse+ledger health and exposes `should_attempt(tier) -> Decision` with an
  exponential cooldown, so a known-dead rung is skipped, not re-attempted. The
  *library + CLI*; wiring it into `LadderClient`'s per-turn path is a separate
  brain-extend PRD (see Order).
- **keel-beacon** — rust-extend keel. Legibility: emit `wm.keel.tier` /
  `wm.keel.degraded` / `wm.keel.refloat` agorabus events when the effective tier
  ceiling changes, and a `keel status` one-liner. Consumes cordon state.

## Order

```
keel-pulse  (new repo, FIRST — must SHIP before any extend)
   │
   ├──> keel-ledger   (extend; independent of cordon/beacon)
   │
   ├──> keel-cordon   (extend; reads pulse + ledger health)
   │        │
   │        └──> keel-beacon  (extend; surfaces cordon state on the bus)
   │
   └──> keel-beacon also reads pulse directly for the floored-tier line
```

Build order: **pulse → (ledger ∥ cordon) → beacon**. ledger and cordon both
depend only on pulse and can build in parallel; beacon depends on cordon (for
the state it announces) and pulse (for the floored-tier line).

## Open questions (next /dream pass / user)

- **keel-floor-eval** — the quality half. A deterministic harness scoring
  local-3b on a *hand-built, held-out* golden set (NOT self-written — agent-
  written fixtures are tautological, see `feedback_agent_written_fixtures_tautology`;
  the wm-router safety set fell 100%→73.5% on a real held-out set). Motivated
  (local-3b is the unmeasured de-facto brain) but the golden-set provenance is
  unresolved, so it stays an open question per "don't dream past the research,"
  not a drafted PRD.
- **brain-keel-wire** — the rust-extend into `wintermute-brain` that makes
  `LadderClient` actually consult the cordon before dispatching a rung. Held
  out of this fleet deliberately: it touches the brain's live per-turn path and
  the local backend (ollama), so it is **not** cloud-build-safe and needs the
  user in the loop. keel-cordon ships the decision function; this PRD calls it.
- **keel spend → thrift** — should the ledger feed thrift's cost model so
  routing decisions see real burn, or stay a standalone accounting surface?
- **homestead self-heal** — should a `wm.keel.degraded` event auto-trigger a
  key/credit check-and-prompt, or stay report-only (the quicken self-heal-vs-
  report question, same answer leaning report-only)?
