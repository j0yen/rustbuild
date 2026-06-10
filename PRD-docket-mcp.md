# PRD: docket-mcp — the chronic-finding ledger as a read-only MCP server

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/docket-mcp`
**Vision:** visions/conduit.md

## TL;DR

`docket` tracks chronic findings and how many runs each has survived — it
is what self-review reads each morning to decide what's still open. But its
state is reachable only by shelling `docket list`/`docket show`. This PRD
exposes docket's **read** surface over MCP so an agent can ask "what's been
open longest, what's escalated, what's the digest" through a connector,
with `resolve`/`ack`/`sweep` structurally unreachable.

## Why this exists

- **docket already drives a daily human read; an agent should share it.**
  Verified live: `docket` offers `list`, `show`, `digest` (compact
  open/escalated rollup for banners) on the read side, and
  `report`/`resolve`/`ack`/`unack`/`sweep` on the write side. The
  2026-06-09 self-review cites docket findings by name (`ctrace-sessionend-
  flake runs_seen:6`, `agentns-session-zeros runs_seen:9`) — exactly the
  records an agent would want to query.
- **It rides the conduit core.** Like recall-mcp, this is a thin `Tool`
  layer over `mcp-core` shelling the installed `docket` binary; no new
  protocol code.
- **The read/write split is clean.** An allowlist of `{list,show,digest}`
  makes the mutating verbs unreachable — an external agent can observe the
  finding ledger but never resolve or acknowledge a finding (those stay
  human/self-review actions).

## What this builds

A binary crate `docket-mcp` (binary `docket-mcp`) at
`~/wintermute/docket-mcp`. `sigpipe::reset()` first line of `main()`.
Depends on `mcp-core` (path); shells the installed `docket` binary.

- `backend.rs` — `DocketCli { bin: PathBuf }` (default `docket` on PATH;
  `--docket-bin`/`DOCKET_BIN` override). Methods `list(escalated?,
  format=json)`, `show(key)`, `digest()` — each a fixed allowlisted verb
  literal + validated flags via `std::process::Command`, returning parsed
  JSON.
- `tools.rs` — three `Tool` impls: `findings_list {escalated?: bool}`,
  `finding_show {key: string}`, `findings_digest {}`. JSON-Schema
  `input_schema` each.
- `main.rs` — clap: `docket-mcp serve [--docket-bin <path>]` registers the
  three tools and calls `mcp_core::serve_stdio`.

Deps: `mcp-core` (path), `serde_json`, `clap`, `sigpipe`, `anyhow`.

## Acceptance criteria

1. `cargo build --release` produces `docket-mcp`; `cargo test` green; MSRV
   1.85, no let-chains; `sigpipe::reset()` first in `main()`.
2. `docket-mcp serve` completes `initialize` and `tools/list` returns
   exactly the three read tools with valid JSON-Schema `inputSchema`.
3. **Read-only by construction**: a test asserts the backend's verb set ⊆
   `{list,show,digest}` and a guard rejects
   `report|resolve|ack|unack|sweep` appearing as a subcommand literal.
4. `findings_list {escalated:true}` invokes `docket list --escalated
   --format json` and returns the parsed array (verified against a stubbed
   `docket` binary on PATH; offline).
5. `finding_show` with no `key` → JSON-RPC error, no panic, no shell call.
6. `--docket-bin`/`DOCKET_BIN` redirects to a fixture script; absent a
   usable binary, `serve` starts and tool calls return a clean `ToolError`.
7. README documents the three tools, the read-only guarantee, and an MCP
   client config snippet; no settings.json edits performed by this PRD.
