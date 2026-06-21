# summa — design deep-dive (build-ready spec)

**Companion to:** visions/summa.md
**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-21
**Purpose:** Lock every decision a builder would otherwise have to make mid-build,
so summa's five PRDs build fast and correctly. This is the authoritative spec; the
PRDs cite it. Where this doc and a PRD's prose differ, **this doc wins** (it is
newer and more thought-through).

---

## 0. The one decision everything hangs on: ownership

summa shares `~/Notes/` with **498 human-authored notes**. The single rule that
makes the whole system safe:

> **summa owns a file iff that file has a `summa:` frontmatter key. summa
> creates, edits, regenerates, and lint-fixes ONLY files it owns. Human notes are
> read-only to summa — it may *link to* them and *catalog* them, never rewrite
> them.**

Corollaries (all locked):
- **New summa pages live under `~/Notes/wiki/`** (a fresh subtree — confirmed
  absent 2026-06-21, clear to claim). Layout:
  - `wiki/entities/<Title>.md` — entity pages
  - `wiki/sources/<slug>.md` — source-summary pages
  - `wiki/answers/<YYYY-MM-DD>-<slug>.md` — filed answers
  Obsidian resolves `[[X]]` by filename anywhere in the vault, so pages under
  `wiki/` link bidirectionally with human notes at top level — the subtree is an
  organizational choice, not a linking barrier.
- **AC "touches no existing note content" becomes trivially true**: summa writes
  only under `wiki/`, plus the three managed files `index.md`/`log.md`/`CLAUDE.md`
  (which it creates and thereafter owns), plus — only behind explicit
  `lint --fix --include-human` — lossless link-rot normalization in human notes.
- The `\|` rot in 18 human-clipper files is therefore **report-only by default**;
  cleaning it is an opt-in, lossless (`\|`→`|`, strip all-digit aliases) action the
  user deliberately triggers.

---

## 1. The schema file — `~/Notes/CLAUDE.md` (write verbatim)

summa-schema writes exactly this (only if absent). It is both the human-readable
schema and the contract the CLI + skill obey.

```markdown
# summa — this vault's wiki schema

This Obsidian vault is an LLM-maintained wiki (Karpathy's "LLM Wiki" pattern).
Three layers:

1. **Raw sources** (immutable): `Clippings/`, `*.pdf`, and the human's own notes
   in the top-level subject folders. summa never rewrites these.
2. **Wiki pages** (summa-maintained, under `wiki/`): summaries of sources, entity
   pages, and filed answers. Every summa-owned file carries a `summa:` frontmatter
   key. summa edits ONLY these.
3. **Schema** (this file): the conventions below.

## Page types (frontmatter `summa:` value)

### source-summary  (`wiki/sources/<slug>.md`)
```
---
summa: source-summary
source: Clippings/2026-05-06-mcp-server-codebase-analysis.pdf
ingested: 2026-06-21T01:00:00Z
title: MCP Server Codebase Analysis
entities: ["[[Model Context Protocol]]", "[[AtScale]]"]
---
```
Body: TL;DR, key claims (each a bullet), open questions. Links its entities.

### entity  (`wiki/entities/<Title>.md`)
```
---
summa: entity
title: Model Context Protocol
aliases: [MCP]
created: 2026-06-21T01:00:00Z
updated: 2026-06-21T01:00:00Z
---
```
Body: a definition paragraph, then `## Mentions` — one sourced bullet per
source-summary that references this entity (back-link + one-line claim).

### answer  (`wiki/answers/<YYYY-MM-DD>-<slug>.md`)
```
---
summa: answer
question: "How does AtScale's MCP server expose models?"
asked: 2026-06-21T01:00:00Z
cites: ["[[MCP Server Codebase Analysis]]", "[[AtScale]]"]
---
```
Body: the answer, citing the pages it drew from.

## Link conventions
- Always `[[Title]]` or `[[Title|Alias]]` — NEVER `[[Title\|123456]]` (clipper rot).
- Entity titles are Title Case. A link resolves to any `.md` with that filename.

