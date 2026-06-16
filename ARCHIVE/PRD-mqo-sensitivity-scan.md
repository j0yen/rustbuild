# PRD: mqo-sensitivity-scan — flag and redact sensitive fields before rows reach the model

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-sensitivity-scan
Vision: visions/mqo-trust.md

## TL;DR

`mqo-mcp` keeps result rows server-side behind handles — but when rows *do* come
inline or via export, nothing checks whether a column is sensitive (PII: SSN,
email, name, account number) before it lands in the model's context and, from
there, possibly a chat transcript or log. This PRD ships `mqo-sensitivity-scan`,
whose `scan` subcommand flags sensitive fields in a model or result by tag map and
name/value patterns, with an optional `--redact` that masks the values before they
ever reach the model.

## Why this exists

Verified 2026-06-15: nothing in the 50-crate workspace matches `pii|sensitiv|
redact|mask`. The README emphasizes read-only governance ("the semantic layer is
the contract") but the contract has no data-sensitivity dimension — a measure or
attribute carrying PII is treated like any other. For an AI agent on production
data, an un-flagged sensitive column reaching the context window is a real
exfiltration/compliance risk, and the cheapest place to stop it is a scan over the
result/model before the rows are returned. This is the missing data-governance
gate, and it pairs with the existing handle protocol (scan a handle export before
inlining).

## What this builds

New repo `joeyen-atscale/mqo-sensitivity-scan` (binary `mqo-sensitivity-scan`):

- **`mqo-sensitivity-scan scan [--model <file>] [--rows <file>] [--tags <file>]
  [--redact] [--format json]`**:
  - Detect sensitive fields by three layered signals:
    - **Tag map** (`--tags`): explicit `{field: sensitivity_class}` (the
      authoritative source — e.g. from `describe_model` column tags or a policy
      file).
    - **Name patterns**: field names matching known PII patterns (ssn, email,
      phone, dob, account, name, address, salary) — a built-in, extensible regex
      set.
    - **Value patterns** (only with `--rows`): values matching PII regexes
      (email, SSN-like, credit-card-like via Luhn) sampled from the rowset.
  - Emit `{sensitive_fields: [{field, class, matched_by, sample_count}],
    clean: bool}`.
  - With `--redact` (requires `--rows`): return the rowset with each sensitive
    field's values masked (`***`, or last-4 for account-like), so the caller can
    inline a safe result. The redaction is deterministic and reversible only by
    re-querying (no key stored).
  - Exit 0 if clean; non-zero if any sensitive field is found *and* `--fail-on-find`
    is set (default: report but don't fail, so scanning is non-disruptive).
- **`mqo-sensitivity-scan serve`** — `{"tool":"scan_sensitivity","args":{"model":…,
  "rows":…,"redact":bool}}` → `{"ok":true,"data":{sensitive_fields,clean,rows?}}`.
- Ships `fixtures/`: a model + rows with a planted email/SSN column and a clean set.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json, regex. `#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `scan --rows fixtures/pii.json` flags the planted email and SSN-like columns
   with their `class` and `matched_by` — a test asserts both fields appear.
3. `scan --model fixtures/model.json` flags name-pattern fields (e.g. a
   `customer_ssn` measure) without needing values — a test asserts name-based
   detection.
4. `--tags fixtures/tags.json` flags a field the patterns would miss (an opaquely
   named sensitive column) — a test asserts the tag-map source is authoritative.
5. `--redact --rows fixtures/pii.json` returns rows where sensitive values are
   masked and non-sensitive values are untouched — a test asserts masking and
   non-masking per column.
6. A clean dataset returns `{clean:true, sensitive_fields:[]}` and exits 0 — a
   test asserts the clean path.
7. `--fail-on-find` exits non-zero when sensitive fields are present and 0 when
   clean — tests assert both.
8. `serve` handles `scan_sensitivity` and emits `{ok,data}`; unknown tool →
   `{ok:false,error}` non-zero — tests assert both. Value-pattern detection uses
   Luhn for card-like numbers (a test asserts a non-Luhn 16-digit string is not
   flagged as a card, avoiding false positives).
