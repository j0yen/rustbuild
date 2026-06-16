# PRD: doxa-moral-core — the shared, framework-neutral moral vocabulary

Status: Draft v0.1
build_target: rust-cli
Vision: visions/doxa.md

## TL;DR

To compare ethical philosophies deductively, they must reason over the *same*
vocabulary. The World Ontology hardwires one philosophy's concepts into its classes;
there is no framework-neutral moral TBox that a consequentialist, a deontologist, and
a virtue ethicist could all range over. This PRD ships `doxa` (new repo) with its
foundation: a BFO-grounded, framework-neutral **moral-domain TBox** (MoralAgent,
MoralPatient, Action, Consequence, Intention, Duty, Virtue, Right, Harm, Wellbeing,
Maxim, Justice and their relations), compiled to OWL via `ousia-forge`. Every doxa
framework module (next PRD) adds axioms over *this* vocabulary, making frameworks
commensurable.

## Why this exists

Verified 2026-06-16 by reading `~/wintermute/ousia-forge/spec/roles.toml`: ethical
content is baked directly into class axioms + annotations (`AuthorityRole subclass_of
"inheres_in some (... bearer_of some Accountability)"`, annotation
`philosophicalGrounding = "Power Demands Restraint"`). The single World Ontology *is*
the Federation humanist philosophy. `ousia-forge build --spec spec/ --out x.owl`
compiles a TOML spec directory to OWL 2 DL — so the engine to compile a *neutral* moral
TBox already exists; only the neutral spec is missing. Without a shared vocabulary,
"compare two philosophies" is not even expressible.

## What this builds

A new rust-cli repo `~/wintermute/doxa` (binary `doxa`):

- **`spec-core/` TOML spec** in the `ousia-forge` format (classes with
  iri/label/parent/definition/annotations; an `ontology.toml` for namespaces + object
  properties). Framework-neutral moral-domain classes, each grounded to a BFO parent:
  - `MoralAgent ⊑ BFO:role` (an entity capable of moral action)
  - `MoralPatient ⊑ BFO:role` (an entity that can be benefited/harmed)
  - `Action ⊑ BFO:process`
  - `Consequence ⊑ BFO:process` (a downstream process of an Action)
  - `Intention ⊑ BFO:disposition` (realizable; what the agent aimed at)
  - `Duty ⊑ BFO:role`, `Right ⊑ BFO:role`
  - `Virtue ⊑ BFO:disposition`, `Vice ⊑ BFO:disposition`
  - `Harm ⊑ BFO:process`, `Wellbeing ⊑ BFO:quality`
  - `Maxim ⊑ BFO:generically_dependent_continuant` (the principle of an action)
  - `Justice ⊑ BFO:quality`
  - the *evaluative* anchor classes frameworks will define:
    `RightAction ⊑ Action`, `WrongAction ⊑ Action`, `PermissibleAction ⊑ Action`
    (declared but **left undefined** in core — each framework supplies the equivalence).
  - object properties: `hasConsequence`, `hasIntention`, `actsUpon` (Action→Patient),
    `performedBy` (Action→Agent), `expresses` (Action→Virtue), `violates` (Action→Right/Duty),
    `maximizes` (Action→Wellbeing), `instantiatesMaxim` (Action→Maxim).
- **`doxa build-core [--out core.owl]`** — shells to `ousia-forge build --spec spec-core/`
  (resolve `ousia-forge` from `$PATH` or `--forge <path>`; actionable error if absent).
- **`doxa check-core`** — `ousia-forge check --spec spec-core/`.
- The neutral stance is explicit: core declares *what an action HAS* (consequence,
  intention, maxim), never *what makes it right* — that is each framework's job.

MSRV 1.85. Deps: clap, anyhow, serde (for any config); the heavy lifting is ousia-forge.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes in `~/wintermute/doxa`.
2. `doxa build-core --out /tmp/core.owl` exits 0 and writes a non-empty OWL file
   (when `ousia-forge` is on `$PATH`; skip with a logged note if absent).
3. `ousia-forge check --spec spec-core/` (or `doxa check-core`) reports the spec valid.
4. The built core ontology contains ≥12 moral-domain classes and ≥8 object properties
   (assert via `ousia-forge stats` or a substring check on the OWL).
5. `RightAction`, `WrongAction`, `PermissibleAction` are declared as subclasses of
   `Action` but carry **no** equivalence/definition axiom in core (a test asserts the
   core OWL has no `EquivalentClasses` axiom for these three — frameworks define them).
6. Every core class has a BFO parent IRI and a non-empty definition annotation.
7. `doxa check-core` exits non-zero on a deliberately malformed spec fixture.
