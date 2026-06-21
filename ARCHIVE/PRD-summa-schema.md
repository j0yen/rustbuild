# PRD-summa-schema

**Status:** Draft v0.1
**Vision:** visions/summa.md
**Design spec (authoritative):** visions/summa-design.md §1–§2 — the exact
`CLAUDE.md`, `index.md`, and `log.md` content to write is given verbatim there.
**build_target:** shell
**build_into:** /home/jsy/Notes

## TL;DR

`~/Notes/` is a 498-markdown / 81-PDF Obsidian vault with no schema, no navigation
files, and no version history. Karpathy's LLM-Wiki pattern needs three things in
place before any ingest or lint can run: a **schema** document defining the wiki's
structure and workflows, the two **navigation files** (`index.md` content catalog
+ `log.md` append-only timeline), and **git-backing** so the compounding artifact
has version history. This PRD lays that foundation, idempotently, on the real
vault — touching no existing note content.

## Why this exists

**Evidence (2026-06-21, live probe):**
- `ls ~/Notes/index.md ~/Notes/log.md` → both absent. Karpathy names these the two
  special files that "enable navigation" (index = content catalog, log = append-only
  timeline of ingests and queries). The vault has neither.
- `git -C ~/Notes rev-parse --is-inside-work-tree` → not a work tree. Karpathy:
  "treat the wiki as a git repository for version history and collaboration." There
  is no history; a bad clipper import is unrecoverable.
- The vault has `.obsidian/` (real Obsidian), 13 subject subdirs, and 149 files
  with `[[wikilinks]]` — there is genuine structure to *describe*, but nothing
  describes it. No `~/Notes/CLAUDE.md` exists (Karpathy's schema layer: "documents
  like CLAUDE.md that define wiki structure and workflows").

## What this builds

Runs on: **this laptop** (`carbon`), targeting `~/Notes/`.

1. **`~/Notes/CLAUDE.md`** — the schema. Defines:
   - The three layers: **raw sources** (`Clippings/`, `*.pdf`, immutable) → **wiki
     pages** (LLM-maintained summaries + entity pages) → **schema** (this file).
   - **Page types** and their frontmatter: `source-summary` (cites a raw source,
     carries `source:` + `ingested:`), `entity` (a concept/person/system page),
     `answer` (a filed query result, carries `asked:` + `question:`).
   - **Naming + link conventions**: entity pages are `Title Case.md`; links are
     `[[Title]]` (never the clipper's `[[Title\|numericID]]` form); a source-summary
     links its entities and back-links from them.
   - **Workflows**: how `summa ingest` and `summa lint` operate (referenced, built
     in later PRDs) so the schema is the single source of truth for the conventions.
2. **`~/Notes/index.md`** — skeleton content catalog: a top section per subject
   subdir + an "Entities" and "Answers" section, with a note that `summa index`
   regenerates it. Seeded, not hand-filled.
3. **`~/Notes/log.md`** — append-only timeline skeleton with a header and the first
   entry (this scaffolding commit), documenting the one-line format
   `YYYY-MM-DDTHH:MMZ  <ingest|answer|lint>  <subject>  <note>`.
4. **`git init`** + `~/Notes/.gitignore` excluding `.obsidian/workspace*`,
   `.obsidian/cache`, `.trash/`, `.DS_Store`, `*.tmp`. One initial commit (Joe Yen
   identity) capturing the current vault state as the baseline.
5. **Idempotent**: re-running creates nothing that exists; never overwrites
   `CLAUDE.md`/`index.md`/`log.md` if present (only seeds when absent); never
   re-inits an existing repo.

## Acceptance criteria

1. `~/Notes/CLAUDE.md` exists and documents the three layers, the three page types
   with their frontmatter, and the link/naming conventions.
2. `~/Notes/index.md` and `~/Notes/log.md` exist with the documented skeletons;
   `log.md` has at least the scaffolding entry.
3. `git -C ~/Notes rev-parse --is-inside-work-tree` → `true`; `git -C ~/Notes log
   --oneline` shows ≥1 commit; `.gitignore` excludes `.obsidian/workspace*` and
   `.trash/`.
4. No pre-existing note file under `~/Notes/` is modified or deleted by the script
   (verify with `git status` showing only the new scaffolding files as the diff
   beyond the baseline import, and a `wchg`/diff check that existing `*.md` content
   is untouched).
5. Re-running the script is a clean no-op: no new commit, no overwritten schema/nav
   files, exit 0.
