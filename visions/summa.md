# Vision: summa — the wiki she'll never maintain by hand

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-21
**Status:** active
**Seed:** jsy — `/dream https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f`
  (Karpathy's "LLM Wiki" pattern).

*Summa* — as in Aquinas's *Summa Theologica*, a single compounding compendium that
synthesizes a whole field; and as in *summary*, the act of distillation. The
vision is Karpathy's LLM-Wiki instantiated on this laptop's real Obsidian vault.

## TL;DR

Karpathy's gist proposes an alternative to query-time RAG: an **LLM-maintained
wiki** — "a structured, interlinked collection of markdown files that sits
between you and the raw sources." The wiki is *a persistent, compounding
artifact*: synthesis accumulates instead of being re-derived per query.
Cross-references, contradictions, and entity pages are pre-computed and kept
fresh. **Humans curate sources and ask questions; the LLM does the maintenance**
that causes hand-built wikis to decay — "the cost of maintenance is near zero."

This laptop already has the raw material and *none* of the maintenance:
`~/Notes/` is a real Obsidian vault — **498 markdown files, 81 PDFs, 13 subject
subdirs** (`AtScale/`, `Clippings/`, `server-side-dax/`, `Journals/`, `Classes/`,
`flashcards/`, …), `.obsidian/` present, **149 files already use `[[wikilinks]]`**.
But it is a *flat date-prefixed pile*, not a maintained wiki:

- **No `index.md`, no `log.md`** — Karpathy's two navigation files (content
  catalog + append-only timeline) are both absent.
- **Not a git repo** (`git -C ~/Notes rev-parse` → not inside work tree) — no
  version history, despite Karpathy's "treat the wiki as a git repository."
- **Link rot already setting in**: 18 files carry broken `\|` escaping from the
  Obsidian web-clipper (`[[PRD: PR #80\|4712398850]]`), wikilinks point at
  numeric clipper IDs, and **81 PDFs sit with no synthesized companion page**.
- **No ingest path**: saving a source is manual; `recall` (the agentic memory
  store) is a *separate* system and does not touch the vault. There is no tool
  that turns a dropped PDF/URL into summary + entity pages + cross-references.

summa is the maintenance layer the vault never had: a small deterministic CLI for
the mechanics (extract, index, log, link-graph, lint) plus a `/summa` skill for
the LLM-judgment parts (summarize a source, mint/strengthen entity pages, file a
good answer back as a permanent page). The human keeps curating sources and
asking questions; summa keeps the compounding artifact coherent.

## End-state

When summa is done:

- `~/Notes/` is a **git repo** with an LLM-maintained `index.md` (content
  catalog) and append-only `log.md` (every ingest + filed answer, timestamped).
- `~/Notes/CLAUDE.md` is the **schema**: the wiki's layer model (raw sources →
  wiki pages → schema), page types (source-summary, entity, answer), naming and
  `[[link]]` conventions, and the ingest/lint workflows — flexible, domain-specific,
  iterated with the user (Karpathy's third layer).
- **Dropping a source is one command**: `summa ingest <pdf|url|md>` stages the raw
  source and `/summa` synthesizes a summary page, creates or strengthens the
  entity pages it mentions, wires cross-references, and appends a `log.md` line.
- **A `summa lint` pass** keeps the artifact honest: orphan pages (no inbound
  links), dangling `[[wikilinks]]`, the `\|` escaping rot, missing index entries,
  and source-summaries staler than their source — with `--fix` for the mechanical
  ones.
- **Good answers compound like sources**: `/summa ask` answers from the vault and
  files the answer back as a dated page with cross-references, so explorations
  accrete instead of evaporating.
- Maintenance cost is near zero: a periodic timer commits the vault; lint runs on
  a cadence; the user only curates and asks.

## Components (PRD-sized)

1. **summa-schema** (shell) — FOUNDATIONAL. Write `~/Notes/CLAUDE.md` (the schema
   layer: page types, naming, link + frontmatter conventions, ingest/lint
   workflows). Seed `index.md` and `log.md` skeletons. `git init` the vault with a
   sane `.gitignore` (`.obsidian/workspace*`, `.trash/`, OS cruft). One initial
   commit. Everything downstream reads these conventions.

2. **summa-cli** (rust-cli, new repo `j0yen/summa`) — the deterministic mechanics
   binary `summa`: `ingest <src>` (classify + extract text: PDF→text, URL→reader
   markdown, md→passthrough; stage the raw into `Clippings/` and emit a synthesis
   stub), `index` (regenerate `index.md` from the vault tree + entity pages),
   `log <line>` (append a timestamped entry to `log.md`), `links` (build the
   wikilink graph; report orphans + dangling targets as JSON). No LLM calls — pure,
   testable mechanics.

3. **summa-lint** (rust-extend → `~/wintermute/summa`) — `summa lint [--fix]`:
   orphan pages, dangling/malformed wikilinks (auto-repair the `\|`→`|` escaping
   and clipper-ID links under `--fix`), pages missing from `index.md`, and
   source-summary pages older than the source they cite (`stale` flag). Emits a
   report; `--fix` performs only the mechanical repairs. Karpathy's "lint pass."

4. **summa-skill** (mixed — `/summa` skill) — the LLM-judgment loop on top of the
   CLI. `/summa ingest <src>`: run `summa ingest` for mechanics, then synthesize
   the source-summary page, mint/strengthen the entity pages it names, wire
   `[[cross-references]]`, and `summa log` the result. `/summa ask <q>`: answer
   from the vault and file the answer back as a dated permanent page. Synthesis
   routes through the brain ladder (local-3b → cloud) consistent with
   brain-local-first.

5. **summa-commit** (shell) — near-zero-maintenance backing: a `summa-commit.timer`
   that commits the vault on a cadence (and after each ingest), so version history
   accrues without manual effort. **Node-local** placement (the vault is
   laptop/`carbon`-held canonical state, like recall — see [[carbon]] open
   questions on canonical-memory placement); classify in `placement.toml`.

## Order

```
summa-schema (1) ──┬──> summa-cli (2) ──┬──> summa-lint (3)
                   │                    └──> summa-skill (4)
                   └──> summa-commit (5)
```

- schema(1) ships first and alone — pure file scaffolding + `git init`.
- cli(2) depends on schema(1) for the conventions it encodes.
- lint(3) extends cli(2); skill(4) depends on cli(2) + schema(1).
- commit(5) depends only on schema(1)'s `git init`.

## Open questions (left here, not drafted)

- **summa vs recall.** `recall` is the *agentic* memory (what Claude learned across
  sessions); summa is the *human's* curated knowledge (sources she chose). They
  should cross-reference, not merge — but a future PRD could let `summa ask`
  consult recall and vice-versa. Relates to [[corpus]] (one self across stores).
- **Synthesis model tier.** Local-3b may be too weak for entity extraction over a
  dense PDF; the skill may need to pin cloud-sonnet for ingest. Calibrate after
  summa-skill ships; do not pre-decide.
- **Contradiction handling.** Karpathy flags "contradictions" as a first-class wiki
  signal. summa-lint detects stale-vs-source; true cross-page contradiction
  detection (page A claims X, page B claims ¬X) is a harder future PRD.
- **Image localization.** Karpathy recommends downloading images locally so the LLM
  can analyze them. The vault has 4 PNGs today; a future `summa ingest` extension
  could localize + caption images. Not motivated enough yet to draft.
- **Hub vs laptop placement.** If the vault ever becomes fleet-shared (read on
  ryzen7/hub), summa-commit's node-local assumption needs revisiting. For now the
  vault is single-node canonical.
