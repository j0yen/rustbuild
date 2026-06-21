# PRD-summa-skill

**Status:** Draft v0.1
**Vision:** visions/summa.md
**build_target:** mixed
**build_into:** /home/jsy/.claude/skills/summa

**Depends on:** PRD-summa-cli (mechanics), PRD-summa-schema (conventions)

## TL;DR

The deterministic CLI handles extraction, indexing, logging, and the link graph —
but the heart of Karpathy's pattern is LLM *judgment*: reading a source and writing
a faithful summary, minting the entity pages it implies, wiring cross-references,
and filing good answers back as permanent pages. That belongs in a skill, not a
binary. This PRD builds the `/summa` skill: an ingest-with-synthesis loop and an
ask-and-file loop, both using `summa` for mechanics and the brain ladder for
synthesis.

## Why this exists

**Evidence (2026-06-21):**
- summa-cli's `ingest` deliberately stops at extraction + a synthesis stub — it
  writes no summary or entity pages because that requires judgment. Something has to
  consume the stub and produce the wiki page. Karpathy: "Humans curate sources and
  ask questions; LLMs handle maintenance." The skill is the LLM-maintenance half.
- The vault has **81 un-synthesized PDFs** and a flat structure with **no entity
  pages** — turning "2026-05-06-mcp-server-codebase-analysis.pdf" into a summary
  page plus `[[Model Context Protocol]]`, `[[AtScale]]` entity pages with
  back-links is exactly the compounding Karpathy describes, and it needs an LLM.
- Karpathy: "Good answers become permanent pages … query results can be filed back
  into the wiki, ensuring explorations compound like ingested sources." Today asking
  a question of the vault leaves nothing behind. `/summa ask` closes that loop.
- The brain ladder (`wmd`, local-3b → cloud haiku/sonnet/opus, brain-local-first)
  is the right synthesis backend — consistent with how this box already routes LLM
  work, and keeps `/summa` usable headless.

## What this builds

A skill at `~/.claude/skills/summa/` (`SKILL.md` + helper scripts under
`scripts/`). Two flows:

1. **`/summa ingest <pdf|url|md>`**
   - Run `summa ingest <src>` for mechanics (extract + stage + stub JSON).
   - Read the extracted text; **synthesize** a `source-summary` page per the schema
     (`~/Notes/CLAUDE.md`): TL;DR, key claims, `source:` + `ingested:` frontmatter.
   - **Mint/strengthen entity pages**: for each significant entity, create the
     `Title.md` entity page if absent or append a sourced bullet if present; wire
     `[[cross-references]]` both ways (summary→entity and entity→summary).
   - `summa index` to refresh the catalog; `summa log ingest <subject> <title>`.
   - Synthesis routes through the brain ladder; the skill documents pinning a cloud
     tier when local-3b is too weak for dense sources (see vision open question).
2. **`/summa ask <question>`**
   - Retrieve candidate pages (grep + `summa links` neighborhood; optionally
     `recall query` per the vision's summa↔recall open question).
   - Answer from the vault, citing the pages used.
   - **File the answer** as an `answer`-type page (dated, `question:` frontmatter,
     `[[links]]` to cited pages), and `summa log answer <subject> <question>`.

The skill must be idempotent-friendly (re-ingesting the same source updates rather
than duplicates its summary page) and must never edit raw sources. Commits to the
vault are left to summa-commit's timer (or an explicit `summa-commit` call at the
end of a flow).

## Acceptance criteria

1. `SKILL.md` exists at `~/.claude/skills/summa/` documenting both flows, the schema
   it follows, and the brain-ladder synthesis backend with tier-pinning guidance.
2. `/summa ingest <fixture.pdf>` produces: a `source-summary` page with correct
   frontmatter, ≥1 entity page (created or appended) cross-linked both ways, a
   refreshed `index.md`, and a new `log.md` ingest line.
3. Re-running `/summa ingest` on the same source updates the existing summary page
   (no duplicate page created) — asserted by file count before/after.
4. `/summa ask "<question answerable from a fixture page>"` returns a cited answer
   AND writes a dated `answer` page linking the cited pages, plus a `log.md` answer
   line.
5. No raw source file (PDF/clipping) is modified by either flow.
6. A smoke run of both flows against a temp fixture vault is documented in the
   skill's `verification` field and passes via `sbx`.
