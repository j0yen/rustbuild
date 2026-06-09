# Vision: vicious-circle — the critique ensemble

> Sub-vision of [roundtable](roundtable.md). The Vicious Circle was the
> Round Table's own name for itself — a ring of distinct voices that savaged
> and celebrated each other's work daily. This vision builds that ring.

## TL;DR

wintermute's creative wing is a pile of soloists. `~/wintermute/day-haiku`
writes a haiku to itself. `~/wintermute/conversations-zine` extracts moments
nobody answers. `~/wintermute/letters-we-never-sent` drafts letters that by
design go unread. `~/wintermute/self-portrait` watches CLAUDE_SELF diffs in
silence. `~/wintermute/ambient` and `~/wintermute/wintermute-music` sound for
an empty room. Every one of these is **write-only** — as the umbrella vision
[roundtable.md](roundtable.md) observes, the work is made and then nothing
answers it.

`vicious-circle` is the answer. It defines a registry of **critic personas**,
each modeled on a real Round Table member with a distinct critical stance and
a signature verbal tic, and runs a **critique round**: each persona reviews the
day's artifacts, then **roasts the others' verdicts** — the mutual sharp
critique that defined the real Circle. The single best line of the round is
**crowned**, and every verdict is written to a persistent **ledger** so the
downstream `conning-tower` column and `new-yorker` issues can read back-issues.

## End-state

You point `vicious-circle review` at the day's creative artifacts. Five
personas — Dorothy Parker (acid epigram), Robert Benchley (gentle absurdist),
Alexander Woollcott (grandiose enthusiast), George S. Kaufman (structural eye),
Edna Ferber (narrative realist) — each emit a structured `Verdict` with a witty
critique line, a score, and their stance. Then `vicious-circle roast` lets each
persona react to the others' verdicts — cross-verdicts, the table arguing with
itself. `vicious-circle crown` surfaces the day's `bon mot`: the single best
line, ranked by score and by how the peers reacted to it. Everything lands in a
`vicious-circle ledger`, an append-only JSONL record the rest of the roundtable
fleet reads.

It leans on the existing `~/wintermute/concord` crate
(`concord-deescalate`, with its `lexicon.rs` / `prompt.rs` / `types.rs`
persona-tone machinery) so the voices are real tone profiles, not just labels.

## Components (one bullet per PRD)

- **vicious-circle-personas** — the persona registry. A Rust lib + CLI defining
  each critic voice (name, era-real bio, critical stance, signature tic) loaded
  from a TOML personas file. `vicious-circle personas` lists them.
- **vicious-circle-review** — each persona reviews one artifact and emits a
  structured `Verdict { persona, target, line, score, stance }`.
- **vicious-circle-roast** — personas react to each *other's* verdicts (the
  mutual roasting); emits cross-verdicts.
- **vicious-circle-crown** — surface the single best line of the round (the day's
  `bon mot`) by score plus peer reaction.
- **vicious-circle-ledger** — persistent append-only record of every round's
  verdicts so back-issues and the conning-tower column can read them.

## Order

```
vicious-circle-personas (registry)
        │
        ▼
vicious-circle-review (per-artifact verdicts) ──> vicious-circle-ledger (persist)
        │                                              ▲
        ▼                                              │
vicious-circle-roast (cross-verdicts) ─────────────────┤
        │                                              │
        ▼                                              │
vicious-circle-crown (bon mot of the day) ─────────────┘
```

- `personas` ships first; it is the data foundation every other PRD loads.
- `review` and `ledger` can ship in either order after personas (`review`
  emits verdicts; `ledger` persists whatever verdicts exist, real or fixture).
- `roast` consumes `review`'s verdicts; `crown` consumes both `review` and
  `roast` to weight peer reaction.

All five build into one repo, `~/wintermute/vicious-circle`, as a single CLI
(`vicious-circle`, lib `vicious_circle`) so subcommands share the `Verdict`
type, the persona registry, and the ledger path. PRDs marked `rust-extend`
after the first to keep the type definitions in one crate.

## Open questions

- Do persona critique lines come from the Claude API (real generated wit, cost
  per round) or from `concord`-style grammar/tone templates (deterministic,
  free)? Default to deterministic templates seeded by the persona's tic + the
  artifact's surface features; gate API behind a `--lavish` flag. Mirrors the
  umbrella's `bon-mot` tiering question.
- How are "the day's artifacts" located? Default to a config-listed set of glob
  roots (the six creative repos' output dirs); defer the noon-pull ritual to the
  `the-lunch` sub-vision, which `review` will consume.
- Ledger format: JSONL append-only (chosen, simplest for back-issue reads) vs a
  `recall`-backed store. Start JSONL; a later `conning-tower` PRD can index it.
- Scoring scale and how peer-reaction weights into the crown — pin a concrete
  formula in `vicious-circle-crown` so the bon mot is reproducible.
