# PRD: muster-mcp — the live-session census as a read-only MCP server

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/muster-mcp`
**Vision:** visions/conduit.md

## TL;DR

`muster` answers the question self-review keeps guessing at — which Claude
sessions are live, which are duplicates, orphans, or stale — with origin
attribution and bus reconciliation. This PRD exposes muster's **read**
surface (`census`, `verdict`) over MCP so an agent can ask "what's the
current session population" through a connector, with `reap` structurally
unreachable.

## Why this exists

- **The census is exactly what an agent needs and can't currently get.**
  Verified live: `muster` offers `census` (enumerate live sessions with
  origin attribution + bus reconciliation) and `verdict` (annotate with
  live/duplicate/orphan/stale + evidence) on the read side, and `reap`
  (propose/execute cleanup) on the write side. This very tick shipped
  `muster-selfreview-bridge` so self-review reads `muster verdict --format
  selfreview` instead of `pgrep`; an MCP surface generalizes that to any
  agent.
- **It rides the conduit core.** Thin `Tool` layer over `mcp-core`
  shelling the installed `muster` binary; no new protocol code.
- **Reap must stay human-gated.** muster's reap is already `--confirm`-
  gated and proposal-by-default; exposing only `{census,verdict}` over MCP
  keeps the kill path entirely off the connector — an agent can see the
  roster but never reap from it.

## What this builds

A binary crate `muster-mcp` (binary `muster-mcp`) at
`~/wintermute/muster-mcp`. `sigpipe::reset()` first line of `main()`.
Depends on `mcp-core` (path); shells the installed `muster` binary.

- `backend.rs` — `MusterCli { bin: PathBuf }` (default `muster` on PATH;
  `--muster-bin`/`MUSTER_BIN` override). Methods `census(format=json)`,
  `verdict(format=json)` — fixed allowlisted verb literals + `--format
  json` via `std::process::Command`, returning parsed JSON.
- `tools.rs` — two `Tool` impls: `sessions_census {}`, `sessions_verdict
  {}`. JSON-Schema `input_schema` each (both no-arg; schema is an empty
  object).
- `main.rs` — clap: `muster-mcp serve [--muster-bin <path>]` registers the
  two tools and calls `mcp_core::serve_stdio`.

Deps: `mcp-core` (path), `serde_json`, `clap`, `sigpipe`, `anyhow`.

## Acceptance criteria

1. `cargo build --release` produces `muster-mcp`; `cargo test` green; MSRV
   1.85, no let-chains; `sigpipe::reset()` first in `main()`.
2. `muster-mcp serve` completes `initialize` and `tools/list` returns
   exactly the two read tools with valid JSON-Schema `inputSchema`.
3. **Read-only by construction**: a test asserts the backend's verb set ⊆
   `{census,verdict}` and a guard rejects `reap` appearing as a subcommand
   literal anywhere in the binary's command construction.
4. `sessions_verdict` invokes `muster verdict --format json` and returns
   the parsed JSON (verified against a stubbed `muster` binary on PATH;
   offline — no real session enumeration needed).
5. A `tools/call` naming an unregistered tool (e.g. `sessions_reap`) yields
   a JSON-RPC error, not a shell call — proving reap is unreachable.
6. `--muster-bin`/`MUSTER_BIN` redirects to a fixture script; absent a
   usable binary, `serve` starts and tool calls return a clean `ToolError`.
7. README documents the two tools, that reap is intentionally excluded, and
   an MCP client config snippet; no settings.json edits performed by this
   PRD.
