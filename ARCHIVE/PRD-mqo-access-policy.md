# PRD: mqo-access-policy — route an agent to the model variant it is cleared for

Status: Draft v0.1
build_target: rust-cli
Vision: visions/atscale-ai-strategy.md
Repo: j0yen/mqo-access-policy (NOT AtScaleInc; consumes documented mqo-spec + live catalog shapes)

## TL;DR

The live semantic layer ships *governed variants* of the same model — but
nothing decides *which variant an agent identity is allowed to touch*, and one
model exposes raw PII with no safe variant at all. An enterprise cannot trust
the agent to pick the safe model; that is the classic confused-deputy hole.
`mqo-access-policy` is the authorization gate: given an agent identity and a
requested model, it returns whether access is allowed, **routes** the request to
the PII-safe variant when one exists, and **denies** specific sensitive columns
when no safe variant exists.

## Why this exists (Phase-1 evidence)

- Live MCP (`list_models`, 2026-06-16) confirms the governed-variant surface is
  real and per-engine: `internet_sales` and `internet_sales_no_pii` are *both*
  registered on `internet_sales_catalog_BigQuery` **and**
  `internet_sales_catalog_Snowflake` — four variant×engine combinations. The
  product already models "same data, PII-stripped twin," but nothing routes an
  under-cleared agent from the raw model to its `_no_pii` twin.
- Critically, the variant safety is **not uniform**: the `Tasty Bytes` model
  exposes raw PII directly — `CUSTOMER_EMAIL`, `CUSTOMER_PHONE_NUMBER`,
  `CUSTOMER_FULL_NAME`, `CUSTOMER_DOB`, `CUSTOMER_POSTAL_CODE`,
  `FRANCHISE_EMAIL`, `FRANCHISE_PHONE_NUMBER` — with **no `_no_pii` variant** in
  the catalog. So policy must do two distinct things: *route to a safe variant*
  when one exists, and *column-deny* when it does not.
- `mqo-sensitivity-scan` (Trust pillar) detects PII in a *result*; that is
  detection after a query is built. Access policy is the *pre-execution* gate
  that prevents the agent from binding to a forbidden surface in the first place
  — a different control point (prevent vs detect), consistent with end-state #2
  ("grounded before execution").
- `mqo-agent` (this fleet's keystone) needs a verdict to consult before it
  binds; without this gate the agent's only PII defense is post-hoc scanning.

## What this builds

A standalone Rust CLI `mqo-access-policy`:

- `mqo-access-policy check --identity <id> --model <model> [--engine <e>]` →
  emits `{allowed: bool, routed_model, routed_engine, denied_columns[], reason}`.
  When the identity lacks PII clearance and a `<model>_no_pii` twin exists, it
  routes there (`routed_model` differs from requested). When no twin exists, it
  returns the requested model with `denied_columns` populated from the policy's
  PII column patterns (e.g. `*_EMAIL`, `*_PHONE_NUMBER`, `*_DOB`, `*_FULL_NAME`).
- A **declarative policy file** (`policy.toml`): identities/roles → clearances
  (e.g. `pii: false`), per-model variant-routing rules, and PII column patterns.
  Bundled with a fixture policy covering the three live models.
- `mqo-access-policy variants --catalog <fixture>` → lists, per model, whether a
  `_no_pii` twin exists on each engine — derived from a catalog snapshot
  (the `list_models` shape), so the routing map is auditable and the
  "no-twin → column-deny" models are explicit.
- `mqo-access-policy enforce --identity <id> --mqo <bound.json>` → given an
  already-bound MQO, fails (non-zero) if it references a denied column or a model
  the identity may not touch, emitting which clause blocked it — the gate
  `mqo-agent` calls before execution.
- `--mock`/fixture mode: all checks run against a bundled catalog snapshot, no
  cluster, no network.
- `serve` subprocess mode exposing `check`/`enforce` as `mqo-mcp-server` tools.

## Acceptance criteria

1. `check --identity <no-pii-clearance> --model internet_sales` routes to
   `internet_sales_no_pii` (routed_model differs) with a recorded reason.
2. `check --identity <pii-cleared> --model internet_sales` allows the raw model
   unchanged (no routing).
3. `check --identity <no-pii-clearance> --model "Tasty Bytes"` (no twin exists)
   returns the same model with `denied_columns` including the email/phone/DOB
   columns from the policy patterns — proving the no-twin path column-denies.
4. `enforce --mqo <bound-referencing-denied-column>` exits non-zero and names the
   blocking clause; `enforce` on a clean MQO exits zero.
5. `variants` correctly reports which live-model fixtures have a `_no_pii` twin
   per engine and which do not.
6. The policy file is declarative and documented; an unknown identity defaults to
   the **most restrictive** clearance (deny-by-default, never fail-open).
7. Determinism: identical verdicts across runs for identical inputs.
8. `serve` answers `check`/`enforce` tool calls; `--help` documents every flag;
   all tests run cluster-free against bundled fixtures.

## Non-goals

- Not an authentication system — it consumes an already-established identity
  string; who the agent *is* is the caller's concern.
- Not PII *detection* in results — that is `mqo-sensitivity-scan` (Trust). This
  prevents binding to forbidden surfaces *before* execution.
- Does not query a live cluster to enumerate variants in tests — it reads a
  catalog snapshot fixture; a live-catalog refresh is a documented manual step.
- Does not define org-wide RBAC schemas; the policy file is intentionally small
  and model-scoped. A richer policy source is a possible follow-on.
