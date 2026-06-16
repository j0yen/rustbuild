# PRD: mqo-decision-log — durable, auditable record of every agent decision

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-decision-log (NOT AtScaleInc; producer for mqo-trace-harvest + mqo-scorecard trends)

## TL;DR

`mqo-agent` makes a chain of decisions per question — which pillars it chose, the
access-policy verdict, the budget consumed, the final signed credential. Those
decisions vanish when the process exits. Two existing PRDs already *assume* a
durable series exists: `mqo-trace-harvest` wants agent transcripts to mine for
golden-set candidates, and `mqo-scorecard` wants "trend deltas when given a prior
scorecard." Nothing produces that durable series. `mqo-decision-log` is the
append-only sink: every agent run writes one structured, queryable decision
record, so the agent's behavior over time is auditable and the downstream
consumers have a real source.

## Why this exists (Phase-1 evidence)

- `mqo-trace-harvest` (Fleet 2, `PRD-mqo-trace-harvest.md`) ingests "demo-runner
  transcripts and bench run I/O" to grow the golden set — but a one-shot
  transcript is not a durable, accumulating series; there is no log that
  *persists* across real agent runs for it to harvest from.
- `mqo-scorecard` (Fleet 2, `PRD-mqo-scorecard.md`) renders "trend deltas when
  given a prior scorecard" — trends require a persisted time series of decisions,
  which nothing on the box currently writes.
- The vision's end-state #5 requires "every AI decision over governed data is
  auditable linked data"; `rosetta-prov`/`rosetta-credential` already emit
  per-answer PROV-O/VCs (verified shipped, CLAUDE_SELF changelog 2026-06-16), but
  there is no *log* tying a session's sequence of those credentials together into
  an inspectable history — the credential proves one answer, not the agent's
  conduct over a session.
- The kernel offers a provenance hook this can lean on: `provfs` stamps
  `user.prov.session` xattrs on closed-after-write files (dream Phase-1.5
  surface, `~/wintermute/provfs/`), so a decision-log file inherits session
  provenance for free where the LSM is active — to be consumed, not duplicated.

## What this builds

A standalone Rust CLI `mqo-decision-log`:

- `mqo-decision-log append --session <id> --record <record.json>` → appends one
  decision record to a durable, append-only log (JSONL). A record captures:
  `{ts, session, question, plan[], access_verdict, budget_consumed,
  pillars_fired[], outcome (answered|clarify|blocked), credential_id?}`. Append
  is the only mutation — never rewrite or delete (audit integrity).
- The record schema is shared with `mqo-agent`'s `answer.json` so the agent emits
  a record this tool ingests directly — no translation layer.
- `mqo-decision-log query [--session <id>] [--since <ts>] [--outcome <o>]` →
  filters the log; emits matching records as JSONL for piping into
  `mqo-trace-harvest`.
- `mqo-decision-log summary [--since <ts>]` → aggregate counts (questions,
  clarify rate, block rate, pillar-fire frequency, total budget consumed) as the
  JSON `mqo-scorecard` consumes for trend deltas. Computes counts only; renders
  nothing (scorecard is the presenter).
- `mqo-decision-log verify` → checks log integrity: append-only (no record
  mutated), monotonic timestamps per session, and — where `provfs` is active —
  reports the `user.prov.session` xattr on the log file as a tamper-evidence
  signal (absent ⇒ reported, never fatal).
- `--mock`/fixture mode and bundled sample log so `query`/`summary`/`verify` are
  testable without a real agent run.
- `serve` subprocess mode exposing `append`/`query`/`summary` as `mqo-mcp-server`
  tools.

## Acceptance criteria

1. `append` adds exactly one record; the log is valid JSONL and prior records are
   byte-identical after the append (append-only proven).
2. A record round-trips: an `mqo-agent` `answer.json` fixture appends without a
   translation step and `query` returns it intact.
3. `query --session <id> --outcome clarify` returns only matching records;
   `--since <ts>` filters by timestamp.
4. `summary` emits aggregate counts (clarify rate, block rate, pillar-fire
   frequency, total budget) over the filtered set, matching hand-computed values
   on the fixture log.
5. `verify` passes on an untampered log and fails (non-zero, naming the offending
   record) on a fixture whose middle record was edited.
6. `verify` reports the `provfs` session xattr when present and reports its
   absence without failing when the LSM is inactive (the current box default).
7. Determinism: `query`/`summary` output is stable across runs for a fixed log
   (timestamps come from the records, not the wall clock).
8. `serve` answers the log tool calls; `--help` documents every flag; all tests
   run cluster-free against bundled fixtures.

## Non-goals

- Not a general logging framework — it stores agent *decision* records in one
  documented schema, not arbitrary events.
- Does not render scorecards or harvest golden entries — it is the *source*;
  `mqo-scorecard` presents and `mqo-trace-harvest` mines. Keeping those separate
  avoids the tautology trap ([[feedback_agent_written_fixtures_tautology]]): the
  log records what happened; it does not judge what *should* have happened.
- Does not implement cryptographic signing of the log itself — per-answer signing
  is `rosetta-credential`'s job; a signed/Merkle-chained log is a possible
  follow-on (this v1 relies on append-only + provfs xattr for tamper-evidence).
- Not a distributed log — one local JSONL per box; fleet aggregation is a
  possible follow-on.
