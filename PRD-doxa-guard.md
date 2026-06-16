# PRD: doxa-guard — pluralist action gating under an explicit, inspectable policy

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/doxa
Vision: visions/doxa.md
Depends-on: doxa-compare (needs the multi-framework verdict set)

## TL;DR

`ousia-guard` returns `allow | flag | deny` according to *one* philosophy. With doxa's
many frameworks and the comparison layer, this PRD ships `doxa guard --scenario <abox>
--policy <policy>`, which aggregates the frameworks' verdicts into a single
`allow | flag | deny` under an **explicit, user-chosen** aggregation policy — unanimity,
majority, a designated framework, or a lexical priority order. The verdict always names
the policy and shows the per-framework breakdown, so the ethics being applied is never
hidden.

## Why this exists

Verified 2026-06-16: the ousia vision (§"Components") describes `ousia-guard` returning
`allow | flag | deny` with a justifying axiom chain — for the single World Ontology
philosophy. doxa makes the philosophy plural; the open question this PRD answers is *how
a pluralist system decides to act* when its frameworks disagree (which doxa-compare shows
they do, on the trolley scenario). The honest answer is not "doxa picks the right ethics"
— it is "the user picks an explicit aggregation policy, and doxa applies it transparently."
This keeps alignment inspectable: you can always see which frameworks dissented and under
what rule they were overruled.

## What this builds

Extend `~/wintermute/doxa`:

- **`doxa guard --scenario <abox.ttl> --policy <policy> [--frameworks <list>]`** where
  `<policy>` is one of:
  - `unanimity` — `allow` only if every decided framework says `RightAction`/`Permissible`;
    `deny` if any says `WrongAction`; else `flag`. (The cautious pluralist.)
  - `majority` — verdict by majority of decided frameworks; ties → `flag`.
  - `framework:<name>` — defer to one named tradition (e.g. `framework:deontology`);
    equivalent to single-philosophy guard but explicit about which.
  - `lexical:<a,b,c>` — apply frameworks in priority order; first one that decides
    (not `undetermined`) wins. (Encodes a ranked-pluralism / Rawlsian-lexical stance.)
- Maps the aggregated framework verdicts to the guard triad:
  `RightAction`/`PermissibleAction` → `allow`; `WrongAction` → `deny`;
  no decisive verdict under the policy → `flag`.
- **Output** always includes: the final `allow|flag|deny`, the policy applied, the
  per-framework verdicts (from doxa-compare), and which frameworks dissented from the
  aggregate. With `--explain`, include each deciding framework's axiom chain.
- Reuse `ousia-guard` per framework where its action-evaluation is richer than the bare
  `RightAction` classification; otherwise reuse doxa-reason's verdicts. Resolve
  `ousia-guard`/`ousia-reason` from `$PATH`; actionable error if absent.
- Exit code reflects the verdict (`allow`=0, `flag`=10, `deny`=20) so it can gate a
  pipeline.

MSRV 1.85.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `doxa guard --scenario scenarios/trolley.ttl --policy unanimity` returns `flag` or
   `deny` (the frameworks conflict on trolley, so unanimity cannot `allow`) and names the
   per-framework split (skip live reasoning if engines absent; unit-test aggregation logic).
3. `--policy majority` on the same scenario returns the majority verdict and reports the
   dissenters.
4. `--policy framework:deontology` returns exactly deontology's verdict mapped to the
   triad, and the output states the policy.
5. `--policy lexical:consequentialism,deontology` returns consequentialism's verdict when
   consequentialism decides; falls through to deontology only when consequentialism is
   `undetermined` (a test exercises both branches).
6. Every output names the policy applied and lists per-framework verdicts — no silent
   aggregation (a test asserts the breakdown is always present).
7. Exit codes map allow/flag/deny → 0/10/20.
8. An unknown policy string → actionable error listing valid policies, non-zero exit.
