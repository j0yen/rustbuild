# PRD: mqo-error-explain — turn backend faults into actionable causes

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-error-explain
Vision: visions/mqo-trust.md

## TL;DR

When a query fails, the backend returns a raw fault — an XMLA HRESULT, a DAX
"cannot find name" error, a PGWire syntax error — that an AI agent cannot act on
and a business user cannot read. The agent typically retries blindly or surfaces
the cryptic string. This PRD ships `mqo-error-explain`, whose `explain` subcommand
maps a backend fault to a structured `{cause, category, suggested_fix}` drawn from
a known-fault catalog, so a failure becomes a next action instead of a dead end.

## Why this exists

Verified 2026-06-15: the workspace has a live-execution path (`mqo-auth-bridge` +
`LiveExecutor` over XMLA/PGWire per ARCHITECTURE.md) but no crate matches
`error|explain` for fault interpretation — failures pass through raw. The README's
governance thesis is about preventing silent-wrong answers; the complementary gap
is loud-but-opaque *failures*. A fault catalog (data, not code) that recognizes the
common XMLA/DAX/MDX/SQL faults and maps each to a plain-language cause and a
concrete fix (e.g. "the date hierarchy has no Day level — query at Month") closes
that gap and is the kind of institutional knowledge that should be captured once
and reused.

## What this builds

New repo `joeyen-atscale/mqo-error-explain` (binary `mqo-error-explain`):

- **`catalog/faults.toml`** — the fault catalog (data): each entry is `{id,
  match: [regex|substring], backend, category, cause, suggested_fix, doc_url?}`.
  Seed with the common faults: XMLA HRESULT classes, DAX `cannot find name` /
  unresolved column, MDX member-not-found, PGWire syntax/permission, cardinality/
  timeout, auth/token-expired. The catalog grows by editing data, not code.
- **`mqo-error-explain explain --error <text|file> [--backend dax|mdx|sql|xmla]
  [--catalog <file>] [--format json]`**:
  - Match the fault text against the catalog (most-specific match wins; `--backend`
    narrows the candidate entries).
  - Emit `{matched: bool, id?, category, cause, suggested_fix, doc_url?,
    raw: <original>}`.
  - On no match: `{matched:false, category:"unknown", cause:"unrecognized fault",
    suggested_fix:"<generic guidance>", raw}` — never fabricate a specific cause.
  - Exit 0 always (explanation is informational).
- **`mqo-error-explain serve`** — `{"tool":"explain_error","args":{"error":…,
  "backend":…}}` → `{"ok":true,"data":{…}}`.
- Ships `fixtures/`: a set of real-shaped fault strings (one per seeded category)
  for tests.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json, regex, toml.
`#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `explain --error "<DAX cannot find name 'Reveue'>"` returns
   `category:"unresolved_name"` (or equivalent) with a `suggested_fix` mentioning
   checking the measure name — a test asserts the category and a non-empty fix.
3. A `--backend xmla` HRESULT fault matches an XMLA catalog entry and not a DAX one
   — a test asserts backend narrowing changes the match.
4. An unrecognized fault returns `{matched:false, category:"unknown"}` with generic
   guidance and never a fabricated specific cause — a test asserts the unknown path.
5. Catalog matching is most-specific-wins: when two entries could match, the one
   with the more specific pattern is chosen — a test asserts precedence on a
   crafted overlap.
6. The catalog is pure data: adding a new `faults.toml` entry makes a previously
   `unknown` fault resolve, with no code change — a test loads an extended catalog
   fixture and asserts the new match.
7. `--format json` emits the documented shape and round-trips; `serve` handles
   `explain_error` and returns `{ok,data}`, unknown tool → `{ok:false,error}`
   non-zero — tests assert all three.
8. A `--catalog` file that is malformed TOML → actionable error naming the parse
   problem, non-zero exit — a test asserts graceful handling.
