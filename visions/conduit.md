# Vision: conduit — wintermute's tools become callable capabilities over MCP

## TL;DR

This laptop is a consumer of MCP servers (settings.json wires
`awslabs-aws-api`, `awslabs-aws-docs`; live sessions attach claude.ai
Gmail/Calendar/Drive/AtScale) but, until this week, a *provider* of none.
`ousia-mcp` (shipped 2026-06-09) changed that — it exposes the ousia
ontology over a stdio MCP server — but it hand-rolled the entire JSON-RPC
2.0 plumbing inline (`src/proto.rs`, `src/dispatch.rs`, `src/server.rs`),
and it is a one-off: nothing else on this box is reachable by an external
agent. `conduit` is the connective tissue that carries wintermute's most
valuable *read* surfaces — its memory (recall), its chronic-finding ledger
(docket), its live-session census (muster), and its kernel-stamped file
provenance (provfs/memlog) — out to any MCP client, on top of one shared,
tested core extracted from ousia-mcp's proof.

## End-state

When this is done:

- A single reusable `mcp-core` crate owns the JSON-RPC 2.0 stdio loop, the
  `initialize` handshake, `tools/list`/`tools/call` dispatch, and a `Tool`
  trait. No future MCP server re-hand-rolls the protocol — ousia-mcp's
  inline plumbing becomes the last hand-rolled one.
- The laptop's four highest-value read surfaces are each a registered MCP
  server: `recall-mcp`, `docket-mcp`, `muster-mcp`, `provenance-mcp`.
- Every conduit server is **read-only by construction** — it can only
  invoke an allowlisted subset of its backing tool's subcommands; the
  mutating verbs (recall write/delete/reindex, docket resolve/ack/sweep,
  muster reap) are structurally unreachable.
- `mcp-register` closes the loop ousia-mcp left open ("no settings.json
  edits performed by the PRD itself"): it wires the conduit servers into a
  client's MCP config idempotently, proposal-by-default, atomic-backup on
  `--write`.
- Any agent — this laptop's Claude today, a customer's agent tomorrow —
  can semantic-search wintermute's memory, read its open findings, see its
  session population, and ask "who wrote this file" without linking Rust or
  shelling out.

## Why now (evidence)

- **ousia-mcp proved the pattern this tick** (2026-06-09): a working stdio
  MCP server, 15 tests green, `initialize` + `tools/list` + five tools.
  Its `src/` is six small modules; `proto.rs`/`dispatch.rs`/`server.rs` are
  pure protocol with no ousia in them — i.e. already an extractable core.
- **The seam was flagged three ticks running.** ousia-mcp's own "Why this
  exists" cites the gossip log: *"the gossip log has flagged
  'MCP-connector-aware layer' as the one un-covered outward seam three
  ticks running."*
- **The backing tools already have read-capable CLIs.** Verified live:
  `recall query|list|show|similar|lineage|sessions`, `docket
  list|show|digest`, `muster census|verdict` — each a stable subcommand
  surface with a clean mutating/non-mutating split.
- **The kernel surface for provenance is already shipped.** provfs stamps
  `user.prov.session`/`user.prov.ts` on closed-after-write files; memlog is
  a per-uid circular log of pre-compaction snapshots (`cli/memlog show
  --since 1h --format json`). A provenance MCP tool consumes these — it
  doesn't invent plumbing.

## Components (one bullet per future PRD)

1. **mcp-core** (`PRD-mcp-core`, rust-lib, new `~/wintermute/mcp-core`) —
   extract ousia-mcp's JSON-RPC 2.0 stdio server into a reusable crate:
   `Request`/`Response`/`RpcError`, `initialize` handshake, `tools/list` +
   `tools/call` dispatch, a `Tool` trait (`name`/`description`/
   `input_schema`/`call`), and a `serve_stdio(Vec<Box<dyn Tool>>)` driver.
   Keystone — every server below depends on it.

2. **recall-mcp** (`PRD-recall-mcp`, rust-cli, new `~/wintermute/recall-mcp`)
   — MCP server over recall's read surface. Tools: `memory_query`,
   `memory_list`, `memory_show`, `memory_similar`, `memory_lineage`,
   `memory_sessions`. Shells the installed `recall` binary; allowlists only
   read subcommands. Highest leverage (recall is the most-queried tool).

3. **docket-mcp** (`PRD-docket-mcp`, rust-cli, new `~/wintermute/docket-mcp`)
   — MCP server over docket. Tools: `findings_list`, `finding_show`,
   `findings_digest`. Read-only; resolve/ack/sweep unreachable.

4. **muster-mcp** (`PRD-muster-mcp`, rust-cli, new `~/wintermute/muster-mcp`)
   — MCP server over muster. Tools: `sessions_census`, `sessions_verdict`.
   Read-only; reap unreachable.

5. **provenance-mcp** (`PRD-provenance-mcp`, rust-cli, new
   `~/wintermute/provenance-mcp`) — MCP server over kernel provenance.
   Tools: `file_provenance` (getfattr `user.prov.*`), `memlog_recent`
   (memlog snapshots). Leans on shipped kernel primitives.

6. **mcp-register** (`PRD-mcp-register`, rust-cli, new
   `~/wintermute/mcp-register`) — wire conduit servers into a client's MCP
   config (`~/.claude.json` / project `.mcp.json`) idempotently.
   Proposal-by-default; `--write` applies with timestamped atomic backup.
   Consumes the server names; closes ousia-mcp's open "who registers it"
   question.

## Order

```
mcp-core ──┬──► recall-mcp ──┐
           ├──► docket-mcp ──┤
           ├──► muster-mcp ──┼──► mcp-register
           └──► provenance-mcp ┘
```

mcp-core is the keystone (everything depends on it). The four servers are
mutually independent and ship in parallel. mcp-register depends on the
servers existing (it registers them by name) but not on their internals.

## Open questions

- **Link vs shell.** ousia-mcp links the ousia libs as path deps. The
  conduit servers shell out to the *installed binaries* instead (recall,
  docket, muster are stable CLIs already on PATH) — more decoupled, no
  version lockstep, and the read-only allowlist is enforced at the
  subcommand boundary. Provenance-mcp shells `getfattr`/`memlog`. Confirm
  this is the right trade vs linking `docket-core`/`recall-io` libs.
- **Transport.** stdio only for v1 (matches ousia-mcp). An HTTP/SSE
  transport for remote agents is a later PRD if a customer-facing use
  appears.
- **Auth.** All conduit servers are read-only and local; no auth in v1.
  If provenance-mcp ever exposes another user's files, revisit.
- Should mcp-core also absorb a back-port of ousia-mcp onto itself (a
  follow-on `ousia-mcp` v0.2 that deletes its inline plumbing)? Left as a
  vision note, not drafted — ousia-mcp works; the refactor is optional.
