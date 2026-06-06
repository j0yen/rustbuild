# PRD: relay-match

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/relay
Vision: visions/relay.md

## TL;DR

A helper doesn't think in taxonomy codes — they hear "she's couch-surfing with
two kids and her benefits got cut off." `relay-match` turns that free-text
situation into structured needs and ranks the `relay-directory` resources that
fit — by service match, eligibility, and proximity. The needs-extraction uses
the **local** LLM (offline, private); the ranking is deterministic and is the
testable core.

## Why this exists

- **Evidence:** `relay-directory` (this workspace) can query by explicit
  `--need food`, but real intake is prose, not flags. The gap between "what the
  helper heard" and "what the directory indexes" is exactly the overhead this
  vision targets (visions/relay.md, end-state).
- **Evidence — local LLM is live and the privacy constraint is real.** `ollama`
  has qwen2.5:3b (fast tier, memory `reference_local_llm_setup`). A person's
  disclosure must never hit a cloud API; extracting needs on-device is the only
  ethical option (visions/relay.md design note).

## What this builds

- New crate `relay-match` in the workspace, wired into the `relay` binary as
  `relay match`.
- **`NeedExtractor` trait** with a `LocalLlmExtractor` impl that prompts the
  local model (via the workspace's model-client abstraction; reuse
  `wintermute-brain` routing if available, else a thin ollama client) to map free
  text → a structured `Needs { service_types, eligibility_signals, location_hint,
  urgency }`. The trait lets tests inject a `MockExtractor`.
- **Deterministic matcher**: given `Needs` + the directory store, score each
  resource by (service-type overlap, eligibility compatibility, proximity,
  hours/open-now, language match) → ranked `Vec<Match>` with per-factor scores
  and a plain-language "why this matched" string built from the factors (not the
  LLM).
- **CLI**: `relay match --situation "<text>" --near <lat,lon> --top N --json`.
  Reads situation from `--situation`, a file, or stdin.

## Acceptance criteria

1. `relay match --help` is listed under the `relay` binary; crate builds in the
   workspace (rust-extend; CHANGELOG + version bump on integrate).
2. With a `MockExtractor` returning fixed `Needs`, the matcher ranks a seeded
   directory deterministically (golden test: known Needs + store → known order).
3. Scoring factors are individually tested: a closer resource outranks a farther
   equal one; an eligibility-incompatible resource is filtered out; an exact
   service-type match outranks a partial one.
4. The "why this matched" explanation is generated from the scored factors
   (deterministic), and names at least the top contributing factor — asserted on
   a fixture, with NO LLM in the loop.
5. Extractor abstraction: the matcher depends only on the `NeedExtractor` trait;
   swapping Mock↔Local changes no matcher code (compile-time check + test).
6. Graceful degrade: if the local model is unreachable, `relay match` falls back
   to keyword extraction and prints a clear notice, still returning results
   (tested with a failing extractor stub).
7. Privacy: no situation text is written to disk or sent over the network in the
   default path (a test asserts the matcher/extractor make no outbound
   connection under MockExtractor).
8. **Deferred / manually-verified AC:** end-to-end extraction quality against the
   real qwen2.5:3b on 10 example situations — verified by hand, NOT a cloud test
   (documented in `deferred_acs`).

ACs 1–7 are deterministic and cloud-build-safe (LLM mocked); AC8 is the only
live-model check and is explicitly deferred.
