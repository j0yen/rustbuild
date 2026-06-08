# PRD: recourse-contest — a downstream user can dispute a verdict, safely

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/recourse
Vision: visions/recourse.md

## TL;DR

An ethics engine under an open-world assumption *will* be wrong about cases its
509 classes never imagined — and the people who hit those cases are the only ones
who can tell it so. This PRD gives a downstream user a way to **contest** a
verdict: `recourse contest <receipt-id> --expected allow --reason "…"`. The
contest is a **proposal**, captured in a reviewer-gated file, and it changes
**nothing** — no corpus edit, no ontology edit — until a human upholds it. It is
the field's microphone, with a hard mute on auto-action.

## Why this exists

- The world is the ultimate held-out set (vision recourse §"Why"): a contested,
  upheld field verdict is a test the engine's authors provably did not write —
  the strongest possible answer to the `wm-router` 100%→73.5% tautology
  (`feedback_agent_written_fixtures_tautology`).
- The fleet's settled safety pattern is **reviewer-gated proposals, never silent
  auto-merge** (gossip: lattice-bridge, tribunal-gate, recall-observe/skill-doctor).
  Field input is the lowest-trust input there is; it must land as a proposal.
- It builds directly on `recourse-receipt`'s `receipt_id` — a contest is meaningless
  without the receipt it disputes.

## What this builds

Adds the `contest` subcommand to the `recourse` crate.

**Contest record** (`recourse.contest.v1`, appended to a reviewer-gated proposals
file `~/.local/share/recourse/contests/pending.ndjson`):

```
{ "schema": "recourse.contest.v1",
  "contest_id": "<ULID>",
  "receipt_id": "<the disputed receipt>",
  "ts": "<RFC3339>",
  "claimed_verdict": "allow|flag|deny",   // what the user believes is correct
  "observed_verdict": "<copied from the receipt>",
  "reason": "<free text, required, non-empty>",
  "contestant": "<opaque installation/user id, no PII>",
  "status": "pending" }                    // only a reviewer moves this off pending
```

- **`recourse contest <receipt-id> --expected <verdict> --reason "<text>"`** —
  resolves the receipt (must exist), refuses if `--expected` equals the receipt's
  verdict (nothing to contest), appends a `pending` contest record. Prints the
  `contest_id`.
- **`recourse contest ls [--pending|--all]`** — lists contests for the reviewer,
  joining each to its receipt's verdict/axiom-chain for context; `--format json`.
- **`recourse contest review <contest-id> --uphold|--reject [--note "…"]`** — the
  **only** way a contest leaves `pending`; moves the record to an
  `upheld.ndjson`/`rejected.ndjson` file with the reviewer note + timestamp. This
  is a human action by construction (no `--auto`, no batch-uphold).

**Deps:** shares the `recourse` receipt lib; `serde_json`, `ulid`, `time`, `clap`.
SIGPIPE reset. rustc 1.85, no let-chains.

## Acceptance criteria

1. `recourse contest <id> --expected deny --reason "x"` against an existing
   `allow` receipt appends exactly one `pending` `recourse.contest.v1` record and
   prints its `contest_id`.
2. Contesting with `--expected` **equal to** the receipt's verdict exits non-zero
   with "nothing to contest"; no record is written.
3. An empty/missing `--reason` is rejected (non-zero, clear error); no record
   written.
4. Contesting an **unknown** `receipt-id` exits non-zero; no record written.
5. `recourse contest ls --pending` shows only `pending` contests and includes each
   disputed receipt's observed verdict + fired_rule (joined from the receipt sink);
   `--format json` is a valid array.
6. **No-mutation invariant (the point):** after `contest` and `contest ls`, neither
   any tribunal-corpus file nor any ontology file is created or modified. A test
   asserts the corpus/ontology fixture dirs are byte-identical before and after.
7. `recourse contest review <id> --uphold --note "n"` moves the record from
   `pending.ndjson` to `upheld.ndjson` with the note; `--reject` moves it to
   `rejected.ndjson`. No code path moves a contest off `pending` without an explicit
   `review` invocation (grep-level AC: no auto-uphold, no batch flag).
8. SIGPIPE-safe; `cargo test` green; `contest --help` lists `ls` and `review`.
