# PRD: conning-tower-back-issues — the persistent, queryable archive of every column

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** (existing repo) `~/wintermute/conning-tower`
**Vision:** visions/conning-tower.md

## TL;DR

Columns syndicate into `columns/YYYY-MM-DD.md`, one file per day — but a directory
of dated Markdown is not an archive you can *ask questions of*. "Show me every
column Parker placed the bon mot in." "What ran on 2026-05-31?" "How many columns
this month?" Without an index those queries mean grepping a growing pile by hand —
the same read-by-hand bottleneck `conversations-zine`'s README warns about. This
PRD builds the back-issues archive: a persistent index over all syndicated
columns, queryable by date, persona, and span, optionally mirrored into recall.

## Why this exists

`~/wintermute/conversations-zine`'s README names the failure exactly: without a
tool, "the editor has to read every transcript by hand." A run of columns has the
same problem at the periodical scale — `new-yorker` (downstream) must assemble
issues from columns, and a reader wants back-issues, and neither can scan a flat
directory efficiently as it grows. The umbrella [roundtable.md](visions/roundtable.md)
explicitly nominates `~/wintermute/recall` — "episodic memory for the column's
back-issues" — as the store for this. `recall` is local-first agentic memory
(plain `.md` + SQLite FTS5, per its README) — exactly the substrate for a
queryable column archive, with a self-sufficient local index as the floor so the
archive never hard-depends on recall's schema.

## What this builds

Extends `~/wintermute/conning-tower` (lib `conning_tower`, CLI `conning-tower`).

- **`archive.rs`** — an append/update index over the columns dir. Each indexed
  entry: `{ date, headline, bon_mot_line, bon_mot_persona, contributor_count,
  personas: [...], path }`, built by parsing each `columns/*.md` (or reading the
  JSON form). Index persisted as `back-issues.jsonl` (or a small SQLite file,
  implementer's choice; JSONL is the documented floor). Re-indexing is idempotent
  and reflects added/changed columns.
- **`query.rs`** — query the index by date (exact), by `--persona` (columns where a
  persona placed any line / the bon mot), and by `--since/--until` span; plus a
  `count` summary.
- **CLI:**
  - `conning-tower archive [--columns-dir PATH] [--rebuild]` — (re)build the index
    from the columns dir.
  - `conning-tower back-issue <date> [--format md|json]` — print one archived
    column (the full Markdown, or its index entry as JSON).
  - `conning-tower back-issues [--persona NAME] [--since DATE] [--until DATE]
    [--bon-mot-only] [--format text|json]` — list matching index entries.
  - `--mirror-recall` (on `archive`) additively writes each column into `recall` via
    the `recall` binary if present; never fatal if absent.
- Deps: reuse existing; SQLite (`rusqlite`) only if the implementer picks the DB
  index — JSONL keeps the dep set unchanged.

## Acceptance criteria

1. `conning-tower archive --columns-dir <tmp>` over a tmp dir holding three
   syndicated columns writes a `back-issues.jsonl` (or DB) with three entries, each
   carrying `{date, headline, bon_mot_line, bon_mot_persona, contributor_count,
   personas, path}` matching the parsed columns.
2. Re-running `archive` after adding a fourth column yields four entries with no
   duplicates (idempotent by date); `--rebuild` discards and rebuilds the index
   from scratch with the same result.
3. `conning-tower back-issue 2026-06-08 --format json` prints that date's index
   entry; `--format md` prints the column's Markdown verbatim from disk; a date not
   in the archive exits non-zero (exit 3) naming the date.
4. `conning-tower back-issues --persona Parker` lists only columns where Parker
   placed a line; `--bon-mot-only` further restricts to columns where Parker held
   the bon mot; output is sorted date ascending and is byte-stable.
5. `conning-tower back-issues --since 2026-06-01 --until 2026-06-07` lists only
   columns whose date falls in `[since, until]` inclusive; an empty span yields an
   empty list and exit 0 (not an error).
6. The archive works with zero recall dependency (local JSONL/DB index is
   self-sufficient); `--mirror-recall` with the `recall` binary absent prints a
   warning and completes the local index with exit 0.
7. Each AC has a matching integration test under `tests/acceptance_ac<n>.rs` using
   committed column fixtures and temp dirs; no test writes outside its temp dir or
   reads the user's real `~/wintermute/conning-tower/columns/`.
