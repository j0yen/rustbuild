# PRD: mqo-trace-harvest — grow the golden set from real binds, without making it tautological

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-trace-harvest (NOT AtScaleInc; consumes mqo-bench / mqo-demo-runner outputs)

## TL;DR

`mqo-bench`'s golden set starts hand-authored at ~30 questions — enough to prove
the harness, not enough to be convincing. The vision's open question asks whether
the set should be *trace-harvested* from real agent activity. `mqo-trace-harvest`
makes that growth real and *safe*: it ingests `mqo-demo-runner` transcripts and
`mqo-bench run` binder I/O, extracts `{nl_question, bound_mqo}` candidate pairs,
dedupes them against the existing golden set, and emits them as **candidates for
human review** — never auto-accepted. A harvested bind is a hypothesis about the
right answer, not ground truth; auto-trusting it would make the benchmark grade
the binder against its own output.

## Why this exists (Phase-1 evidence)

- The vision's open questions name this directly: *"Where do mqo-bench's golden
  NL→MQO pairs come from? Hand-authored… or harvested from real agent traces?
  v1 default: hand-authored, ~30 questions."* This PRD is the harvester that
  unblocks growth past v1 (visions/atscale-ai-strategy.md).
- The tautology risk is documented box experience, not hypothetical:
  [[feedback_agent_written_fixtures_tautology]] — when an agent writes both the
  rules and the test fixtures, recall/accuracy claims prove nothing; a reviewer
  must validate on an independent held-out set (the wm-router safety regression,
  100%→73.5% on a real held-out set, is the cautionary case). A harvester that
  auto-accepted its own binds would reproduce exactly that failure.
- The upstream producers exist as of this fleet: `mqo-demo-runner` emits ordered
  `transcript.json` with each bind, and `mqo-bench run` invokes a binder and
  captures its predictions. Both are structured JSON this tool can parse — no new
  capture plumbing is invented.
- `recall` already ships an offline near-duplicate detector (cosine BGE,
  `recall-memdedup`, per the box changelog) — the dedupe step reuses that
  approach rather than inventing a new similarity check.

## What this builds

A standalone Rust CLI `mqo-trace-harvest`:

- `mqo-trace-harvest extract --from <transcript-or-runlog>...` → parse
  `mqo-demo-runner` transcripts and/or `mqo-bench run` outputs, pull every
  `{nl_question, bound_mqo}` pair, and emit them as candidate golden entries with
  a `provenance` field (source file, tool, timestamp passed in — never minted
  internally) and `status: candidate`.
- `mqo-trace-harvest dedupe --candidates <c.json> --against <golden.json>` →
  drop candidates whose question is a near-duplicate (cosine over BGE embeddings,
  threshold configurable) of an existing golden question; report what was dropped
  and why (no silent truncation, cf. dropped-coverage logging discipline).
- `mqo-trace-harvest review --candidates <c.json>` → emit a human-review worksheet
  (markdown) listing each candidate's question + bound MQO + provenance, with an
  explicit accept/reject column. Output is for a human; nothing is promoted to
  the golden set by this tool.
- Hard invariant: no subcommand writes into a golden set file. Promotion is a
  separate, human-gated step. `--help` and the README state this prominently.
- `serve` subprocess mode exposing `extract`/`dedupe` as `mqo-mcp-server` tools.
- Fixtures under `fixtures/` with a sample demo-runner transcript, a sample
  bench run log, an existing golden set, and the expected candidate + dedupe
  outputs.

## Acceptance criteria

1. `extract` parses a sample `mqo-demo-runner` transcript and a sample
   `mqo-bench run` log and emits candidate pairs each carrying a `provenance`
   source reference and `status: candidate`.
2. No subcommand mutates or writes a golden-set file; an attempt path does not
   exist (verified by the absence of a write target flag and a test asserting the
   golden fixture is byte-unchanged after a full run).
3. `dedupe` removes a candidate that is a paraphrase of an existing golden
   question (no lexical overlap required) and reports it in a `dropped[]` list
   with the matched golden id and similarity score.
4. `dedupe` keeps a genuinely new question and reports it in `kept[]`.
5. `review` emits a markdown worksheet with one row per candidate and an explicit
   unfilled accept/reject column.
6. Timestamps/provenance are taken from input or passed-in args, never generated
   internally (so output is reproducible and honest about origin).
7. `serve` answers an `mqo-mcp-server` tool call over stdin/stdout.
8. Determinism: identical inputs → identical candidate + dedupe output; `--help`
   documents every subcommand/flag; tests run offline and cluster-free.

## Non-goals

- Does **not** add to the golden set. It proposes; a human disposes. This
  boundary is the whole safety argument and is non-negotiable.
- Not a live MCP tap — it reads already-produced transcripts/logs, it does not
  intercept the running server. (A live trace sink is a possible follow-on.)
- No quality judgement of a bind's correctness — that is what human review (and
  ultimately `mqo-bench` against a *held-out* set) is for.
