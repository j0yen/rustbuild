# PRD: recall-mcp — wintermute's memory store as a read-only MCP server

**Status:** Draft v0.1
**build_target:** rust-cli
**build_into:** (new repo) `~/wintermute/recall-mcp`
**Vision:** visions/conduit.md

## TL;DR

`recall` is the most-queried tool on this laptop — every /dream pass opens
with `recall query`/`recall list`, and self-review leans on it — but it is
reachable only by shelling out from a Rust/Bash context. This PRD exposes
recall's **read** surface over MCP so any agent (this box's Claude today, a
customer's agent tomorrow) can semantic-search wintermute's memory through
a standard connector, with the mutating verbs structurally unreachable.

## Why this exists

- **recall is the highest-leverage surface to expose.** Verified live this
  pass: `recall` offers `query` (keyword + optional vector, ranked),
  `list` (newest-first, `--subject` prefix), `show <id>`, `similar <id>`,
  `lineage <id>`, and `sessions` — a clean read cluster. It also offers
  `write`/`update`/`delete`/`reindex`/`touch` — the mutating cluster an
  external agent must never reach.
- **The pattern is proven and about to be shared.** ousia-mcp shipped a
  working stdio MCP server 2026-06-09; `PRD-mcp-core` extracts its plumbing
  into a `Tool` trait. recall-mcp is the first consumer of that core and
  the one with the most immediate payoff.
- **Read-only-by-construction is enforceable here.** Because recall's
  read/write split is at the subcommand boundary, an allowlist of
  `{query,list,show,similar,lineage,sessions}` makes write paths
  unreachable — there is no code path in recall-mcp that can emit a
  mutating subcommand.

## What this builds

A binary crate `recall-mcp` (binary `recall-mcp`) at
`~/wintermute/recall-mcp`. `sigpipe::reset()` first line of `main()`.
Depends on `mcp-core` (path) for the protocol; shells the installed
`recall` binary for data.

- `backend.rs` — `RecallCli { bin: PathBuf }` (default resolves `recall`
  on PATH; override via `--recall-bin` / `RECALL_BIN`). One method per
  read subcommand, each building an arg vector from a **fixed allowlisted
  verb** + validated flags, running via `std::process::Command`, and
  returning parsed JSON (recall already supports JSON output for query;
  for list/show, parse or wrap stdout). A hard guard: the verb is a
  compile-time `&str` literal per method — never interpolated from tool
  args — so no caller can inject `write`/`delete`.
- `tools.rs` — six `Tool` impls over `backend`:
  `memory_query {query, limit?, hybrid?, kind?}`,
  `memory_list {kind?, subject?, limit?}`, `memory_show {id}`,
  `memory_similar {id, limit?}`, `memory_lineage {id}`,
  `memory_sessions {}`. Each with a JSON-Schema `input_schema`.
- `main.rs` — clap: `recall-mcp serve [--recall-bin <path>]` registers the
  six tools and calls `mcp_core::serve_stdio`.

Deps: `mcp-core` (path), `serde_json`, `clap`, `sigpipe`, `anyhow`.

## Acceptance criteria

1. `cargo build --release` produces `recall-mcp`; `cargo test` green; MSRV
   1.85, no let-chains; `sigpipe::reset()` is `main()`'s first statement.
2. `recall-mcp serve` completes the MCP `initialize` handshake and
   `tools/list` returns exactly the six read tools, each with a valid
   JSON-Schema `inputSchema` (asserted against a fixture client driver).
3. **Read-only by construction**: a test enumerates every verb the backend
   can emit and asserts it is a subset of
   `{query,list,show,similar,lineage,sessions}`; a grep-style guard in the
   test rejects `write|update|delete|reindex|touch|init` appearing as a
   subcommand literal.
4. A `memory_query` tool call with `{query:"…", limit:5, hybrid:true}`
   invokes `recall query "…" --limit 5 --hybrid` and returns its ranked
   results as JSON (verified against a stubbed `recall` binary on PATH in
   the test, so no real store is needed and the test runs offline).
5. Bad/missing required args (`memory_show` with no `id`) yield a JSON-RPC
   error response, not a panic and not a shell invocation.
6. The backing binary is configurable: `--recall-bin`/`RECALL_BIN` points
   at a fixture script in tests; absent a usable binary, `serve` still
   starts and tool calls return a clean `ToolError` (no crash).
7. README documents the six tools, the read-only guarantee, and an MCP
   client config snippet registering `recall-mcp serve`; no settings.json
   edits are performed by this PRD (registration is `mcp-register`'s job).
