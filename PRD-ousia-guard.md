# PRD: ousia-guard — deductive ethical gate for agent actions

Status: Draft v0.1
build_target: rust-cli
Vision: visions/ousia.md

## TL;DR

This is the keystone of the seed — "making ethical AI possible." `ousia-guard`
takes a description of a proposed agent action (as RDF or JSON), evaluates it
against the World Ontology's reasoned graph, and returns `allow | flag | deny`
**together with the axiom chain that justifies the verdict**: does the action
violate a Right, push a SentientBeing below the Dignity floor, or vest an
AuthorityRole without Accountability? Alignment becomes a deductive check an
agent runs *before* acting — the paper's §9.1 thesis turned into a callable
gate.

## Why this exists

- **The paper proposes exactly this and stops at proposal.** §9.1: provide AI
  systems "with formally structured ethical knowledge they can reason over
  deductively rather than learned statistically… The two approaches reinforce
  each other." Nobody has built the runtime check. guard is it.
- **The dignity floor is stated as an operational rule.** §5.2's `aiGuidance`:
  *"dignity imposes an inviolable floor below which no utilitarian calculation
  may push."* That is a guard rule, verbatim, waiting to be enforced.
- **This laptop is the first customer.** Joe runs a fleet of autonomous Claude
  build/dream sessions (manifest.json: 10 in-flight PRDs, /build + /dream
  timers). A deductive pre-action ethics gate is directly dogfoodable as a hook
  — the honest, motivated first deployment, and the strongest "it works" demo
  for the market story.

## What this builds

A `cargo` Rust CLI + lib `ousia-guard` at `~/wintermute/ousia-guard/`.

**Deps (pinned at build time):**
- `ousia-sparql` (lib) — the reasoned query substrate.
- `ousia-reason` (lib) — to classify the action's described participants.
- `clap`, `serde`, `serde_json`.

**Action model.** An action is described as a small RDF/JSON document: the
process type, its participants (with their asserted qualities/roles), and what
it `affects`/`violates`/`enables`. guard loads this as an ABox fragment, lets
reason classify it, then runs a fixed battery of **rule checks** expressed as
SPARQL ASK queries against the reasoned graph:
- **dignity-floor**: does the action harm an individual that bears Dignity
  (via materialized sentience→dignity)? → deny.
- **rights-violation**: is the action an InjusticeProcess subtype, or does it
  `violates some Right`? → deny.
- **unaccountable-authority**: does it grant/exercise an AuthorityRole on an
  entity not bearing Accountability? → flag (open-world: missing, not denied).
- **welfare-undermine**: does it remove a WelfareCondition required for some
  Flourishing the subject participates in? → flag.

**Verdict.** `deny` if any deny-rule fires; `flag` if any flag-rule fires and no
deny; else `allow`. Output carries, per fired rule, the `explain` axiom chain
from ousia-reason.

**UX.**
```
ousia-guard check --owl world-ontology.owl --action action.json   # → verdict + JSON
ousia-guard check --action action.json --format json --explain
ousia-guard rules                                                 # list rule battery + paper refs
```

## Acceptance criteria

1. An action harming an individual the reasoner classifies as bearing Dignity
   returns `deny` with the `dignity-floor` rule and the sentience→dignity axiom
   chain in the justification.
2. An action typed as an InjusticeProcess subtype (Oppression, Discrimination,
   Exploitation, Censorship) returns `deny` with `rights-violation` citing
   §5.7 / §6.6.
3. An action exercising an AuthorityRole on an entity with no asserted
   Accountability returns `flag` (not deny) — respecting the paper's open-world
   assumption (§5.3, §7.2), and the justification says membership is unproven,
   not refuted.
4. A benign action affecting only non-sentient entities returns `allow`.
5. `--format json` emits a machine-readable verdict: `{verdict, rules_fired[],
   justifications[]}` where each justification is the ordered axiom chain.
6. The verdict precedence is exactly deny > flag > allow, covered by a test with
   an action that fires both a deny-rule and a flag-rule (→ deny).
7. `ousia-guard rules` lists the rule battery with each rule's paper-section
   citation.
8. `cargo test` covers one fixture action per rule (deny dignity, deny rights,
   flag authority, flag welfare, allow benign) and the precedence case. A
   malformed action document returns a clean error, not a panic.
9. Ships a documented example wiring guard as a pre-action check (e.g. a
   `PreToolUse`-shaped hook script under `examples/`) so the dogfood path is
   reproducible — no settings.json changes made by the PRD itself.