## Navigation
- `index.md` — content catalog (summa-managed between anchors). Regenerate with
  `summa index`.
- `log.md` — append-only timeline of ingests, answers, lint runs. `summa log …`.

## Workflows
- Ingest a source: `/summa ingest <pdf|url|md>` (extract → summarize → entities →
  links → index → log).
- Ask & file: `/summa ask "<question>"` (answer from vault → file as answer page).
- Maintain: `summa lint [--fix]` (orphans, dangling, malformed, stale, un-ingested).
```

---

## 2. Managed file formats

### `index.md`
```markdown
# Index

<!-- summa:index-start -->
## Entities
- [[Model Context Protocol]]
- [[AtScale]]
## Sources
- [[MCP Server Codebase Analysis]] — 2026-05-06
## Answers
- [[2026-06-21-mcp-model-exposure]]
## Human notes (by folder)
- **AtScale/** — 41 notes
- **Clippings/** — 63 notes
<!-- summa:index-end -->
```
**Rule:** summa rewrites only between the anchors. Human pages are listed at
**folder granularity** (one line per top-level subject dir with a count), summa
pages **individually**. This keeps index.md bounded over a 498-file vault.

### `log.md`
Append-only. One line per event:
```
2026-06-21T01:00:00Z  ingest  AtScale  MCP Server Codebase Analysis
2026-06-21T01:05:00Z  answer  AtScale  How does AtScale's MCP server expose models?
2026-06-21T03:00:00Z  lint    -        fixed 18 malformed links, 0 dangling
```
Grammar: `<ISO-8601-Z>  <ingest|answer|lint>  <subject|->  <note>` (two spaces as
field sep; note is free text to end-of-line).

---

## 3. summa-cli — data model, modules, crates, JSON (locked)

Binary `summa`, Rust, **rustc 1.85, MSRV-safe, no let-chains** (matches lib-crate
toolchain on this box). `sigpipe::reset()` first line of `main()` (local-CLI
SIGPIPE convention). Vault root: `$SUMMA_VAULT` else `~/Notes`.

### Modules
| file | responsibility |
|------|----------------|
| `main.rs` | clap dispatch, sigpipe reset |
| `vault.rs` | root resolution; enumerate files (walkdir, skip `.obsidian`,`.git`,`.trash`); classify md vs source |
| `frontmatter.rs` | parse YAML frontmatter (serde_yaml); `is_summa_owned()`, page-type enum |
| `links.rs` | wikilink parser + graph (orphans, dangling, malformed) |
| `ingest.rs` | classify + extract (pdf/url/md) + stage + stub |
| `index.rs` | regenerate index.md between anchors |
| `log.rs` | append a line |
| `page.rs` | mint/append entity, source-summary, answer pages (the skill's tested write path) |

### Crates (locked — minimal)
`clap` (derive), `serde`/`serde_json`/`serde_yaml`, `walkdir`, `regex` (link
parsing — simpler & sufficient vs a full md parser), `chrono` (UTC ISO-8601),
`anyhow`, `pdf-extract` (pure-Rust PDF text — **no pdftotext on this box**,
confirmed), `ureq` + `html2text` (URL fetch → text, best-effort, offline-fails
cleanly). `sigpipe`.

### Subcommands & JSON contracts
- `summa ingest <path|url> [--dest Clippings]` → stages raw, extracts text to a
  temp file, prints:
  ```json
  {"source":"<orig>","kind":"pdf|url|md","staged_path":"Clippings/x.pdf",
   "extracted_text_path":"/tmp/summa-<sha8>.txt","suggested_title":"X","bytes":1234}
  ```
- `summa index` → rewrites `index.md` between anchors; idempotent (second run = no
  diff). Exit 0.
- `summa log <kind> <subject> <note...>` → appends one line; never rewrites.
- `summa links [--json]` →
  ```json
  {"orphans":["wiki/entities/Foo.md"],
   "dangling":[{"target":"Bar","in":["wiki/sources/x.md"]}],
   "malformed":[{"file":"AtScale/y.md","raw":"[[A\\|123]]","fix":"[[A]]"}],
   "stats":{"pages":520,"links":1840}}
  ```
- `summa page entity <Title> [--alias A] [--mention "<srclink> — <claim>"]` →
  create `wiki/entities/<Title>.md` if absent (with frontmatter), else append the
  mention bullet under `## Mentions` (dedup on the source link). Bumps `updated:`.
- `summa page summary --source <path> --title <T> --tldr <file> --entity "[[E]]"…`
  → write/overwrite `wiki/sources/<slug>.md` (idempotent on slug → update, no dup).
- `summa page answer --question <q> --slug <s> --body <file> --cite "[[P]]"…` →
  write `wiki/answers/<date>-<slug>.md`.

> **`summa page` is in scope for summa-cli** (it is the tested write path the
> skill calls — without it the skill would hand-roll file writes). summa-cli's AC
> set should include `page` round-trip tests. summa-skill depends on it.

### Wikilink grammar (the subtle part — get this right)
- Token regex (links + embeds): `!?\[\[([^\]]+)\]\]`.
- Inside the capture, split on the **first** `|` OR `\|` → left = target+section,
  right = alias. `\|` present ⇒ **malformed**. Alias that is all digits ⇒
  **malformed** (clipper ID).
- Target = capture before `#` and before the alias separator, trimmed.
- **Resolution (Obsidian semantics):** `[[X]]` resolves if any `.md` file in the
  vault has stem `X` (case-sensitive first; fall back to case-insensitive).
  No match ⇒ **dangling**.
- Embeds `![[...]]` count as links for the graph.
- Inbound count for orphan detection includes links from human notes.

---

## 4. summa-lint — exact predicates & `--fix` scope (locked)

`summa lint [--fix] [--include-human] [--json]`. Checks:

| check | predicate | `--fix`? |
|-------|-----------|----------|
| orphan | summa-owned page (entity/source-summary/answer) with 0 inbound links; exclude index/log/CLAUDE | no (needs judgment) |
| dangling | link target with no resolvable `.md` | no |
| malformed | token has `\|` or all-digit alias | **yes** — `\|`→`|`, strip digit alias. Summa-owned files by default; human files only with `--include-human` |
| missing-index | summa-owned page absent from index anchors | **yes** — run `summa index` |
| stale-vs-source | source-summary `ingested` < mtime(`source`) | no (re-summary is the skill's job) |
| un-ingested | a raw source (pdf, or `Clippings/*.md` lacking `summa:`) with no source-summary whose `source:` points at it | no |

`--fix` does ONLY malformed + missing-index, prints what changed, idempotent
(second `--fix` = no diff). Never edits a source-summary body, a raw source, or a
stale/orphan page's content. `lint` appends a `log.md` lint line summarizing counts.

---

## 5. summa-skill — synthesis is Claude itself (corrected backend)

**Locked correction:** `wmd` is a long-running daemon (Claude API loop over the
bus), not a one-shot `ask`. So synthesis is performed by **Claude executing the
`/summa` skill** — the agent *is* the wiki maintainer (faithful to Karpathy). A
fully-headless cron path can use `claude -p` or an Agent; that is secondary and
optional, not v1's primary path. No `wmd` dependency.

### `/summa ingest <pdf|url|md>` — state machine
1. `summa ingest <src>` → stub JSON; read `extracted_text_path`.
2. **Synthesize summary** (Claude): produce TL;DR + key-claim bullets + open
   questions from the extracted text. Prompt skeleton:
   > "You are maintaining a personal wiki. Summarize the following source into:
   > a one-paragraph TL;DR, 3–8 key-claim bullets (each standalone, sourced), and
   > any open questions. Then list the significant **entities** (concepts, systems,
   > people, papers) it discusses, as a JSON array of Title-Case names. Source
   > text: <…>"
3. `summa page summary --source <staged> --title <T> --tldr <tldr-file> --entity …`
   (idempotent on slug → re-ingest updates, never duplicates — satisfies the
   "no duplicate page" AC).
4. For each entity: `summa page entity <Title> --mention "[[<summary title>]] — <one-line claim>"` (creates or appends+dedups).
5. `summa index`; `summa log ingest <subject> <title>`.

### `/summa ask "<question>"` — state machine
1. **Retrieve**: `grep -ri` question keywords across vault + `summa links`
   neighborhood of hits + (optional, per open question) `recall query`.
2. **Answer** (Claude) citing the pages used.
3. `summa page answer --question <q> --slug <s> --body <file> --cite "[[P]]"…`;
   `summa log answer <subject> <question>`.

Subject classification = the top-level folder of the dominant cited page (else `-`).

### Tier guidance
When the secondary headless path is used, default to **cloud-sonnet** for ingest
(dense PDFs); `SUMMA_TIER` env overrides. Interactive `/summa` just uses the
running Claude.

---

## 6. Test fixtures (so ACs are checkable)

`tests/fixtures/vault/` (used by summa-cli + summa-lint):
```
CLAUDE.md, index.md (with anchors, one page deliberately missing), log.md
wiki/entities/Model Context Protocol.md   (entity, has inbound link)
wiki/entities/Orphan Concept.md           (entity, NO inbound link → orphan)
wiki/sources/mcp-analysis.md              (source-summary, ingested AFTER source mtime → fresh)
wiki/sources/stale-analysis.md            (source-summary, ingested BEFORE source mtime → stale)
wiki/answers/2026-06-21-example.md        (answer)
Clippings/mcp-analysis.pdf                (the fresh summary's source)
Clippings/stale-analysis.pdf              (the stale summary's source)
Clippings/never-ingested.pdf              (un-ingested source)
AtScale/human-note.md                     (human note with a [[Title\|12345]] malformed link + a [[Nonexistent]] dangling link)
```
Each lint category has a positive and a negative case here.

---

## 7. Build order, effort, fast-path

| # | PRD | target | effort | fast-path |
|---|-----|--------|--------|-----------|
| 1 | summa-schema | shell, `~/Notes` | S | copy §1 CLAUDE.md verbatim; seed §2 index/log; `git init`+`.gitignore`; idempotent guards |
| 2 | summa-cli | rust-cli, new `j0yen/summa` | L | modules §3; crates §3; JSON §3; grammar §3; `page` included; fixtures §6 |
| 3 | summa-lint | rust-extend `~/wintermute/summa` | M | predicates §4; reuse `summa links`; `--fix` = malformed+index only |
| 4 | summa-skill | mixed, `~/.claude/skills/summa` | M | state machines §5; Claude-as-synthesizer; calls `summa page/index/log` |
| 5 | summa-commit | shell → constellation | S | node-local timer; git commit w/ log-derived msg; remote-optional |

Critical path: 1 → 2 → {3,4}. summa-commit (5) can land any time after 1.
summa-cli (2) is the only heavy build; everything else is small/mechanical.

---

## 8. Open questions — now answered (so they don't block the build)

- **summa vs recall:** keep **separate stores**, cross-link only. `/summa ask` MAY
  consult `recall query` as one retrieval source (optional in §5), but summa never
  writes into recall and vice-versa. No merge in v1.
- **Synthesis tier:** interactive = the running Claude; headless = cloud-sonnet
  default, `SUMMA_TIER` override. Locked; no per-build decision.
- **Contradiction detection:** out of v1 (lint does stale-vs-source only). Future
  PRD `summa-contradict` if motivated.
- **Image localization/captioning:** out of v1 (4 PNGs only). Future PRD.
- **Fleet-shared vault:** out of v1 — vault is single-node canonical; summa-commit
  is node-local. Revisit only if the vault is ever read on another node.
- **PDF extraction:** `pdf-extract` crate (no system pdftotext). Locked.
- **Empty/huge index:** human notes catalogued at folder granularity (§2). Locked.
