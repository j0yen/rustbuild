# PRD: mqo-template — parameterized, reusable saved MQOs

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mqo-template
Vision: visions/mqo-tools.md

## TL;DR

An agent that asks "monthly revenue by region" this session will rebuild the same
MQO shape from scratch next session — re-selecting the measure, the date grain,
the dimension, the filter structure — and may get it subtly different each time.
The `mqo-mcp` workspace has no notion of a saved, parameterized query. This PRD
ships `mqo-template`, whose `list` and `instantiate` subcommands manage a library
of MQO templates with named parameter slots (`{{date_range}}`, `{{measure}}`,
`{{region}}`) and produce a concrete, ready-to-bind MQO when given bindings.

## Why this exists

Verified 2026-06-15: no crate in `joeyen-atscale/mqo-mcp` matches `templ` and the
server tool registry has no save/instantiate concept — every MQO is built fresh
per call. `mqo-next-query-proposer` proposes *follow-up* MQOs from adjacency, and
`mcp-session-journal` snapshots session mutations for replay — neither offers a
reusable parameterized query shape. Templates are the standard BI-tooling answer
to "the same question, repeatedly, consistently": they make a query shape a
durable, reviewable artifact instead of a per-session reconstruction (which the
README's thesis says is exactly where silent coherent-but-wrong errors creep in).

## What this builds

New repo `joeyen-atscale/mqo-template` (binary `mqo-template`):

- **Template format**: a JSON MQO (the documented `mqo-spec` shape) with
  `{{param}}` placeholders in value positions, accompanied by a `params` block
  declaring each slot's `{name, type, required, default?, description}`. Stored
  one-per-file under a templates dir (default `~/.config/mqo-template/` or
  `--dir`).
- **`mqo-template list [--dir <d>] [--format text|json]`** — list available
  templates, one line each with name + one-line description + declared params.
- **`mqo-template show <name> [--dir <d>]`** — print a template's MQO skeleton +
  its param declarations.
- **`mqo-template instantiate <name> --params <file|inline-json> [--dir <d>]
  [--format json]`** — substitute each `{{param}}` with its binding, validate that
  all `required` params are supplied and types match the declaration, and emit a
  concrete MQO ready for `query_multidimensional`. A missing required param or a
  type mismatch → actionable error naming the param, non-zero exit. Unknown
  params (supplied but not declared) → error (no silent drop).
- **`mqo-template serve`** — `{"tool":"instantiate_template","args":{"name":…,
  "params":…}}` → `{"ok":true,"data":{mqo}}`; also `{"tool":"list_templates"}`.
- Ships `templates/` with at least two example templates (e.g.
  `revenue_by_region_monthly`, `topn_products_by_measure`) for tests.

MSRV 1.85. Deps: clap, anyhow, serde/serde_json. Substitution is structural over
the JSON value tree (not text templating) so types are preserved — a `{{top_n}}`
slot binds to an integer, not a quoted string. `#![forbid(unsafe_code)]`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `list` over the shipped `templates/` dir shows ≥2 templates each with a
   non-empty description and declared params — a test asserts the count and that
   params are listed.
3. `instantiate revenue_by_region_monthly --params '{"date_range":["2026-01-01",
   "2026-03-31"],"region":"EMEA"}'` emits a concrete MQO with no remaining
   `{{...}}` placeholders — a test asserts no placeholder survives and the bound
   values appear in the correct positions.
4. Structural typing is preserved: a `{{top_n}}` slot declared `integer` binds to a
   JSON number, not a string — a test asserts the emitted MQO has an integer there.
5. A missing required param → actionable error naming the param, non-zero exit — a
   test asserts the message and exit code.
6. A supplied-but-undeclared param → error (no silent drop) — a test asserts this.
7. `serve` handles `instantiate_template` and `list_templates`, emitting
   `{ok,data}`; unknown tool → `{ok:false,error}` non-zero — tests assert both.
8. `--format json` round-trips through `serde_json` for both `list` and
   `instantiate`.
