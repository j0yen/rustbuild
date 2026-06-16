# PRD: mqo-result-cache — one query cost for many phrasings of the same question

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-result-cache
Vision: visions/mqo-trust.md

## TL;DR

Two users (or one agent across turns) asking "revenue by region last quarter" and
"last quarter's regional revenue" produce the same BoundMqo — but `mqo-mcp`
executes both against the cluster, paying the query cost twice. This PRD ships
`mqo-result-cache`, a content-addressed cache keyed on the *canonicalized BoundMqo*
(not the raw question), so semantically identical queries share one cached result —
cheaper, faster, and consistent (the same question never returns two different
answers within the cache window).

## Why this exists

Verified 2026-06-15: nothing in the 50-crate workspace matches `cache`. The handle
store (`mqo-duckdb-handle-store`) persists a *result* behind a handle for follow-up
ops, but there is no cache *across queries* — a fresh `query_multidimensional` with
the same bound shape re-executes. Because the pipeline already produces a fully
resolved BoundMqo (every ref an exact `unique_name`), the canonical cache key is
right there: serialize the BoundMqo deterministically and hash it. Differently
phrased questions that bind identically collapse to one key. This is a pure
efficiency-and-consistency layer that needs no cluster and composes ahead of
execution.

## What this builds

New repo `joeyen-atscale/mqo-result-cache` (binary `mqo-result-cache`):

- **Canonical key**: `mqo-result-cache key --bound <file>` — canonicalize the
  BoundMqo (sort field lists, normalize filter order, drop cosmetic fields) and
  emit a stable content hash. Two BoundMqos that differ only in cosmetic ordering
  produce the **same** key.
- **Cache ops** (on-disk under `--dir`, default `~/.cache/mqo-result-cache/`):
  - `mqo-result-cache get --bound <file> [--max-age <dur>]` — return the cached
    result if present and within `--max-age`, else exit non-zero with a
    cache-miss marker.
  - `mqo-result-cache put --bound <file> --result <file>` — store a result under
    the bound's canonical key with a timestamp.
  - `mqo-result-cache stats` / `purge [--older-than <dur>]` — cache introspection
    and eviction.
- **`mqo-result-cache serve`** — `{"tool":"cache_get","args":{"bound":…,"max_age":…}}`
  → `{"ok":true,"data":{hit:bool, result?, age_s?}}`; `{"tool":"cache_put",…}` →
  `{"ok":true,"data":{key}}`.
- Honor a `--no-store` / sensitivity flag so results flagged sensitive (by
  `mqo-sensitivity-scan`) are never cached to disk — a documented safety interlock.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json, sha2, time. `#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `key --bound A` and `key --bound B`, where A and B are the same BoundMqo with
   field lists and filters in different order, produce the **same** hash — a test
   asserts key equality under reordering.
3. Two BoundMqos that differ in an actual selection (different measure) produce
   **different** keys — a test asserts key inequality.
4. `put` then `get` for the same bound returns the stored result with an `age_s`;
   a `get` for an absent bound is a cache miss (non-zero exit / `hit:false`) —
   tests assert hit and miss.
5. `get --max-age 0s` treats any stored entry as expired (miss); a generous
   `--max-age` returns the hit — a test asserts age-based expiry.
6. `purge --older-than <dur>` removes only entries older than the duration — a test
   asserts selective eviction.
7. `--no-store` on `put` (sensitivity interlock) does not write to disk and the
   subsequent `get` misses — a test asserts the interlock prevents caching.
8. `serve` handles `cache_get`/`cache_put` and emits `{ok,data}`; unknown tool →
   `{ok:false,error}` non-zero — tests assert both. `--format json` round-trips.
