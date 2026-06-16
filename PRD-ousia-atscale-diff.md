# PRD: ousia-atscale-diff — make "is this the same revenue?" an executable command

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/ousia-atscale
Vision: visions/ousia-atscale.md

## TL;DR

The MARKET.md pitch opens with the question semantic layers can't answer: *"is this
the same revenue as last quarter?"* — and claims BFO grounding gives a formal answer.
But there is no command that compares two models' groundings. This PRD ships
`ousia-atscale diff modelA.json modelB.json`, which compares the BFO grounding of
two models element-by-element and reports: **agreements** (same name, same BFO
category), **divergences** (same name, *different* BFO category — a semantic red
flag), and **unique** elements (present in only one model). The headline pitch
becomes a runnable command.

## Why this exists

Verified 2026-06-16: `src/main.rs` has only `ground`/`annotate`/`report`. The
MARKET.md pitch (committed in the repo) leads with "is this the same revenue as last
quarter? there is no formal answer" and positions BFO grounding as that answer — yet
ships no comparison command. For colleagues evaluating the tool, a `diff` that flags
when two models disagree on what `revenue` *is* (e.g. one grounds it as a measure
GDC, another as a Quality) is the single most convincing demonstration of why formal
grounding matters. Two real fixtures already exist (`sales_model.json`,
`finance_model.json`) — both have `revenue`-flavored measures to compare.

## What this builds

Extend `~/wintermute/ousia-atscale`:

- **New subcommand `diff`** in `src/main.rs`:
  ```
  ousia-atscale diff --a modelA.json --b modelB.json [--format text|json]
  ```
  (positional `<A> <B>` acceptable too; keep consistent with the crate's clap style).
- **New module `src/diff.rs`**: ground both models (reuse `Mapper`), join element
  lists by **name** (case-insensitive), and classify each joined pair:
  - `agree` — same name, same BFO category
  - `diverge` — same name, different BFO category (the important case; include both
    categories + both rationales)
  - `only_in_a` / `only_in_b` — name present in one model only
- **Text output**: a summary line (`N agree, M diverge, P unique`) followed by a
  table of the divergences and uniques (agreements collapsed to a count unless
  `--verbose`).
- **JSON output**: a structured `{agree:[], diverge:[], only_in_a:[], only_in_b:[]}`
  for machine consumption (and for the future MCP `serve` path).
- Exit code: 0 if no divergences, non-zero if ≥1 divergence — so `diff` can gate a
  CI check ("our quarterly model must not silently redefine revenue").

No change to existing subcommands. No new heavy deps. MSRV stays 1.85.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `ousia-atscale diff --a fixtures/sales_model.json --b fixtures/sales_model.json`
   reports all-agree, zero divergences, and exits 0 (a model is identical to itself).
3. `ousia-atscale diff --a fixtures/sales_model.json --b fixtures/finance_model.json`
   runs, prints a summary line with agree/diverge/unique counts, and lists the
   unique elements of each.
4. A crafted pair where the same column name is grounded differently (e.g. via
   `bfo_hint`, or a name that triggers the Quality/Role heuristic in one but not the
   other) is reported under `diverge` with both categories shown, and the process
   exits non-zero.
5. `--format json` emits valid JSON with the four keys; a test round-trips it.
6. The join is case-insensitive on element name (a test with `Revenue` vs `revenue`
   matches them).
7. Divergence count drives the exit code (0 ⇔ no divergences).
