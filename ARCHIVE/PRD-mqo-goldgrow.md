# PRD: mqo-goldgrow — human-gated curation of harvested golden-set candidates

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-goldgrow (NOT AtScaleInc; keystone of Fleet 4 — the LEARN pillar)

## TL;DR

`mqo-trace-harvest` (shipped 2026-06-16) emits *candidate* `{nl_question,
bound_mqo}` pairs mined from agent transcripts — and deliberately never
auto-accepts them, because a harvested bind is a hypothesis, not ground truth.
But nothing on the box turns a reviewed candidate into a real golden entry:
the harvester's output flows nowhere. `mqo-goldgrow` is the missing curation
step — it presents candidates one at a time, records a human accept/reject
verdict with a reason, appends *accepted* candidates to `mqo-bench`'s golden
set with full provenance, and keeps a durable rejection ledger so a rejected
candidate is never re-surfaced. It is the only sanctioned path by which the
benchmark's ground truth grows, and it exists precisely so growth never
becomes tautological.

## Why this exists (Phase-1 evidence)

- `mqo-trace-harvest` shipped today (CLAUDE_SELF changelog 2026-06-16: "harvest
  NL→MQO candidates from agent traces for human-gated golden-set growth") and
  its repo is on disk at `~/wintermute/mqo-trace-harvest/` — but its emitted
  candidates have no consumer; the "human-gated" half of its own description is
  unbuilt.
- `mqo-bench` shipped (binary on PATH, `~/.local/bin/mqo-bench`) and owns the
  golden set the whole MEASURE pillar scores against — but its golden fixture is
  hand-authored and static; there is no audited way to extend it.
- The anti-tautology constraint is load-bearing here
  ([[feedback_agent_written_fixtures_tautology]]): when an agent writes both the
  rule and the fixture, recall/accuracy numbers prove nothing. The held-out-set
  discipline (validate on an independent set a human signed off on) is exactly
  what this tool enforces — accepted candidates carry a human verdict, not an
  agent's self-assertion.
- `mqo-decision-log` shipped (binary on PATH) and `query` emits the durable
  record series; harvested candidates ultimately trace back to those records, so
  goldgrow can cite a candidate's originating session for the reviewer.

## What this builds

A standalone Rust CLI `mqo-goldgrow`:

- `mqo-goldgrow review --candidates <harvest.jsonl> [--golden <golden.json>]` →
  iterates harvest candidates, skipping any that exact- or near-duplicate an
  existing golden entry or a prior rejection (reuse the `recall`-style
  near-duplicate check the harvester already documents), and emits each
  remaining candidate as a `{candidate, originating_session?, nearest_golden,
  similarity}` review item (JSON). Non-interactive: emits the work-list; a human
  or wrapper supplies verdicts in the next step.
- `mqo-goldgrow accept --candidate <c.json> --reviewer <id> [--note <s>]` →
  appends the candidate to the golden set with a provenance stamp
  `{source: "harvested", reviewer, ts, originating_session?}`; refuses if it
  duplicates an existing entry (idempotent, no double-merge).
- `mqo-goldgrow reject --candidate <c.json> --reviewer <id> --reason <s>` →
  appends to an append-only rejection ledger so `review` filters it out forever;
  never mutates the golden set.
- `mqo-goldgrow stats [--golden <g.json>]` → reports golden-set composition:
  count by `source` (hand-authored vs harvested), per-model spread, and the
  accept/reject ratio over the ledger — so a reviewer sees whether harvested
  entries are diluting a held-out set.
- Provenance is mandatory: an `accept` with no `--reviewer` is a hard error
  (no anonymous ground truth). The golden file stays valid `mqo-bench` input —
  goldgrow only *adds* provenance fields the bench tolerates.
- `--mock`/fixture mode with a bundled sample harvest file, golden file, and
  rejection ledger so `review`/`accept`/`reject`/`stats` are testable offline.
- `serve` subprocess mode exposing `review`/`accept`/`reject`/`stats` as
  `mqo-mcp-server` tools.

## Acceptance criteria

1. `review` over a fixture harvest emits only candidates that are neither in the
   golden set nor the rejection ledger; a candidate matching an existing golden
   entry above the similarity threshold is excluded and named.
2. `accept --reviewer alice` appends exactly one golden entry carrying
   `{source:"harvested", reviewer:"alice", ts}`; the file remains valid
   `mqo-bench` golden input (bench loads it without error).
3. `accept` with no `--reviewer` exits non-zero and writes nothing (provenance
   is mandatory).
4. A second `accept` of the same candidate is a no-op success (idempotent — no
   duplicate golden entry).
5. `reject --reason "wrong grain"` appends to the rejection ledger and the same
   candidate is absent from the next `review` run; the golden set is byte-
   unchanged by a reject.
6. `stats` reports counts by `source` and the accept/reject ratio matching
   hand-computed values on the fixtures.
7. Determinism: `review` ordering and `stats` output are stable across runs for
   fixed inputs (no wall-clock or map-iteration nondeterminism).
8. `serve` answers the curation tool calls; `--help` documents every flag; all
   tests run cluster-free against bundled fixtures.

## Non-goals

- Does not *generate* candidates — that is `mqo-trace-harvest`'s job; goldgrow
  only curates what harvest emits.
- Does not score binding accuracy — that is `mqo-bench`; goldgrow only grows the
  set bench scores against.
- Never auto-accepts. There is no `--auto` flag; ground truth requires a named
  human reviewer by design.
