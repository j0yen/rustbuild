# PRD: careen-ledger — was-it-worth-it accounting for sweeps

Status: Draft v0.1
build_target: rust-cli
Vision: visions/careen.md
Repo: j0yen/careen-ledger (PUBLIC)
Depends on: PRD-careen-sweep.md

## TL;DR

Careening a target dir reclaims disk now but forces the *next* build to
recompile what was pruned. A naive `cargo clean` always nets negative on
a repo you're about to rebuild. `careen-ledger` records each sweep's
reclaimed bytes AND the rebuild cost it provoked, so the fleet learns
which repos are worth careening (cold, rarely rebuilt) versus which just
thrash the compiler (hot, rebuilt daily). It is the feedback loop that
makes careen smarter than `cargo clean`.

## Why this exists

- `recall` and `wintermute-brain` are the two 13G giants — but they are
  also among the most actively rebuilt repos (self-review 2026-06-16:
  heavy /build day, top write prefixes were `target/debug/incremental`,
  `.fingerprint`, `deps`). Careening a hot repo may reclaim 9G and then
  pay it back in a full recompile minutes later. Without measurement,
  careen-guard can't tell a good sweep from a wasteful one.
- careen-sweep emits a per-sweep JSON summary (`removed_bytes`, classes)
  but has no memory and no rebuild-cost view. The signal needed —
  "did the next build have to redo this?" — is only observable *after*
  the sweep, across time. That is a ledger's job, not a one-shot tool's.

## What this builds

A Rust CLI `careen-ledger` with an append-only JSONL store
(`~/.local/state/careen/ledger.jsonl`).

**Subcommands:**

- `record <sweep-summary.json>` — append a sweep event: repo, ts,
  removed_bytes, classes. Returns a ledger entry id.
- `attribute <id>` — after the next build of that repo, attribute its
  rebuild cost to the sweep: recompiled-crate count and/or wall-time
  delta vs the repo's rolling baseline. Cost source: parse cargo's
  `--timings` / `cargo build --message-format=json` compiler-artifact
  count, or a caller-supplied `--rebuilt-crates N --wall-secs S`. Writes
  a paired `attributed` record.
- `verdict <repo>` — roll up the repo's history into a net signal:
  `reclaimed_bytes_total`, `rebuild_cost_total`, and a
  `worth_careening: bool` recommendation (cold repos with high reclaim /
  low rebuild → true; hot repos that pay it all back → false). Output
  JSON; careen-guard can read it to skip not-worth-it repos.

**Design:** append-only (Hard rule echo from gossip discipline — never
rewrite history); each `record` is immutable, `attribute` adds a new
paired line keyed by id, `verdict` is a pure read-fold. No deletion.

**Crates:** `clap`, `serde`/`serde_json`, `anyhow`, `directories` (for
state path).

**NOT in scope:** triggering sweeps (careen-guard); the sweep itself
(careen-sweep). Ledger only observes and scores.

## Acceptance criteria

1. `careen-ledger record <summary.json>` appends one immutable JSONL
   line carrying repo, ts, removed_bytes, and classes, and prints the
   new entry id.
2. `attribute <id> --rebuilt-crates N --wall-secs S` appends a paired
   record keyed by that id with the rebuild cost; the original record is
   left byte-for-byte unchanged (append-only proven by a test).
3. `verdict <repo>` folds all paired records for the repo into JSON with
   `reclaimed_bytes_total`, `rebuild_cost_total`, and a boolean
   `worth_careening`.
4. The `worth_careening` rule is deterministic and documented: a repo
   whose cumulative reclaimed bytes greatly exceed its rebuild cost
   recommends true; a repo that recompiles everything it reclaimed each
   time recommends false. Two fixtures (a cold-win repo and a
   hot-thrash repo) classify oppositely.
5. `attribute` can also derive rebuild cost from a cargo
   `--message-format=json` build log (counting `compiler-artifact`
   lines) when `--build-log <path>` is given instead of explicit counts.
6. The ledger never rewrites or deletes a prior line; corruption of one
   line (bad JSON) is skipped with a warning on read, not a crash.
7. `verdict` on a repo with no history exits cleanly with an
   `insufficient-data` JSON verdict, not an error.
8. State path is created on first `record` if absent; `--ledger <path>`
   overrides the default for testing.
