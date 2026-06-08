# Vision: herald — package & distribute a capability as an installable Claude skill

*A herald carries a message outward.* This vision is the distribution layer:
turn a wintermute capability into a one-command-installable Claude Code skill,
published through a marketplace anyone can `add`. The flagship payload is the
[ousia](ousia.md) BFO-grounded ethical reasoner — "ethical AI as a skill you
install."

## TL;DR

The seed (jsy, 2026-06-08): *"package and distribute ethical reasoning as a
skill."* Today on this laptop, skills are `SKILL.md` directories symlinked into
`~/.claude/skills/` from `~/wintermute/` (e.g. `build -> build-skill`). That is
fine for **the author's own machine** — but there is **no path for anyone else
to install them**. Phase-1 research (2026-06-08) confirmed: no wintermute repo
ships a `.claude-plugin/plugin.json`, and there is no `j0yen` `marketplace.json`.
The existing skill tooling (`skill-doctor`, `skill-manifest`, `wm-skill-edit`)
only *validates or edits* skills already installed locally; none **packages** a
capability for distribution.

`herald` closes that gap. It builds (1) a generic packager that turns a skill
dir + its Rust binary deps into an installable plugin, (2) a marketplace
manifest + publish flow under `j0yen`, and (3) the flagship `/conscience` skill
that wraps `ousia-guard`/`ousia-reason` so any Claude Code user gets deductive,
BFO-grounded ethical checking of proposed actions. The ousia vision answers
*"what is ethical reasoning?"*; herald answers *"how does anyone else get it?"*

## End-state

When fulfilled:

1. A maintainer runs `herald-pack` on a skill dir + a named wintermute binary
   and gets a self-contained installable plugin (`.claude-plugin/plugin.json`,
   the `SKILL.md`, an `install.sh` that builds/fetches the binary).
2. `herald-market` maintains a `j0yen` `marketplace.json` so a user runs
   `claude plugin marketplace add j0yen/wintermute-skills` and sees the catalog
   — the same `git-subdir` source shape the official 222-plugin marketplace uses.
3. `/conscience` is an installable skill: a user invokes it on a proposed
   action and gets `allow | flag | deny` with the axiom chain that justifies it,
   backed by the World Ontology — no Rust knowledge required.
4. The distribution channel is itself trustworthy: every published plugin passes
   `skill-doctor`/`skill-manifest` validation and carries its source binary's
   provenance, so an "ethics skill" isn't shipped broken or unverifiable.

## Components (PRD-sized)

1. **herald-pack** (`PRD-herald-pack`) — generic packager: skill dir + Rust
   binary dep → installable plugin (`.claude-plugin/plugin.json` + `SKILL.md` +
   `install.sh`). Reusable for any wintermute capability. *Foundational.*
2. **herald-market** (`PRD-herald-market`) — generate & maintain the `j0yen`
   `marketplace.json` aggregating packaged plugins (git-subdir sources, pinned
   ref+sha), with an `add`/`publish` flow mirroring the official marketplace.
   Depends on herald-pack's output format.
3. **herald-conscience** (`PRD-herald-conscience`) — the flagship: the
   `/conscience` Claude Code skill wrapping `ousia-guard` + `ousia-reason`, then
   packaged by herald-pack and listed by herald-market. Depends on
   herald-pack **and** ousia-guard (from the ousia vision).

## Order

```
herald-pack ──┬──► herald-market
              └──► herald-conscience   (also needs ousia-guard)
```

herald-pack is the gate. market and conscience both consume its output and can
proceed in parallel once it lands; conscience additionally waits on the ousia
fleet's guard/reason libs.

## Open questions

- **Binary delivery in `install.sh`.** Build-from-source (cargo, slow, needs
  rustc) vs fetch a prebuilt release artifact (fast, needs a release pipeline +
  per-arch builds). Starting with build-from-source (matches the existing
  `skill-manifest` `curl … | bash` install one-liner pattern); a release-fetch
  path is a later herald PRD once `rollout`/`cloudbuild` can emit signed
  artifacts.
- **Attestation / signing.** An *ethics* distribution channel arguably should
  be the most trustworthy one — a `herald-attest` PRD could compose
  `skill-doctor` + `provfs` provenance + a signature so consumers can verify a
  plugin wasn't tampered with. Deferred: it depends on a signing-key decision
  that is jsy's call. Captured here, not drafted.
- **Where does the marketplace repo live?** A dedicated public
  `j0yen/wintermute-skills` repo vs a subdir of an existing repo. Leaning new
  public repo (marketplaces are public-by-design and the official one is its
  own repo). Confirm with jsy before herald-market publishes.
- **Skill name.** `/conscience` vs `/ethics` vs `/ousia`. Using `/conscience`
  as the working name (evocative, unclaimed); easily renamed before publish.
