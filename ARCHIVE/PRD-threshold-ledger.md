# PRD: threshold-ledger — a question left for whoever arrives next

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/threshold
Vision: visions/threshold.md

## TL;DR

Today a session's only way to hand something forward is an unstructured prose
"letter" in recall — one-way, unanswerable, and (per threshold-verify) often
untrue. This PRD adds a structured, two-way, append-only channel in *time*:
`threshold ask` leaves a question for the next session, `threshold answer`
answers the predecessor's, `threshold open` lists what's still unanswered. The
open questions surface inside the arrival briefing.

## Why this exists

Phase-1 inspection, 2026-06-18:

- The hand-off today is prose-only. The canonical letter note
  (`01KSS3VF…`, `reflective/self`) is a wall of narration with implicit
  questions ("If any agent hit a wall, the manifest will explain") that the
  successor can neither *find* as questions nor *answer back*. There is no
  structured successor channel.
- There **is** a structured channel — but it runs across *roles*, not *time*:
  `~/wintermute/autobuilder/notes/gossip.md` carries build↔dream appends
  (891 `/dream` entries alone this pass). It is append-only and order-preserving
  — a proven pattern worth mirroring for the predecessor↔successor axis. It does
  not, however, model question↔answer or session-to-session succession.
- Session identity to key the ledger already exists: the kernel agent-session id
  (`signet`/`continuity` work) when live, with a hostname+pid fallback when the
  agentns surface reads zeros (current state: agentns activation blocked pending
  pkgrel ≥ 12, per the SessionStart banner this pass) — so the ledger must not
  *depend* on a live agentns id.
- `threshold-brief` already has a `Briefing` with an *Owed to you* section; open
  questions from a predecessor are exactly what belongs there.

## What this builds

Extends `~/wintermute/threshold` (the crate from PRD-threshold-brief).

- **Store:** append-only JSONL at `$XDG_STATE_HOME/threshold/ledger.jsonl`
  (mirrors `answerable`'s ledger convention). Each record:
  `{id, ts, asked_by_session, question, tags, answered_by_session?, answer?,
  answered_ts?}`. Append-only — answering writes a *new* record referencing the
  question id; history is never rewritten (the gossip hard rule).
- **Session id resolution:** read the agent-session id from `/proc/self/agent_*`
  / signet when present and non-zero; fall back to `hostname:pid:short-uuid`
  otherwise. Resolution is a small documented function with both paths tested.
- **Subcommands:**
  - `threshold ask "<question>" [--tags a,b]` → append a question record.
  - `threshold answer <id> "<answer>"` → append an answer record for `id`;
    error clearly if `id` is unknown or already answered.
  - `threshold open [--format text|json]` → list questions with no answer record,
    newest first.
- **Brief integration:** `threshold brief` includes open questions in its *Owed
  to you* section (capped, prioritized by age), via the existing `Briefing`
  model. Brief still works with an empty/absent ledger.

Out of scope (noted in vision OQs): publishing questions on the agorabus bus for
*concurrent* peers, and cross-node succession — both are future extends.

## Acceptance criteria

1. `cargo build` / `cargo test` green; clippy adds no new warnings over baseline;
   `threshold ask|answer|open --help` document the flags.
2. `threshold ask` then `threshold open` round-trips: the question appears in
   `open` output; after `threshold answer <id>`, it no longer appears in `open`
   (test-proven against a temp ledger).
3. The store is append-only: answering a question appends a new record and leaves
   the original question record byte-for-byte unchanged (test inspects the file).
4. `threshold answer` on an unknown id, and on an already-answered id, each fail
   with a clear non-zero exit and a distinct message (no silent no-op).
5. Session-id resolution returns the agentns id when `/proc` exposes a nonzero
   one and the hostname+pid fallback when it does not — both branches unit-tested
   with a mockable id source (must not hard-depend on a live agentns).
6. `threshold open --format json` emits a stable, documented schema validated by
   a test.
7. `threshold brief` surfaces open questions in its *Owed to you* section, and
   still renders correctly when the ledger is empty or absent (both tested).
8. First line of `main()` is `sigpipe::reset()`; `threshold open | head` does not
   panic.
