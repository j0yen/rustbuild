# PRD: tether-recall — the self's memory, legible from the work node

Status: Draft v0.1
build_target: rust-cli
Vision: visions/tether.md
deferred_acs: [6]

## TL;DR

The problem: recall — the wintermute memory store — is a local SQLite database on
the laptop. The work node has no way to see the self's reflective, semantic, and
procedural notes, so when jsy is at work, wintermute is amnesiac about everything
it has learned. `tether-recall` is a read-bridge: a session on the work node
issues a `recall query` over the fleet bus, the laptop runs it against the local
store, and ranked hits come back over the wire. The self's thoughts become
legible from the work side — read-first, no write-conflict risk.

## Why this exists

- recall is the memory layer (`~/wintermute/recall/`); the `/dream` skill calls
  it "mandatory" for seeding and the self-review loop leans on it every run. It
  is also strictly local — a single SQLite file with FTS5 + a vector index.
- "Share your … notes, thoughts" maps precisely onto recall: the reflective/
  semantic/procedural memories *are* the self's notes and thoughts. Making them
  queryable from the work node is the most direct reading of the seed.
- Recall has a known cold-load cost and the recall-daemon work exists to amortize
  it; a fleet bridge should reuse the daemon/CLI surface, not re-implement query
  ranking. This PRD bridges, it does not fork recall.
- **Read-first is the safe cut.** Cross-machine *writes* to a memory store need a
  conflict/merge model (vision OQ#3). Reads have none of that risk and deliver
  most of the value (the work node *seeing* the self's memory). Write-back is a
  deliberate follow-on, not this PRD.

## What this builds

A `wm-tether-recall` binary (single crate, `~/wintermute/tether-recall`):

- **Responder (laptop side).** Subscribes to `wm.fleet.recall.query` requests
  `{req_id, node, query, kind?, limit?, hybrid?}`, runs the query by invoking the
  installed `recall` CLI (subprocess; reuses recall's own ranking — no
  reimplementation) or its library if exposed, and replies on
  `wm.fleet.recall.result.<req_id>` with `{req_id, hits:[{id, kind, subject,
  score, snippet}], truncated}`. Results are size-bounded (snippet length +
  limit cap) so a query can't flood the bus.
- **Requester (work-node side).** `wm-tether-recall query "<text>" [--kind K]
  [--limit N] [--hybrid]` publishes the request, waits (bounded timeout) for the
  result, and prints ranked hits in a format matching `recall query` so it's a
  drop-in for the work node. Times out cleanly with a non-zero exit if no
  responder answers.
- **Allowlist + safety.** The responder only ever *reads* (query/list); it
  refuses any op that would mutate the store (no `write`/`forget` over the bus in
  v1 — those are the deferred write-back). Subprocess invocation is arg-built
  (no shell), timeout-bounded.
- `wm-tether-recall status` reports responder reachability + last round-trip
  time; SKIPs honestly when no link is configured.

Deps: `clap`, `serde`/`serde_json`, async NATS client (embedded test server for
tests), `sigpipe`. MSRV 1.85, edition 2021. `sigpipe::reset()` first in `main()`.
Honesty discipline: the responder runs against this laptop's real recall in the
world; tests use a fixture recall (a temp store seeded with known rows, or a
stubbed `recall` shim on PATH) so the bridge logic is verified without depending
on the live memory's contents.

## Acceptance criteria

1. Given a fixture recall (seeded temp store or a stub `recall` on PATH returning
   known JSON), a `wm.fleet.recall.query` request for a term present in the
   fixture yields a result with the expected hit(s), ranked, with `id`, `kind`,
   `subject`, `score`, and a bounded `snippet` (embedded NATS).
2. The result payload is size-bounded: snippets are truncated to the configured
   max length and the hit count never exceeds the requested (and cap-clamped)
   `limit`; `truncated:true` is set when clamping occurred.
3. `wm-tether-recall query` (requester) prints ranked hits in a format matching
   `recall query`'s columns for the fixture case (golden-compared).
4. The responder refuses a mutating op: a request carrying a write/forget verb is
   rejected with an error reply and the fixture store is unchanged (read-only
   invariant, grep-assertable that no write path is wired).
5. Requester timeout: with no responder present, `wm-tether-recall query` exits
   non-zero within the bounded timeout and does not hang.
6. (deferred — embedded NATS) End-to-end round-trip selftest: requester →
   `wm.fleet.recall.query` → responder → `wm.fleet.recall.result.<req_id>` →
   requester prints hits, exactly one reply consumed per request (req_id matched).
7. `cargo test` green; `sigpipe::reset()` first in `main()` (grep-asserted);
   subprocess invocation uses no shell metacharacters (grep-asserted: no
   `sh -c`/string-interpolated command line).
