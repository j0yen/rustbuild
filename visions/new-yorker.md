# Vision: new-yorker — the durable publication, bound from the table

> Sub-vision of [roundtable](roundtable.md). Harold Ross founded *The New
> Yorker* from the Algonquin in February 1925 — "not edited for the old lady in
> Dubuque." Where the rest of `roundtable` is the lunch (ephemeral, daily, a
> conversation that evaporates), `new-yorker` is the thing that *outlives* any
> single lunch: a small periodical with a masthead, a house style, numbered and
> dated issues, and a cover. It is the binding layer.

## TL;DR

`roundtable` produces a *daily* artifact — `conning-tower`'s column of crowned
bon mots, attributed to persona contributors (see `visions/conning-tower.md`,
PRD `conning-tower-column`). A daily column is the right cadence for a
conversation but the wrong cadence for a *publication*: a pile of 30 dated
Markdown columns is not an issue of anything. Nobody binds them. The wit that
survived the table still ends up as a write-only heap — the exact failure mode
the umbrella `roundtable.md` set out to fix.

`new-yorker` binds. It defines a stable publication identity (a masthead with
the persona contributors), enforces a single house voice across columns written
by five different personas (Ross's precise, urbane standard), assembles N daily
columns plus the period's best games into one numbered, dated issue, and gives
each issue a generative cover drawn from its own vocabulary. The output is a
durable artifact — an *issue* — that survives the laptop's daily churn the way
the zine survives a quarter.

## End-state

On a periodical cadence (weekly or monthly; the timer is a downstream wrapper,
out of scope here), `new-yorker` reads the back-issues of the `conning-tower`
column archive, runs each column's prose through the house-style enforcer to
flatten five persona voices into one editorial voice, assembles the cleaned
columns plus the issue's crowned bon mots and best games into a single bound
issue with a masthead and a folio (issue number + date), and renders a
generative cover motif seeded from the issue's own most-frequent words. The
issue lands as a self-contained Markdown document (plus an optional rendered
form on the `daily-receipt` deterministic-render precedent) under a
`~/wintermute/new-yorker/issues/NNNN/` tree — numbered, dated, and reproducible.

## Components (one per PRD)

- **newyorker-masthead** (`rust-lib` + thin CLI) — the publication identity.
  Title, the persona roster as a masthead (editor, contributors, their column
  bylines), and deterministic issue numbering/dating (folio: monotonic issue
  number + ISO date + volume). The stable header every other component stamps
  onto its output. Ships first; everyone depends on it.
- **newyorker-house-style** (`rust-cli`) — the house-style enforcer. A
  lint/transform pass over column text that enforces an urbane, precise
  editorial voice: flags and optionally cleans clichés, hedges ("I think",
  "sort of", "arguably"), filler, and sloppiness, on Ross's standard. `lint`
  reports; `clean` rewrites deterministically. The voice-flattener.
- **newyorker-issue** (`rust-cli`) — the issue-assembler. Binds N daily columns
  (from the `conning-tower` archive) + the period's crowned bon mots + best
  games into one periodical issue: stamps the masthead, runs each column through
  house-style, lays out sections, writes numbered/dated Markdown, and emits an
  optional render on the `daily-receipt` path. The binder.
- **newyorker-cover** (`rust-cli`) — the generative cover. A deterministic
  SVG/ASCII cover motif seeded from the issue's own vocabulary (word-frequency →
  a generative geometric/typographic motif), stamped with the folio. Each
  issue's identity, drawn from its own contents.

## Order

```
newyorker-masthead (identity) ──┬──> newyorker-issue (bind) <── newyorker-house-style (voice)
                                └──> newyorker-cover (cover) ──┘
```

- `newyorker-masthead` ships first — the folio/identity lib everyone stamps.
- `newyorker-house-style` is independent of masthead; it can ship in parallel.
- `newyorker-issue` consumes masthead + house-style + the `conning-tower`
  archive; it is the keystone.
- `newyorker-cover` consumes masthead (folio) + the assembled issue's
  vocabulary; it can ship after issue or alongside it.

This whole sub-vision ships **last** in `roundtable` — it binds what everything
upstream produced.

## Why this is honest, not fan-fiction

- `~/wintermute/conversations-zine/` (binary `zine`) is the existing
  **periodical-extractor** — wintermute's closest prior art for "bind accumulated
  material into a small printed publication." Its README names the quarterly
  zine and ships the moment-extractor; `new-yorker` is the *issue-assembler* that
  zine's pipeline left explicitly downstream and human-driven.
- `~/wintermute/daily-receipt/` (binary `daily-receipt`) is the existing
  **deterministic render path**: `render --summary --content --out` emits a
  byte-stable artifact (AC3: byte-identical inputs → byte-identical output) and
  embeds the ISO date verbatim (AC7). `newyorker-issue`'s optional rendered form
  follows this determinism contract.
- `~/wintermute/self-portrait/` + the `cadence` vision establish the
  house-voice / diff-narration work and the noon-render ritual the periodical
  cadence extends.
- `~/wintermute/recall/` is the episodic store the column back-issues live in,
  the same way `conning-tower` archives them.
- The umbrella `roundtable.md` frames `new-yorker` as "the durable publication
  Harold Ross founded from this very room" — the artifact that outlives any
  single lunch.

## Open questions

- **Issue cadence: weekly or monthly?** The real *New Yorker* is weekly; the
  daily column accumulates fast. Default: assemble on demand (`issue assemble
  --since <date>`), let a downstream timer decide cadence. Decided here: no
  built-in timer.
- **Published outward, or local like the zine?** The umbrella deferred this to
  `new-yorker`. Default **local** — issues land under `~/wintermute/new-yorker/
  issues/`. An outward static-site/gist export is a deferred follow-on, not in
  these 4 PRDs.
- **House-style: deterministic rules or API rewrite?** Mirrors the umbrella's
  grammar-vs-API tier. Default **deterministic** rule/transform pass (free,
  reproducible, testable); an `--lavish` API rewrite is a deferred flag, out of
  scope for v0.1.
- **Cover format: SVG or ASCII?** Both are deterministic-friendly. Default ship
  **both** from one motif model (ASCII to stdout/terminal, SVG to file); raster
  is out of scope.
