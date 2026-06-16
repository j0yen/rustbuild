# Vision: doxa — ousia holds one philosophy; let it hold them all

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-16
**Status:** active
**Fleet 1 drafted:** 5 PRDs
**Seed:** jsy (2026-06-16) — `/dream of extending ousia to all knowledge and philosophies`.

*δόξα (doxa)* — in Greek philosophy, the realm of *belief and opinion*, the many
contending views about how things are and ought to be. It is the natural complement
to *ousia* (being/substance): ousia is what **is**; doxa is the manifold of views
about what one **ought to do**. This vision makes the ousia reasoner hold not one
ethics but many.

## TL;DR

The seed has two halves. **"All knowledge"** is already [lattice](lattice.md)'s job —
federating the 500+ BFO-conformant domain ontologies into one traversable graph
(lattice Fleet 2 was dreamed 2026-06-16). This vision does *not* duplicate that; a
doxa framework reasoning about a real scenario can pull domain facts *via* lattice.

**"All philosophies"** is genuinely uncovered. Verified 2026-06-16 by reading
`ousia-forge/spec/roles.toml`: the World Ontology hardwires exactly **one** ethical
philosophy directly into its class axioms and annotations — the Federation
secular-humanist one ("Power Demands Restraint", "Justice Is Non-Negotiable", the ten
axioms of `~/Notes/federation-utopian-philosophy.md`). `ousia-guard` returns
`allow | flag | deny` according to *that* single stance. There is no way to ask "what
would a *consequentialist* conclude? a *Kantian*? a *virtue ethicist*?" — or to see
where the great ethical traditions **agree** (an overlapping consensus) and where they
**conflict** (the real moral dilemma).

`doxa` factors ethics into modules. A shared, framework-neutral **moral-domain TBox**
(MoralAgent, Action, Consequence, Intention, Duty, Virtue, Right, Harm, Wellbeing,
Maxim, Justice — all BFO-grounded) gives every philosophy a common vocabulary to range
over. Each **philosophical framework** — consequentialism, deontology, virtue ethics,
and beyond — is then a separate, auditable axiom module that defines `RightAction` *its*
way. The reasoner can then evaluate a scenario *within* a chosen framework, *compare*
frameworks side-by-side, and let a pluralist guard aggregate their verdicts by an
explicit policy (unanimity, majority, a chosen tradition, or lexical priority).

## End-state

When doxa is fulfilled:

1. **There is a shared moral vocabulary.** A BFO-grounded, framework-neutral TBox of
   moral concepts that every framework's axioms reference — so frameworks are
   *commensurable* (they reason over the same `Action`, the same `Consequence`).
2. **Each philosophy is a module, not a hardcode.** Consequentialism, deontology, and
   virtue ethics each ship as an auditable TOML axiom module defining `RightAction`
   per their tradition, citing the standard formulation (SEP / primary texts).
3. **The reasoner answers per-framework.** `doxa reason consequentialism --scenario S`
   returns whether the action is Right *under consequentialism*, with the axiom chain.
4. **Frameworks can be compared.** `doxa compare consequentialism deontology
   virtue-ethics --scenario S` returns an agreement/conflict matrix — converging
   verdicts are the overlapping consensus; diverging ones are the dilemma surface.
5. **The guard is pluralist.** `doxa guard --scenario S --policy unanimity|majority|
   framework:<name>|lexical:<order>` aggregates the frameworks' verdicts under an
   explicit, inspectable policy — alignment that is honest about which ethics it applies.

## Why this is distinct from the existing ethics arc

- **ousia** = the *single* World Ontology and its reasoner/guard for *one* philosophy.
  doxa makes the philosophy a swappable module and adds the comparison layer ousia lacks.
- **tribunal** = verifies ousia's verdicts against a held-out corpus (still one stance).
- **herald / recourse** = ship and appeal *that* one `/conscience`. doxa is upstream of
  all of them: it is about *which* ethics is being reasoned with at all.
- **lattice** = federates *descriptive* domain knowledge (what is). doxa federates
  *normative* frameworks (what ought to be). They are duals and meet at the ABox: a
  scenario's facts can come from lattice; its evaluation comes from doxa.
- **concord** = de-escalates *human* disagreement rhetorically. doxa formalizes the
  *philosophical* disagreement deductively. Different layers of the same pluralism.

doxa reuses ousia wholesale: `ousia-forge` compiles each framework's TOML to OWL,
`ousia-reason` materializes its axioms, `ousia-guard` is the per-framework verdict
engine doxa aggregates.

## Components (PRD-sized)

- **doxa-moral-core** (rust-cli, new repo `~/wintermute/doxa`) — the shared,
  framework-neutral moral-domain TBox spec (BFO-grounded) + `doxa build-core` forging
  it via `ousia-forge`. The commensurability foundation.
- **doxa-frameworks** (rust-extend doxa) — the first three normative families as axiom
  modules over moral-core: **consequentialism** (Right ≡ maximizes aggregate Wellbeing),
  **deontology** (Right ≡ universalizable maxim ∧ treats agents as ends), **virtue
  ethics** (Right ≡ expresses a virtue the phronimos would). `doxa list` / `doxa build`.
- **doxa-reason** (rust-extend doxa) — `doxa reason <framework> --scenario <abox>`:
  per-framework verdict over a scenario ABox, reusing `ousia-reason`, with the axiom
  chain that justifies it.
- **doxa-compare** (rust-extend doxa) — `doxa compare <fw...> --scenario`: run N
  frameworks on one scenario; emit the agreement/conflict matrix (consensus vs dilemma).
- **doxa-guard** (rust-extend doxa) — pluralist `allow | flag | deny` aggregated across
  frameworks by an explicit policy (unanimity / majority / framework:<name> /
  lexical:<order>), reusing `ousia-guard` per framework.

## Order

```
doxa-moral-core
   └── doxa-frameworks      (axioms over the core vocab)
          └── doxa-reason   (evaluates one framework on a scenario)
                 ├── doxa-compare   (runs many, diffs verdicts)
                 └── doxa-guard     (runs many, aggregates by policy)
```

moral-core first; frameworks second; reason third; compare and guard are independent
of each other once reason ships.

## Open questions (for the next /dream pass or the user)

- **Which frameworks next?** Fleet 1 ships the three normative families. Fleet 2
  candidates (each one PRD-sized module once the shape proves out): Kantian categorical
  imperative (the strict reading), Rawlsian justice-as-fairness, Aristotelian eudaimonia
  (distinct from generic virtue ethics), care ethics, Stoicism, Confucian role-ethics,
  Buddhist (ahimsa / dependent-origination), Ubuntu. Left as bullets, not over-dreamt.
- **Scenario authoring.** A scenario ABox (an Action with its Intention, Consequences,
  Agents) needs an ergonomic input format. Start with hand-written Turtle/JSON; a
  `doxa scenario` builder is a deferred follow-on.
- **Commensurability is a real philosophical claim, not just code.** Frameworks
  genuinely disagree about what the morally relevant *facts* are (a consequentialist
  cares about outcomes a deontologist brackets). The shared TBox must be rich enough to
  carry every framework's relevant predicates without smuggling one framework's
  metaphysics into the "neutral" core. The honest stance: the core is descriptive
  (Action *has* Consequence, *has* Intention); each framework decides what *matters*.
  Where the core proves too thin, that is a finding, not a failure — surface it.
- **Meta-ethics is out of scope.** doxa formalizes *normative* frameworks (how to act).
  It does not adjudicate *meta-ethics* (whether moral facts exist). The aggregation
  policy is the user's explicit choice, never doxa's claim about which philosophy is true.
