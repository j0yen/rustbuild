# PRD: ousia-mqo-mcp — expose ousia-mqo tools in the mqo-mcp-server subprocess protocol

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/ousia-mqo
Vision: visions/ousia-mqo.md
Depends-on: ousia-mqo-ground, ousia-mqo-diff, ousia-mqo-bind

## TL;DR

`ousia-mqo ground`, `diff`, and `bind` are standalone CLIs. To make them
available to Claude and other MCP agents through `mqo-mcp-server`, they need
to speak the server's **subprocess tool protocol**: accept a JSON payload on
stdin, emit a JSON response on stdout, exit 0/non-zero. This PRD ships
`ousia-mqo serve` — a single dispatching binary that wraps all three CLIs in
that protocol — plus the registration snippet for `mqo-mcp-server`'s `ToolPaths`
config so agents can call `semantic_ground_model`, `semantic_diff_clusters`, and
`semantic_bind_phrase` as named MCP tools.

## Why this exists

Verified 2026-06-15 by reading `joeyen-atscale/mqo-mcp` ARCHITECTURE.md (Section
"Subprocess tools"): subprocess tools in `mqo-mcp-server` are binaries called via
`ToolPaths`, communicate via stdin/stdout JSON, and return `{ok: bool, data: …}`
or `{ok: false, error: "…"}`. The pipeline tools (`mqo-catalog-binder`,
`mqo-dax-compiler`, etc.) follow this exact shape. Adding `ousia-mqo serve` as
another `ToolPaths` entry requires no changes to the server source — only a config
addition and the binary on `$PATH`.

The three tools this wraps:
- `ground_model(model_json: Value) -> GroundedOverlay` — ground a `describe_model`
  response inline (common LLM call: "ground this model").
- `semantic_diff_clusters(model_a: Value, model_b: Value) -> DiffResult` — formal
  cross-cluster semantic diff (replaces/augments `mcp-cross-cluster-diff`).
- `semantic_bind_phrase(phrase: String, model_json: Value, top: usize) ->
  Vec<BindCandidate>` — grounded NL → candidate measures.

## What this builds

Extend `~/wintermute/ousia-mqo`:

- **`src/serve.rs` — subprocess dispatcher**:
  - `ousia-mqo serve` reads one line of JSON from stdin:
    `{"tool": "<name>", "args": {…}}` and emits `{"ok": true, "data": {…}}` or
    `{"ok": false, "error": "…"}` to stdout.
  - Dispatches on `tool`:
    - `"ground_model"` → args: `{model: Value}` → `Grounder::ground_value`
    - `"semantic_diff_clusters"` → args: `{model_a: Value, model_b: Value}` →
      `SemanticDiff::compare`
    - `"semantic_bind_phrase"` → args: `{phrase: String, model: Value, top?: usize}`
      → `NlBinder::resolve_json`
  - Unknown tool → `{"ok": false, "error": "unknown tool: <name>"}`, non-zero exit.
  - All in-process (no subprocess call to `ousia-mqo ground`/`diff`/`bind` —
    those binaries exist for standalone use; `serve` links the libs directly).
- **`ousia-mqo tool-list`** — emits the tool schema for `mqo-mcp-server`'s tool
  discovery (JSON array of `{name, description, input_schema}`), so the server can
  auto-register without a static config snippet.
- **Registration doc** at `docs/mqo-mcp-registration.md`:
  - The exact `ToolPaths` config stanza to add to `mqo-mcp-server`'s config.
  - Tool request/response samples for each of the three tools.
  - One-line install note: `cargo install --path . --features serve`.

MSRV 1.85. No new deps beyond the prior ousia-mqo PRDs.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `echo '{"tool":"ground_model","args":{"model": <sales_fixture>}}' | ousia-mqo serve`
   emits `{"ok":true,"data":{…}}` with the grounded overlay for the sales fixture
   — a test asserts `data.annotations.revenue.philosophical_grounding.iri ==
   "http://purl.obolibrary.org/obo/BFO_0000033"`.
3. `semantic_diff_clusters` with identical model A and B emits `{"ok":true,"data":
   {"diverge_count":0,"verdicts":[…]}}` — a test asserts `diverge_count == 0`.
4. `semantic_bind_phrase` with `{"phrase":"revenue","model":<fixture>}` emits
   `{"ok":true,"data":[…]}` with ≥1 candidate — a test asserts the candidate list
   is non-empty.
5. An unknown tool name emits `{"ok":false,"error":"unknown tool: …"}` and exits
   non-zero — a test asserts both.
6. `ousia-mqo tool-list` emits a valid JSON array with 3 entries each having
   `name`, `description`, `input_schema` — a test asserts the count and the three
   expected names.
7. `docs/mqo-mcp-registration.md` exists and contains the exact `ToolPaths` config
   stanza and one request/response sample per tool.
8. `echo '{"tool":"ground_model","args":{"model":{}}}' | ousia-mqo serve` (empty
   model) emits `{"ok":false,"error":"…"}` and exits non-zero — a test asserts
   graceful handling of a malformed/empty model.
