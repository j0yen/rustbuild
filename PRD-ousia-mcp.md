# PRD: ousia-mcp — expose the ontology to any agent over MCP

Status: Draft v0.1
build_priority: high
build_target: rust-cli
Vision: visions/ousia.md

## TL;DR

`ousia-mcp` is an MCP server that exposes `ousia-reason`, `ousia-sparql`, and
`ousia-guard` as MCP tools, so any agent — this laptop's Claude sessions today,
a customer's agent tomorrow — can classify entities, query the ethical
structure, and gate actions without linking Rust. MCP is the distribution
channel that turns the ontology from a local CLI into a callable capability.

## Why this exists

- **MCP is the proven connector surface on this box.** This very /dream session
  has live MCP servers attached (claude.ai Gmail / Calendar / Drive / AtScale —
  gossip 2026-06-08). The gossip log has flagged "MCP-connector-aware layer" as
  the one un-covered outward seam three ticks running. ousia-mcp is a concrete,
  motivated use of that surface.
- **Reuse beats re-implementation.** reason/sparql/guard already expose lib APIs
  (their PRDs mandate it). ousia-mcp is a thin protocol shim over them, not new
  reasoning — small scope, high leverage.
- **It is how "ethical AI" reaches other agents.** The paper's §9.1 vision is an
  AI that *consumes* the ontology. MCP is the standard way an LLM agent consumes
  an external capability today; a Rust MCP server makes the World Ontology a
  drop-in tool.

## What this builds

A `cargo` Rust CLI `ousia-mcp` at `~/wintermute/ousia-mcp/` — an MCP server
(stdio transport at minimum; HTTP optional) over the ousia libs.

**Deps (pinned at build time):**
- An MCP server crate for Rust (e.g. `rmcp` / official Rust MCP SDK — exact
  crate + version resolved at build time), or a minimal hand-rolled JSON-RPC
  stdio loop if no suitable crate resolves.
- `ousia-reason`, `ousia-sparql`, `ousia-guard` (libs).
- `serde_json`, `clap`.

**Tools exposed:**
- `classify(abox)` → materialized inferences for the supplied facts.
- `query(sparql)` → SPARQL results JSON (read-only).
- `ask_canned(name)` → run a named query from sparql's pack.
- `guard_check(action)` → `{verdict, rules_fired, justifications}`.
- `explain(entity)` → axiom chain for an inferred fact.

**Config.** Server points at a forged `world-ontology.owl` (path via flag/env);
loads it once at startup, materializes, holds the reasoned store in memory.

**UX.**
```
ousia-mcp serve --owl world-ontology.owl            # stdio MCP server
# register in a client's mcp config as command: ousia-mcp serve --owl <path>
```

## Acceptance criteria

1. `ousia-mcp serve` starts an MCP server over stdio that completes the MCP
   initialize handshake and lists the five tools with valid JSON schemas.
2. A `guard_check` tool call with a dignity-violating action returns the same
   `deny` verdict + justification that the `ousia-guard` CLI returns for the
   same input (parity test).
3. A `query` tool call with a SPARQL SELECT returns SPARQL results JSON
   identical to `ousia-sparql query` for the same query/store.
4. `classify` and `explain` tool calls return the same materializations /
   axiom chains as the corresponding `ousia-reason` CLI invocations.
5. The server is read-only: no tool mutates the ontology or store; `query`
   rejects SPARQL UPDATE.
6. The server loads the ontology once and answers subsequent calls from the
   in-memory reasoned store (no re-materialization per call) — verifiable by a
   timing assertion or a load-count instrument in tests.
7. Ships a documented example MCP client config snippet (`examples/`) and a
   README section for registering the server with Claude Code, with no
   settings.json edits performed by the PRD itself.
8. `cargo test` covers the initialize handshake, tool-list schema validity, and
   CLI/MCP parity for `guard_check` and `query` on fixtures.
