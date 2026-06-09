# PRD: herald-conscience — the /conscience ethics skill, packaged for anyone

Status: Draft v0.1
build_target: mixed
build_priority: high
build_into: /home/jsy/wintermute/herald-conscience
Vision: visions/herald.md

## TL;DR

This is the flagship payload the whole herald + ousia effort exists to deliver:
a `/conscience` Claude Code skill that lets any user (no Rust required) check a
proposed action against the BFO-grounded World Ontology and get
`allow | flag | deny` with the deductive axiom chain that justifies it. It wraps
`ousia-guard` + `ousia-reason`, ships as a `SKILL.md` + scripts, and is packaged
by `herald-pack` and listed by `herald-market` — "ethical AI as a skill you
install."

## Why this exists

- **It is the literal seed.** jsy, 2026-06-08: *"package and distribute ethical
  reasoning as a skill."* ousia builds the reasoning; herald-pack/market build
  the distribution; this PRD is the user-facing skill that joins them.
- **The reasoning engine is being built.** `PRD-ousia-guard` (verified present
  on disk, 2026-06-08) specifies the `allow/flag/deny` + justification verdict
  over the World Ontology. conscience is the thin, human-facing skill layer over
  guard's CLI/lib — not new reasoning, a new *surface*.
- **A skill is the right surface for humans-in-Claude.** ousia-mcp targets
  agents; ousia-atscale targets the data market; the gap is the Claude Code user
  who wants to ask "is this action ethically sound?" in-session. The packaging
  pain that motivates this being a *real installable* skill (not a local
  symlink) is the recurring SKILL.md-drift / install-validation pain recall
  surfaces (skill-manifest's own README: "three SKILL.md bugs shipped to
  production… because no install-time validation exists").

## What this builds

A skill repo `herald-conscience` at `~/wintermute/herald-conscience/`:
`SKILL.md`, `scripts/`, and a `herald.toml` packaging spec consumed by
herald-pack.

**Shape.**
- `SKILL.md` — defines `/conscience`. Invocation: `/conscience <action
  description>` (free text or a path to an action JSON). The skill translates
  the description into the small RDF/JSON action document `ousia-guard` expects,
  shells `ousia-guard check --format json`, and renders the verdict + the
  justifying axiom chain in readable markdown.
- `scripts/check.sh` — the bridge: normalize input → call `ousia-guard` → format
  output. Degrades gracefully if the ontology/binary is absent (actionable
  install message, not a stack trace).
- `herald.toml` — packaging spec naming `ousia-guard` (+ the forged
  `world-ontology.owl`) as the bundled dependency, so `herald-pack` can emit an
  installable plugin and `herald-market` can list it.

**Deps:** `ousia-guard` (CLI/lib, from the ousia vision) and a forged
`world-ontology.owl` (from `ousia-forge`). The skill itself is mixed
(SKILL.md + shell), not a new Rust crate.

## Acceptance criteria

1. `SKILL.md` defines `/conscience` with a clear invocation, a "what it does"
   section, and worked examples for each verdict (allow, flag, deny) tied to the
   ousia rule battery (dignity-floor, rights-violation, unaccountable-authority,
   welfare-undermine).
2. `scripts/check.sh <action>` shells `ousia-guard check --format json` and
   renders the verdict (`allow|flag|deny`) plus the per-rule axiom-chain
   justification as readable markdown.
3. When `ousia-guard` or `world-ontology.owl` is missing, the skill prints an
   actionable install message (point at the herald-pack-built plugin), exits
   non-zero, and does **not** emit a misleading "allow".
4. A `herald.toml` is present and valid: `herald-pack check --spec herald.toml`
   passes (binary `ousia-guard` declared; SKILL.md invocations resolve).
5. `herald-pack build --spec herald.toml` produces an installable plugin tree
   for `/conscience` (this PRD's output is the first real input to herald-pack —
   they are validated together).
6. End-to-end fixture test: given a dignity-violating action description, the
   skill (via check.sh → ousia-guard) returns `deny` with the sentience→dignity
   chain; given a benign action, returns `allow`. (Test may stub `ousia-guard`
   with a recorded fixture if the real binary is unavailable in CI, but the wire
   contract — args + JSON shape — must match the real `ousia-guard`.)
7. `SKILL.md` passes `skill-doctor` (no undocumented invocations, no stale flag
   references) — the distribution channel ships a validated skill.
8. The skill makes **no** settings.json edits itself; any hook wiring is shown
   as a documented opt-in example (mirrors PRD-ousia-guard AC-9).
