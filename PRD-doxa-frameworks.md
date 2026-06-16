# PRD: doxa-frameworks — the three normative families as auditable axiom modules

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/doxa
Vision: visions/doxa.md
Depends-on: doxa-moral-core (the shared vocabulary)

## TL;DR

With the framework-neutral moral TBox in place (doxa-moral-core), this PRD ships the
first three philosophical frameworks as separate, auditable axiom modules — each
defining `RightAction` *its own way* over the shared vocabulary:
**consequentialism**, **deontology**, and **virtue ethics** (the three major normative
families). Each is a TOML axiom module the user can read and amend; `doxa list` shows
the catalog and `doxa build <framework>` compiles moral-core + that framework's axioms
into a per-framework OWL.

## Why this exists

The doxa vision's whole premise is that a philosophy should be a swappable module, not
a hardcode. doxa-moral-core declares `RightAction`/`WrongAction`/`PermissibleAction`
*without* definitions precisely so frameworks can supply them. This PRD supplies the
first three — chosen because they are the standard tripartite division of normative
ethics (consequentialist / deontological / aretaic) and they genuinely *disagree*, so
the later `doxa-compare` has real divergence to surface. Formulations follow the
Stanford Encyclopedia of Philosophy entries (consequentialism, deontological ethics,
virtue ethics) and the primary sources (Mill, Kant, Aristotle).

## What this builds

Extend `~/wintermute/doxa`:

- **`frameworks/<name>/` directories**, each an `ousia-forge`-format axiom module that
  imports/extends the moral-core vocabulary and adds the framework's defining axioms:
  - **`frameworks/consequentialism/`** — `RightAction ≡ Action and (maximizes some
    Wellbeing)`; the morally relevant feature is `hasConsequence`; intention is
    bracketed. Annotation `philosophicalGrounding` citing Mill's greatest-happiness
    principle (SEP: Consequentialism §1).
  - **`frameworks/deontology/`** — `RightAction ≡ Action and (instantiatesMaxim some
    UniversalizableMaxim) and not (actsUpon some (MoralPatient and treatedMerelyAsMeans))`;
    the relevant feature is the maxim + respect for persons, not the consequence.
    Annotation citing Kant's categorical imperative (Formula of Universal Law +
    Formula of Humanity; SEP: Deontological Ethics §1, Kant §5).
  - **`frameworks/virtue-ethics/`** — `RightAction ≡ Action and (expresses some Virtue)
    and not (expresses some Vice)`; the relevant feature is the character the action
    expresses. Annotation citing Aristotle's *Nicomachean Ethics* (the phronimos
    standard; SEP: Virtue Ethics §2).
  - Each framework declares any framework-specific helper classes it needs
    (`UniversalizableMaxim`, `treatedMerelyAsMeans`, the specific virtues) so the core
    stays neutral.
- **`doxa list`** — list available frameworks (scan `frameworks/`), one line each with
  the framework's one-sentence thesis from its annotation.
- **`doxa build <framework> [--out <fw>.owl]`** — forge moral-core + the named
  framework's module into one OWL via `ousia-forge` (combine the spec dirs, or forge
  each and merge; reuse the build-core path from doxa-moral-core).
- **`doxa build --all`** — build every framework.

MSRV 1.85. The heavy lifting stays in `ousia-forge`; this PRD is spec authoring + the
`list`/`build` plumbing.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `doxa list` shows at least `consequentialism`, `deontology`, `virtue-ethics`, each
   with a non-empty one-line thesis.
3. `doxa build consequentialism --out /tmp/c.owl` produces an OWL file in which
   `RightAction` carries an `EquivalentClasses` axiom referencing `maximizes`/`Wellbeing`
   (skip the build if `ousia-forge` absent, but unit-test that the spec parses).
4. `doxa build deontology` produces an OWL where `RightAction`'s definition references a
   maxim/universalizability class and the "as ends" constraint — and is *structurally
   different* from consequentialism's `RightAction` definition (a test asserts the two
   equivalence axioms differ).
5. `doxa build virtue-ethics` produces an OWL where `RightAction` references
   `expresses`/`Virtue`.
6. Each framework module has a `philosophicalGrounding` annotation citing its source
   (Mill / Kant / Aristotle) — verified by substring presence in the built OWL.
7. `ousia-forge check` passes on all three framework spec dirs.
8. `doxa build --all` builds all three without error.
