# Vision: conning-tower — publishing the best

> Sub-vision of [roundtable](roundtable.md). Named for Franklin Pierce Adams's
> syndicated newspaper column "The Conning Tower," which printed Round Table
> members' epigrams and light verse and made them famous overnight. FPA was the
> table's publisher — the place a line went to stop being lunch talk and start
> being literature. This vision builds that column.

## TL;DR

The roundtable fleet now *makes* wit and *judges* it — `vicious-circle` crowns a
`bon mot` each round and writes every `Verdict` to an append-only ledger
(`PRD-vicious-circle-ledger`), and `bon-mot` scores generated lines. But a crown
that lands in a JSONL file is just another write-only artifact — exactly the
disease the umbrella [roundtable.md](roundtable.md) names: work that is made and
then nothing answers it. `~/wintermute/conversations-zine` is the cautionary
tale — its own README says the bottleneck is the moment-extractor and that
"layout/print/mail are explicitly downstream and human-driven," so the extracted
moments sit unread.

`conning-tower` is the press. It reads the crowned lines and best verdicts from
the ledger, composes them into a single **dated Markdown column** with an
attributed **contributor masthead**, **syndicates** it into a columns directory
(and optionally the zine's input), runs a **Constant Reader** feedback loop (after
Dorothy Parker's book-review byline) where a reader answers the column and that
answer biases future crowning, and keeps every column in a queryable
**back-issues** archive leaning on `~/wintermute/recall` episodic memory.

## End-state

At day's end (after the lunch convenes and the circle crowns), you run
`conning-tower compose`. It reads today's crowned line and the top verdicts from
`~/wintermute/vicious-circle`'s ledger, attributes each to its persona and source
artifact, and renders `columns/YYYY-MM-DD.md` — a column with a dated headline,
the day's bon mot set in display, runner-up lines, and a masthead crediting the
contributing personas. `conning-tower syndicate` routes that column into its
canonical home and, on a flag, drops it into `~/wintermute/conversations-zine`'s
input so the zine finally has something answered to print. `conning-tower read`
lets the user (or a persona) file a **Constant Reader** response — "this one
landed," "tonstant weader fwowed up" — recorded as a verdict-on-the-verdict that
`conning-tower bias` exports back so tomorrow's crown weights reader-approved
voices. `conning-tower archive` and `conning-tower back-issue <date>` give a
persistent, queryable run of every column ever printed.

The end-state: the day's single best line stops being a row in a ledger and
becomes a published, attributed, answerable column — with a reader on the other
end whose taste feeds back into the table.

## Components (one bullet per PRD)

- **conning-tower-column** — the daily artifact. Read the ledger's crowned line +
  top verdicts and compose one dated Markdown column. `conning-tower compose`.
- **conning-tower-contributors** — attribution + masthead. Map each printed line
  to its persona and source artifact; render a credited contributor masthead.
- **conning-tower-syndicate** — publish/route the column to `columns/` and,
  optionally, into the zine input. `conning-tower syndicate`.
- **conning-tower-constant-reader** — the feedback loop. A reader answers a column;
  responses are recorded and exported to bias future crowning.
- **conning-tower-back-issues** — persistent, queryable archive of every column,
  leaning on `~/wintermute/recall` episodic memory.

## Order

```
conning-tower-column (compose the daily column)
        │
        ▼
conning-tower-contributors (attribute + masthead)
        │
        ▼
conning-tower-syndicate (route/publish) ──> conning-tower-back-issues (archive + query)
        │
        ▼
conning-tower-constant-reader (reader answers; bias the crown)
```

- `column` ships first; it defines the `Column` type and the ledger reader every
  other PRD loads. The rest are `rust-extend` on the same crate so the type, the
  ledger path, and the columns dir live in one place.
- `contributors` enriches the column with attribution before it is published.
- `syndicate` and `back-issues` can ship in either order after `contributors`
  (one routes the file out, one files it into the archive).
- `constant-reader` is last: it consumes a published column and writes a bias
  export the upstream `vicious-circle-crown` can read next round.

All five build into one repo, `~/wintermute/conning-tower`, as a single CLI
(`conning-tower`, lib `conning_tower`).

## Upstream / downstream

- **Upstream:** reads the ledger from `PRD-vicious-circle-ledger` (the append-only
  JSONL of `Verdict { persona, target, line, score, stance }` and the crowned bon
  mot from `PRD-vicious-circle-crown`); the scored lines from the `bon-mot`
  sub-vision flow through that same crown/ledger path.
- **Downstream:** `new-yorker` binds accumulated columns into issues; the Constant
  Reader bias export feeds back into `PRD-vicious-circle-crown`.
- **Leans on:** `~/wintermute/recall` (episodic memory for back-issues),
  `~/wintermute/conversations-zine` (optional syndication target), the
  `daily-receipt` + `cadence` noon/daily-render ritual for cadence.

## Open questions

- Is the column published anywhere outward (gist/static site) or kept local like
  the zine? Default local (`columns/` dir); outward is a deferred flag, mirroring
  the umbrella's `new-yorker` deferral.
- Does the Constant Reader response come from the real user (a prompt) or a
  persona (generated)? Support both: `--reader user` records a passed-in line,
  `--reader <persona>` is deferred to a `--lavish` API tier, mirroring `bon-mot`.
- Back-issues: write columns *into* recall as episodic memories, or keep a local
  index and only optionally mirror to recall? Default to a local JSONL index that
  is self-sufficient; recall mirroring is an additive flag so the archive never
  hard-depends on recall's schema.
