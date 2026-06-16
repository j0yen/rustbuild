# PRD: mqo-catalog-embed — semantic retrieval over the model catalog, beyond keyword search

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-catalog-embed (NOT AtScaleInc; consumes public mqo-spec shapes only)

## TL;DR

The single biggest lever on NL→metric binding accuracy is finding the *right*
measure among hundreds — and the live MCP's `search_columns` is documented as
**keyword search**. An agent asked for "gross margin" never finds `net_margin_pct`
or `gross_profit_rate` if the lexical tokens don't overlap. `mqo-catalog-embed`
adds semantic (embedding) retrieval over measure/dimension captions and
descriptions, so meaning matches even when words don't.

## Why this exists (Phase-1 evidence)

- The AtScale MCP server instructions state `search_columns` is "(keyword
  search)" — verified in this session's MCP server description. Purely lexical
  retrieval misses synonyms and paraphrases.
- The live models carry rich, retrievable text: model `description`s
  (AdventureWorks/TPC-DS prose), captions, and dimension/measure names that are
  business-meaningful (`AVG_ORDER_VALUE`, `IS_HEALTHY_FLAG`).
- `ousia-mqo-bind` resolves NL→BFO→measure *formally* (great when grounding
  exists); `mqo-catalog-embed` is the *lexical-free statistical* complement that
  works on any model whether or not it's BFO-grounded — and feeds better
  candidates into both the binder and `mqo-bench`.
- `recall` already ships a local BGE embedder on this box (offline, no API),
  so the embedding dependency is already solved and proven.

## What this builds

A standalone Rust CLI `mqo-catalog-embed`:

- `mqo-catalog-embed index --model <describe_model.json> [--out <index.bin>]` →
  embed each column/measure's `{name, caption, description, folder}` into a
  compact local vector index.
- `mqo-catalog-embed search --index <index.bin> --query "<nl phrase>" [--top-k 8]`
  → top-k columns by cosine similarity, each with score, path, and kind
  (measure/dimension/time), ready to hand to a binder.
- `mqo-catalog-embed hybrid --index <index.bin> --query "<phrase>" [--alpha 0.5]`
  → blend cosine with a lexical (BM25/FTS) score, mirroring `recall`'s hybrid
  mode, so exact-token matches still rank.
- Embedder is pluggable: default to the local BGE model `recall` uses; allow an
  `--embedder <cmd>` subprocess override. No network in the default path.
- `serve` subprocess mode exposing `search`/`hybrid` as `mqo-mcp-server` tools, a
  semantic counterpart to `search_columns`.

Deps: `serde`/`serde_json`, `clap`, the same embedding crate/path `recall` uses;
a small cosine/BM25 implementation. Index is a local file; no cluster needed.

## Acceptance criteria

1. `index` embeds every column in a `describe_model` fixture and writes a
   reloadable index file.
2. `search --query` returns top-k columns ranked by cosine; a paraphrase query
   ("profitability margin") ranks a semantically-matching measure above a
   lexically-closer-but-wrong one, on a fixture designed to show it.
3. `hybrid --alpha` blends semantic and lexical scores; `--alpha 1.0` reproduces
   pure semantic, `--alpha 0.0` pure lexical (documented + tested).
4. The default embedder path uses the local on-box model and makes **no network
   call** (verified — test passes with networking unavailable).
5. `--embedder <cmd>` override routes embedding to a subprocess and is exercised
   by a stub-embedder fixture.
6. `serve` mode answers a `search` tool call in `mqo-mcp-server` stdin/stdout JSON.
7. `cargo test` is green offline; the paraphrase-beats-keyword case is a regression
   test.
8. Output candidates are in a shape `mqo-bench` can score and a binder can consume
   (documented schema).
