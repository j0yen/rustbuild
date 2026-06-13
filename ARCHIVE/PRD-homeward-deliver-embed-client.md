# PRD: homeward-deliver-embed-client — one embed client, shared by both halves

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

The async client for homeward's Python DINOv2 sidecar (`/enroll`, `/query`,
`/health`) lives inside `homeward-ingest` as `embed_client.rs`. The owner-facing
`homeward-report` crate needs the very same client to embed a lost-pet query
photo — but making `homeward-report` depend on the entire `homeward-ingest`
daemon crate (sqlite store, orchestrator, departure detection, connectors) to
reach one HTTP client is wrong. This PRD lifts the client into a small, focused
`homeward-embed-client` workspace crate that both ingest and report depend on.
It is a pure extract-and-rewire: no new behavior, same wire protocol.

## Why this exists

Phase-1 live inspection (2026-06-13, `~/wintermute/homeward`):

- `homeward-ingest/src/embed_client.rs` is a complete async client: it documents
  three endpoints (`POST /enroll` — embed+index an intake photo; `POST /query` —
  embed a query photo + kNN; health), `EmbedClientConfig::from_env`, typed
  `EmbedClientError`, built on `reqwest::Client` (`embed_client.rs:1-51`).
- It is **only reachable through `homeward-ingest`**. `homeward-report/Cargo.toml`
  depends on neither it nor `homeward-match` (confirmed: empty grep). The deliver
  fleet's owner-side wire (`homeward-deliver-query`) must call `/query` from
  `homeward-report`; pulling `homeward-ingest` in for that is a heavy, circular-
  smelling dependency (ingest already conceptually sits *downstream* of schema).
- The client is currently **dead code** — never instantiated anywhere in the
  workspace (no `EmbedClient::new`, `.enroll(`, `.query(` outside its own file).
  Extracting it now, before two callers wire it, avoids a later two-caller
  refactor and gives both deliver wires one honest dependency target.

This is a `rust-extend` of the `~/wintermute/homeward` workspace (adds a member).

## What this builds

- **A new workspace crate `homeward-embed-client`** (added to `Cargo.toml`
  `members`), library-only, edition 2024, rust-version 1.85, inheriting the
  workspace lints (`unsafe_code = "deny"`, `missing_docs = "warn"`).
- **The client, moved verbatim in behavior**: `EmbedClient`, `EmbedClientConfig`
  (`from_env`), `QueryRequest`/`EnrollRequest` and their response types, and
  `EmbedClientError`, relocated from `homeward-ingest/src/embed_client.rs` into
  the new crate's `lib.rs`. Same endpoint paths, same env var names, same JSON
  shapes — the Python sidecar contract (`homeward/embed/homeward_embed/
  service.py`) is unchanged.
- **`homeward-ingest` rewired** to depend on `homeward-embed-client` and
  re-export the client types it previously owned (so existing
  `homeward_ingest::embed_client::*` paths, if referenced by tests, keep
  resolving via a `pub use`). The old `embed_client.rs` module is deleted, not
  left as a duplicate.
- **No new transport, no protocol change.** This PRD does not call the client;
  wiring is `homeward-deliver-enroll` (ingest side) and `homeward-deliver-query`
  (report side).

## Acceptance criteria

1. `homeward-embed-client` exists as a workspace member; `cargo build` and
   `cargo test` pass for the whole workspace.
2. `EmbedClient`, `EmbedClientConfig::from_env`, the enroll/query request +
   response types, and `EmbedClientError` are public from `homeward-embed-client`
   and carry doc comments (workspace `missing_docs` clean).
3. `homeward-ingest` no longer defines its own `embed_client` module; it depends
   on `homeward-embed-client`. A `pub use` re-export keeps
   `homeward_ingest::embed_client::EmbedClient` resolving for any existing
   reference (proven by an in-crate doc-test or unit reference).
4. The endpoint paths (`/enroll`, `/query`, health), env var names, and request/
   response JSON field names are byte-for-byte the same as before the move
   (a serde round-trip test asserts the JSON shape of each request type).
5. `EmbedClientConfig::from_env` with no env set yields the documented localhost
   default; a malformed base URL yields a typed `EmbedClientError`, not a panic.
6. `cargo clippy` over the new crate is clean at the workspace lint level
   (no new `-D` regressions versus the baseline).
