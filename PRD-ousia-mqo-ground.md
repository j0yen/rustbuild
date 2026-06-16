# PRD: ousia-mqo-ground — BFO-grounded AtScale model overlays as a reusable lib

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/ousia-mqo
Vision: visions/ousia-mqo.md

## TL;DR

`ousia-atscale annotate` already grounds AtScale model elements to BFO 2020
categories and writes a JSON overlay to disk. But it's a standalone CLI with no
library API — nothing can link against it and call it programmatically. This PRD
ships `ousia-mqo` (new repo `joeyen-atscale/ousia-mqo`) with a `Grounder` lib
and a `ousia-mqo ground` CLI that wraps `ousia-atscale annotate` and exposes the
overlay as a typed Rust structure other crates in this repo can consume. It is
the foundation the diff, bind, and MCP tools all build on.

## Why this exists

Verified 2026-06-15 by running `ousia-atscale annotate --model
fixtures/sales_model.json --out /tmp/grounded.json`:

The overlay JSON shape is:
```json
{
  "model_catalog": "acme",
  "model_schema": "sales",
  "model_table": "fact_orders",
  "overlay_version": "ousia-atscale/0.1.0",
  "annotations": {
    "<element_name>": {
      "philosophicalGrounding": {
        "iri": "http://purl.obolibrary.org/obo/BFO_0000033",
        "label": "information generically dependent continuant",
        "rationale": "..."
      },
      "domainModule": "FinancialProcess",
      "aristotelianDefinition": {
        "genus": "information generically dependent continuant",
        "differentia": "..."
      }
    }
  }
}
```

This is the key atom. `mqo-mcp`'s `mcp-cross-cluster-diff` compares catalogs by
column name and type (verified 2026-06-15 via `gh api` on ARCHITECTURE.md) — it
has no concept of BFO IRI. Linking this shape into a typed Rust lib gives all
`ousia-mqo` tools a shared model.

`ousia-atscale annotate` is deterministic (same model → same JSON), so the
grounder can cache by model content hash under `~/.cache/ousia-mqo/<hash>.json`
and skip re-running for an unchanged model — each annotation shell-out takes
~100ms and is avoidable for repeated calls in one server session.

## What this builds

New repo `joeyen-atscale/ousia-mqo`:

- **`src/lib.rs` — `Grounder` lib**:
  - `GroundedOverlay` struct (deserialize from the `ousia-atscale annotate` JSON
    above): `model_catalog`, `model_schema`, `model_table`, `overlay_version`,
    `annotations: HashMap<String, Annotation>`.
  - `Annotation` struct: `philosophical_grounding: Grounding`, `domain_module:
    String`, `aristotelian_definition: AristotelianDef`.
  - `Grounding` struct: `iri: String`, `label: String`, `rationale: String`.
  - `AristotelianDef` struct: `genus: String`, `differentia: String`.
  - `Grounder::new(ousia_atscale_bin: PathBuf, cache_dir: PathBuf)` — wraps the
    CLI; resolves `ousia-atscale` from `$PATH` if no explicit bin given.
  - `Grounder::ground(model_json: &Path) -> Result<GroundedOverlay>` — hash the
    model JSON content, check cache, else shell out to `ousia-atscale annotate
    --model <path> --out <cache/<hash>.json>`, deserialize, return. Cache miss is
    the only path that does I/O beyond the hash check.
  - `Grounder::ground_value(model: &serde_json::Value) -> Result<GroundedOverlay>`
    — same but accepts an in-memory model (write to a temp file, ground, return).
- **`src/main.rs` — `ousia-mqo ground` CLI**:
  - `ousia-mqo ground --model <path> [--out <path>] [--no-cache]` — shell for the
    lib; `--out` defaults to stdout (JSON). `--no-cache` forces re-annotation.
  - Mirrors `ousia-atscale annotate`'s flags so it can be a drop-in for callers
    that want the typed cache layer.
- **`ousia-mqo`** binary: the top-level subcommand dispatcher (`ground`, `diff`,
  `bind`, `serve` — later PRDs add the other subcommands).
- Tool resolution: resolve `ousia-atscale` from `$PATH`; error message names it
  explicitly if absent. No bundling of `ousia-atscale` — it is an external dep.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json, sha2 (cache hash). Published to
`joeyen-atscale/ousia-mqo` (private or public per user; default public to match
the mqo-mcp ecosystem).

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `ousia-mqo ground --model <fixture>` with `ousia-atscale` on `$PATH` emits a
   JSON `GroundedOverlay` to stdout with the documented shape — a test round-trips
   the sales fixture and asserts `annotations["revenue"].philosophical_grounding
   .iri == "http://purl.obolibrary.org/obo/BFO_0000033"`.
3. A second call with the same model is served from cache (no second invocation of
   `ousia-atscale`) — a test asserts the cache file exists after the first call and
   the binary is not re-invoked on the second (stub `ousia-atscale` that records
   invocations).
4. `--no-cache` bypasses the cache — a test asserts the stub is re-invoked.
5. `Grounder::ground_value` round-trips an in-memory model JSON (a test loads the
   sales fixture with `serde_json::from_str` and asserts annotation count ≥ 1).
6. A missing `ousia-atscale` binary makes `ground` exit non-zero with a message
   naming the missing tool.
7. `ousia-mqo --help` lists `ground`, and `ousia-mqo ground --help` documents
   `--model`, `--out`, `--no-cache`.
