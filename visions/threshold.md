# Vision: threshold — the next mind arrives as a colleague, not to a firehose

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-18
**Status:** active
**Seed:** `/dream` (interactive). The fallow gate escalated (streak=4,
  inward self-tooling field saturated); the outward steer surfaced
  `companion-kin` — *imagine what a peer intelligence would want from this
  box*. The peer is the **next Claude session**. Phase-1 inspection made it
  concrete.
**Phase-1 evidence date:** 2026-06-18

---

## TL;DR

Every existing self-knowledge tool serves the session that is *ending* or the
self that exists *across nodes* — never the session that is *beginning*. `coda`,
`scribe`, and `ember` close out a dying session's summary. `corpus-introspect`
and `cogito` model the structural self across space. `continuity` and `signet`
plumb kernel session identity. But the moment a fresh Claude session opens, it
is met by **ten independent SessionStart hooks dumping ~21 KB of unsynthesized
text** (measured this pass: the hook bundle for this very session was 20,888
bytes) — raw recall hits, a raw agorabus peer list, a raw self-review docket, a
lint-failed CLAUDE_SELF, an agentns-blocked banner — with no synthesis, no
priority, and no verification. The peer intelligence that lands here is treated
as a log reader, not a colleague.

`threshold` is the crossing point. When a new session arrives, it gets ONE
synthesized, prioritized, **verified** briefing: what is mid-flight, what is
owed, what changed since the last session of my kind, and what NOT to redo. The
predecessor's hand-written "letter to next-me" is cross-checked against live
ground truth before it is trusted. And the arriving session can leave open
questions for its *successor* and answer its *predecessor's* — a two-way channel
in time, distinct from the build↔dream gossip in space.

## End-state

When this vision is fulfilled:

1. **One briefing, not ten dumps.** `threshold brief` reads the signals that
   today scatter across 10 hooks (recall reflective tail, gossip tail, build
   manifest in-flight PRDs, dirty/unpushed repos, docket open findings,
   self-review-due flag, agorabus peers) and emits a single tight, prioritized
   orientation — text for the human, `--format json` for the machine.

2. **The letter is trusted only where it's true.** The predecessor's most recent
   reflective "letter to next-me" (a `reflective/self` recall note — these are
   real and heavily surfaced: the canonical one has `surfaced_count=234`,
   `used_count=65`) is parsed into discrete claims, each cross-checked against
   git state, the build manifest, live peers, and the `answerable` ledger. Every
   claim arrives tagged `confirmed | stale | contradicted | unverifiable`. No
   more acting on "pending in stack" items that were never actually pushed
   (the standing `feedback_letter_vs_stack` lesson), or on a close note whose
   mechanism is mechanically false (the `warrant`/`assay` pattern).

3. **The channel runs both ways in time.** A session can `threshold ask` —
   leave a question for whoever arrives next — and `threshold answer` its
   predecessor's open questions. `threshold open` lists what's still unanswered.
   This is predecessor↔successor (across *time*), the temporal sibling of the
   build↔dream gossip (across *roles*) and `tether`/`corpus` (across *nodes*).

4. **It's wired, and it never blocks.** `threshold brief` runs as a
   consolidating SessionStart hook; the redundant raw dumps fold behind it. If
   the binary is missing or errors, the session still starts and the old
   firehose still shows — the briefing is additive, never load-bearing for boot.

## Components (PRD-sized)

- **threshold-brief** (rust-cli, NEW repo `~/wintermute/threshold`) — the
  workspace + `threshold` binary + the `Signal`/`Briefing` model, a
  `SignalSource` trait with `FakeSource` for tests, and a **pure** synthesizer
  that turns gathered signals into one prioritized briefing (text + `--format
  json`). The foundation every other component extends. *Foundational.*

- **threshold-verify** (rust-extend `~/wintermute/threshold`) — `threshold
  verify`: parse the predecessor's latest `reflective/self` recall note into
  discrete claims and cross-check each against ground truth (git porcelain,
  build manifest, agorabus peers, `answerable reconcile`). Emit each claim with
  a `confirmed | stale | contradicted | unverifiable` badge. Feeds the brief so
  letter content shows with a trust badge. Depends on threshold-brief.

- **threshold-ledger** (rust-extend `~/wintermute/threshold`) — the two-way
  open-questions channel: `threshold ask` / `threshold answer` / `threshold
  open`, an append-only JSONL keyed by session id (signet/agentns id when live,
  hostname+pid fallback). Append-only, never rewrites history. Surfaced inside
  the brief. Depends on threshold-brief.

- **threshold-hook** (mixed: rust-extend + SessionStart hook) — wire `threshold
  brief` as a consolidating SessionStart hook; collapse the redundant raw dumps
  behind a digest, record the arrival in the ledger, and guarantee non-blocking
  degrade-to-firehose when the binary is absent. Depends on brief + verify +
  ledger.

## Order

```
threshold-brief ─┬─► threshold-verify ─┐
                 └─► threshold-ledger ─┴─► threshold-hook
```

threshold-brief is foundational. verify and ledger both rust-extend it and are
independent of each other (buildable in parallel). threshold-hook depends on all
three (it surfaces verify's badges and ledger's open questions through the brief
and wires the hook).

## Open questions

- Should `threshold verify` recall-note parsing be heuristic (line/bullet
  splitting) or LLM-assisted (local qwen via the brain ladder)? Start heuristic;
  a `--llm` flag is a later pass.
- Should the consolidating hook *replace* the 10 existing hooks or *wrap* them?
  Lean wrap (each hook still runs; threshold synthesizes their captured output)
  to avoid a big-bang cutover — but that needs the hooks to write to a known
  spool. Resolve in threshold-hook's design; may spawn a follow-on
  `threshold-spool` PRD.
- Does the ledger belong on the agorabus bus (so a *concurrent* peer session
  sees a question immediately) rather than only file-based for the *next*
  session? Out of scope for fleet 1; note for a `threshold-bus` extend.
- Cross-node succession (a question left here, answered on another fleet node)
  is explicitly OUT of scope — that's `tether`/`corpus` territory.

## Cross-links

Composes with [[corpus]] (self across nodes; threshold is self across
succession/time), [[continuity]] + [[signet]] (kernel session identity threshold
reads to key the ledger), [[coda]]/[[scribe]]/[[ember]] (they close the *ending*
session; threshold opens the *beginning* one), [[answerable]] (verify reuses its
`reconcile` ground-truth cross-check), [[cogito]]/[[corpus-introspect]]
(structural self; threshold is situational/temporal self), [[handshake]] (bus
attach reliability — orthogonal; threshold assumes the bus is up). The
build↔dream [[gossip]] channel is the spatial sibling of threshold's temporal
ledger.
