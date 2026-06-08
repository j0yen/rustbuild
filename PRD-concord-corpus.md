# PRD: concord-corpus

Status: Draft v0.1
build_priority: high
build_target: rust-cli
Vision: visions/concord.md

## TL;DR

You cannot bridge a divide you can only see one side of. Most "research" on a
contested question collapses into one stance's framing — the sources you already
agree with, ranked by an engine optimized for engagement. `concord-corpus` is the
foundation of the `concord` workspace: given a contested claim, it gathers
*perspective-diverse* sources, tags each by the stance it argues, deduplicates
near-identical framings, scores credibility, and emits one structured `Corpus`
that every later concord stage consumes. This PRD creates the `~/wintermute/concord`
cargo workspace and the `concord` binary. It is pure plumbing + structure — no LLM
reasoning yet — so it ships fully cloud-build-safe.

## Why this exists

- **Evidence — the gathering shape is already proven here.** The `/deep-research`
  skill (`~/.claude/skills/deep-research/`) already does multi-source web search →
  fetch → adversarial verification → cited synthesis. `concord-corpus` reuses that
  shape but inverts the objective: instead of converging on one answer, it
  deliberately *spreads* across stances. We don't reinvent fetch/search; we add
  stance-diversity and dedup on top.
- **Evidence — the homeward/relay ingest pattern is the house idiom.**
  `~/wintermute/homeward` and the just-drafted `relay-directory`
  (`PRD-relay-directory.md`) both ingest open data into one normalized local
  store and query it. `concord-corpus` mirrors that ingest→normalize→query shape,
  pointed at argument sources rather than civic records.
- **Evidence — the ground is clear.** Per `visions/concord.md`, recall confirms
  no prior conflict/dialogue tooling exists on this laptop (the only "conflict"
  memories are git merge conflicts), so this is a new workspace, not an extend.
- **Why a foundation crate first:** every other concord component
  (steelman, cruxes, bridge) consumes a `Corpus`. If the structure isn't stable
  and tested first, the LLM stages have nothing honest to reason over.

## What this builds

- New cargo workspace at `~/wintermute/concord` (Rust 2021, `rust-toolchain.toml`
  pinned to 1.85 per `project_rust_toolchains_multi`), with a `concord-corpus` lib
  crate and a thin `concord` binary (`[[bin]]`) hosting subcommands
  (`concord corpus ...` to start). SIGPIPE reset as first line of `main()` per
  `self_sigpipe_panic_toolkit`.
- **Schema** (`concord-corpus` lib): a normalized `Source` model — title, url,
  publisher, retrieved-at, excerpt/full-text, a `Stance` tag (an open enum:
  `Supports | Opposes | Mixed | Background`, plus a free-text `stance_label` for
  the specific position), a `credibility` score, and provenance. A `Corpus` is
  `{ claim: String, sources: Vec<Source>, stances: Vec<StanceSummary> }`,
  serde-serializable to JSON (the wire format every later stage reads).
- **Gather**: a `SourceGatherer` trait + a default implementation that drives
  web search/fetch. The gatherer is injected behind the trait so tests run
  against a `FixtureGatherer` (a directory of canned HTML/JSON sources) — **no
  outbound network in the test path**, which is both the privacy guarantee and
  the cloud-build-safety guarantee.
- **Stance tagging**: a deterministic, rule-based first pass (lexical/structural
  cues — quoted claims, "critics say", citation polarity) that assigns a
  provisional `Stance`. LLM-based stance refinement is explicitly *out of scope*
  here and lands as a hook for `concord-steelman` to sharpen later. Keeping the
  v1 tagger rule-based keeps this PRD LLM-free and cloud-buildable.
- **Dedup**: near-identical framings (syndicated copy, quote-only reposts)
  collapsed via shingled-token Jaccard similarity over normalized text, above a
  tunable threshold. Deterministic and unit-testable on fixtures.
- **Credibility**: a transparent, rule-based score (has-author, has-date,
  primary-vs-secondary, domain-class) — *not* a trust oracle; documented as
  heuristic and surfaced so a human can override. No ML model.
- **CLI UX**: `concord corpus build "<claim>" [--out corpus.json] [--fixtures <dir>]`,
  `concord corpus show <corpus.json>` (human table of sources by stance),
  `concord corpus stats <corpus.json>` (stance balance, dedup count).

## Acceptance criteria

1. `cargo build` and `cargo test` are green in a fresh `~/wintermute/concord`
   workspace; `concord --help` lists the `corpus` subcommand. MSRV 1.85, no
   let-chains (per `self_recall_baseline_gate_red`).
2. `concord corpus build "<claim>" --fixtures tests/fixtures/<case>` produces a
   `Corpus` JSON validating against the documented schema, with every `Source`
   carrying a `Stance` and a provenance record.
3. The dedup pass collapses a fixture set containing 3 syndicated copies of one
   article into a single retained `Source` (asserted in a unit test on the
   fixture corpus).
4. Stance tagging assigns the correct `Stance` to ≥80% of a hand-labelled,
   **independently authored** fixture set (≥15 sources spanning ≥2 stances). The
   fixture labels live in a checked-in `expected.json` written before the tagger,
   not generated by it (avoids the tautology trap per
   `feedback_agent_written_fixtures_tautology`).
5. No code path in the default `build` reaches the network unless a real gatherer
   is explicitly configured; the test suite runs with networking unavailable and
   passes (privacy + cloud-build guarantee). A test asserts the `FixtureGatherer`
   path performs zero outbound connections.
6. `main()` calls `sigpipe::reset()` first; `concord corpus show <file> | head`
   does not panic.

## Out of scope (later PRDs)

- LLM steelmanning of stances → `concord-steelman`.
- Crux vs misunderstanding classification → `concord-cruxes`.
- Balanced synthesized brief → `concord-bridge`.
- Message de-escalation → `concord-deescalate` (independent of the corpus chain).
