# PRD: mcp-core — the reusable JSON-RPC 2.0 stdio MCP-server core

**Status:** Draft v0.1
**build_target:** rust-lib
**build_into:** (new repo) `~/wintermute/mcp-core`
**Vision:** visions/conduit.md

## TL;DR

`ousia-mcp` (shipped 2026-06-09) hand-rolled an entire JSON-RPC 2.0 stdio
MCP server inline: `src/proto.rs` (Request/Response/RpcError),
`src/dispatch.rs` (initialize / tools/list / tools/call handlers), and a
hardcoded `match name { … }` tool dispatcher in `dispatch_tool`. Every
future conduit server (`recall-mcp`, `docket-mcp`, `muster-mcp`,
`provenance-mcp`) needs exactly the same plumbing and exactly one thing
different: the set of tools. This PRD extracts that plumbing into a
reusable `mcp-core` crate with a `Tool` trait, so ousia-mcp's inline
version is the last one anyone hand-rolls.

## Why this exists

- **The duplication is concrete and imminent.** Read live this pass:
  `ousia-mcp/src/dispatch.rs` exposes `handle_initialize`,
  `handle_tools_list`, `handle_tools_call`, and a `dispatch(req, state)`
  method router that matches `"initialize" | "tools/list" | "tools/call"`.
  `dispatch_tool(name, args, state)` is a hardcoded `match` over the five
  tool names. None of that — except the five arms — is ousia-specific.
  vision-conduit proposes four more servers; without a shared core each
  re-types the protocol and they drift.
- **MCP protocol details should live in one tested place.** ousia-mcp's
  `proto.rs` already pins `jsonrpc: "2.0"`, `protocolVersion:
  "2024-11-05"`, the `-32601` method-not-found code, and the
  `serde(skip_serializing_if)` response shape. A second copy is a second
  place to get the wire format subtly wrong.

## What this builds

A library crate `mcp_core` (no binary) at `~/wintermute/mcp-core`.
Standard scaffold: `Cargo.toml`, `rust-toolchain.toml` (`"1.85.0"`
exact), MIT license (attribution: Joe Yen), README. MSRV 1.85, no
let-chains.

Modules:
- `proto.rs` — `Request { jsonrpc, id, method, params }`,
  `Response { jsonrpc, id, result, error }` with `Response::ok(id, val)` /
  `Response::err(id, RpcError)`, `RpcError { code, message }`, and the
  standard JSON-RPC codes as constants (`METHOD_NOT_FOUND = -32601`,
  `INVALID_PARAMS = -32602`, `INTERNAL_ERROR = -32603`). Ported from
  ousia-mcp's `proto.rs` verbatim where possible.
- `tool.rs` — the abstraction the whole vision turns on:
  ```rust
  pub trait Tool: Send + Sync {
      fn name(&self) -> &str;
      fn description(&self) -> &str;
      fn input_schema(&self) -> serde_json::Value;     // JSON Schema
      fn call(&self, args: &serde_json::Value) -> Result<serde_json::Value, ToolError>;
  }
  pub struct ToolError { pub message: String }          // → RpcError
  ```
- `dispatch.rs` — protocol handlers parameterized over a `&[Box<dyn
  Tool>]`: `handle_initialize(req, server_name, server_version)`,
  `handle_tools_list(req, tools)` (builds the descriptor array from each
  tool's `name`/`description`/`input_schema`), `handle_tools_call(req,
  tools)` (looks up by `params.name`, validates presence, calls, maps
  `ToolError` → `RpcError`), and `dispatch(req, tools, server_name,
  server_version)` routing `initialize | tools/list | tools/call` and
  returning `-32601` for anything else.
- `serve.rs` — `serve_stdio(tools, server_name, server_version) ->
  io::Result<()>`: the newline-delimited read-loop over stdin, dispatch,
  write response to stdout, flush. Notifications (no `id`) get no
  response. EOF on stdin ends the loop cleanly.
- `lib.rs` — re-export `Tool`, `ToolError`, `Request`, `Response`,
  `RpcError`, `serve_stdio`, `dispatch`.

Deps: `serde`, `serde_json`. No transport crate (stdio is std). No
network.

## Acceptance criteria

1. `cargo build` and `cargo test` are green on toolchain 1.85; clippy adds
   no new warnings; MSRV 1.85, no let-chains.
2. A test registers two trivial `Tool` impls (e.g. `echo`, `add`) and
   drives `dispatch` directly: an `initialize` request returns
   `protocolVersion`, `capabilities.tools`, and `serverInfo
   {name,version}`; a `tools/list` returns both tools with their
   `inputSchema`; a `tools/call` of `echo` returns the echoed value.
3. `dispatch` returns a JSON-RPC error with `code == -32601` for an
   unknown method, and `-32602`-class error for `tools/call` naming a tool
   that isn't registered (asserted on the `error.code`/`message`).
4. A `Tool::call` returning `Err(ToolError)` is mapped to a JSON-RPC
   `error` response (not a panic, not a dropped connection); the loop
   continues to the next request.
5. `serve_stdio` round-trips: a test feeds two newline-delimited requests
   through an in-memory reader/writer (or a `serve` helper generic over
   `BufRead`/`Write`) and asserts two newline-delimited responses in
   order; a notification (no `id`) produces no response line.
6. The wire constants match ousia-mcp's shipped values exactly:
   `jsonrpc == "2.0"`, `protocolVersion == "2024-11-05"`, method-not-found
   `== -32601` (a test asserts these literals so the two servers stay
   interoperable with the same clients).
7. README documents the `Tool` trait and shows a ~20-line "minimal MCP
   server" example using `serve_stdio`.
