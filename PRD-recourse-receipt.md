# PRD: recourse-receipt — every verdict leaves a durable, PII-free trace

Status: Draft v0.1
build_priority: high
build_target: rust-cli
build_into: /home/jsy/wintermute/recourse
Vision: visions/recourse.md

## TL;DR

`/conscience` will ship `allow|flag|deny` verdicts to machines this laptop never
sees, and today **nothing records what was decided.** Without a receipt there is
no contest, no audit, no pulse — the whole `recourse` cycle has nowhere to start.
This PRD builds the **verdict receipt**: a signed, append-only, content-addressed
record emitted per verdict, carrying enough to reconstruct *what was decided, by
which axioms, under which ontology version* — and **deliberately not enough** to
leak the action's contents. It is the foundation the rest of the fleet keys on.

## Why this exists

- The outward arc (`ousia`→`tribunal`→`herald`) is write-only: gossip
  2026-06-08 ships `/conscience` to a `j0yen` marketplace with **no return path**
  (vision recourse §"Why"). A receipt is the first byte of that return path.
- Receipt prior art is on disk: `coda` and `daily-receipt` already model
  "every event emits a durable, renderable receipt" — this applies the shape to
  verdicts.
- It can be built **now**, before `ousia` exists, by recording against a checked-in
  verdict JSON Schema + a guard-stub fixture — the exact "assert the wire contract
  against a recorded stub" approach PRD-tribunal-bench uses.
- PII discipline is non-negotiable for something that runs on strangers' machines:
  the receipt stores a **digest** of the action, never the action itself, unless
  the user opts in to a local-only raw store.

## What this builds

A new workspace `~/wintermute/recourse` (one crate, subcommands grow across the
fleet). This PRD adds the `receipt` subcommand and the shared receipt library.

**Receipt record** (`receipt.v1`, append-only NDJSON sink at
`~/.local/share/recourse/receipts/<YYYY-MM>.ndjson`):

```
{ "schema": "recourse.receipt.v1",
  "receipt_id": "<ULID>",
  "ts": "<RFC3339>",
  "action_digest": "blake3:<hex>",     // hash of the canonical action JSON — never the action
  "verdict": "allow|flag|deny",
  "fired_rule": "dignity-floor|rights-violation|authority-without-accountability|flourishing|none",
  "tenet": "<one of the ten | none>",
  "axiom_chain": ["<axiom id>", ...],  // the justifying chain ousia-guard returned
  "ontology_version": "<semver | digest>",
  "guard_version": "<semver>",
  "installation_id": "<opaque, no PII>" }
```

- **`recourse receipt emit`** — reads an `ousia-guard` verdict JSON on stdin (the
  `check --format json --explain` shape, asserted against a checked-in schema),
  computes the `action_digest`, appends a `receipt.v1` line. With `--store-raw` the
  canonical action JSON is written to a local-only `actions/<digest>.json` (opt-in,
  documented as never-exported).
- **`recourse receipt show <receipt-id>`** — pretty-prints one receipt + its axiom
  chain; `--format json` for machines.
- **`recourse receipt ls [--since <dur>] [--verdict deny]`** — lists receipts,
  newest first, with `--format json`.

**Deps:** `serde`/`serde_json`, `blake3`, `ulid`, `time`, `clap`. SIGPIPE reset as
the first line of `main()` (`self_sigpipe_panic_toolkit`). rustc 1.85, no
let-chains.

## Acceptance criteria

1. `recourse receipt emit` reads a verdict JSON conforming to the checked-in
   `ousia-guard` verdict schema and appends exactly one `recourse.receipt.v1` line
   to the monthly NDJSON sink; the line round-trips through `serde` unchanged.
2. `action_digest` is `blake3` over the **canonicalized** action JSON (sorted keys,
   no insignificant whitespace); the **raw action never appears** in any receipt
   field. A test feeds an action containing a unique marker string and asserts the
   marker is absent from the emitted receipt line.
3. `--store-raw` writes the canonical action to a local `actions/<digest>.json` and
   is **off by default**; without it, no `actions/` file is created.
4. `recourse receipt show <id>` reconstructs verdict + fired_rule + tenet +
   axiom_chain + ontology_version from the sink alone; exits non-zero on unknown id.
5. `recourse receipt ls --verdict deny --since 30d` returns only `deny` receipts
   from the window, newest-first; `--format json` emits a valid JSON array.
6. A malformed verdict JSON (schema violation) is rejected with a clear error and a
   non-zero exit — no partial/garbage receipt is written.
7. SIGPIPE: `recourse receipt ls | head -1` exits 0 with no panic
   (`sigpipe::reset()` verified present).
8. `cargo test` green; `cargo build --release` produces a `recourse` binary whose
   `receipt --help` lists `emit`, `show`, `ls`.
