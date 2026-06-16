# PRD: ousia-atscale-mcp — expose BFO grounding to a live AI agent over MCP

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/ousia-atscale
Vision: visions/ousia-atscale.md
Depends-on: ousia-atscale-diff (so the server can also expose diff)

## TL;DR

The MARKET.md pitch's third claim is "AI-ready: a semantic layer whose measures
trace to BFO categories can be traversed deductively by an LLM or agent — the agent
*knows* that revenue is an information GDC about a sales process." But an AI agent
has no way to *ask* ousia-atscale anything — the tool is CLI-only. This PRD adds a
`serve` subcommand exposing ground / report / diff over MCP (stdin/stdout JSON-RPC),
reusing the `ousia-mcp` pattern, so a Claude session analysing an AtScale model can
query its formal grounding mid-conversation.

## Why this exists

Verified 2026-06-16: `ousia-mcp` exists at `~/.local/bin/ousia-mcp` (the World
Ontology already has an MCP server); `lattice-context serve` also runs as an MCP
server on stdin/stdout — so the box has an established MCP-server pattern to copy.
The MARKET.md "Next Steps" explicitly lists "Expose ousia-atscale as an MCP tool so
AI agents can query grounding directly." The AtScale Non-prod/Prod MCP connectors
that feed `describe_model` are already attached in interactive sessions — so an agent
can fetch a model *and* ground it without leaving the conversation.

## What this builds

Extend `~/wintermute/ousia-atscale`:

- **New subcommand `serve`** in `src/main.rs`: run an MCP server on stdin/stdout.
- **New module `src/mcp.rs`** exposing MCP tools:
  - `ground_model(model_json)` → the per-element BFO grounding (JSON)
  - `coverage_report(model_json)` → the coverage stats (JSON)
  - `diff_models(model_a_json, model_b_json)` → the diff result (JSON, from -diff)
  - each tool takes a model JSON string (the shape the AtScale MCP `describe_model`
    returns), so an agent can pipe one MCP tool's output into this one.
- Follow the existing `ousia-mcp` wire format exactly (same MCP library/crate the
  ousia workspace uses — check `ousia-mcp`'s Cargo.toml and reuse it; do NOT invent
  a new MCP framing). If `ousia-mcp` uses a shared `mcp-core` crate (present at
  `~/wintermute/mcp-core`), depend on that.
- The server is **read-only / pure**: it computes grounding from the supplied model
  JSON and returns it. No file writes, no network. This keeps it safe to expose.
- Document the server in README under a new "MCP server" section with a registration
  snippet for `~/.claude/settings.json` (or `claude mcp add`).

MSRV: match `ousia-mcp` / `mcp-core`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `ousia-atscale serve` starts an MCP server that responds to an MCP `initialize`
   handshake on stdin/stdout (tested by feeding a recorded initialize request and
   asserting a well-formed response).
3. The server advertises at least three tools: `ground_model`, `coverage_report`,
   `diff_models` (assert via the MCP `tools/list` response).
4. Calling `ground_model` with the sales fixture JSON returns a grounding with 13
   elements (a test drives one tool call end-to-end over the MCP transport).
5. Calling `diff_models` with two distinct fixtures returns a diff result matching
   what the `diff` CLI produces for the same inputs (consistency between surfaces).
6. The server performs no file or network I/O during a tool call (verified by a test
   that runs a call and asserts no files were created in a temp cwd).
7. README has an "MCP server" section with a working registration snippet.
