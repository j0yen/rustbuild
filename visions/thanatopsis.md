# Vision: thanatopsis — the games club

> Sub-vision of [roundtable](roundtable.md). The Round Table didn't only
> critique — it *played*. Many of its members belonged to the "Thanatopsis
> Literary and Inside Straight Club," a weekly poker game that also ran
> charades, the cross-word craze of the 1920s, and an elaborate parlor game
> called "Murder." This vision builds that table's playful, generative side:
> agents playing creative games with the day's material instead of only
> judging it.

## TL;DR

The roundtable fleet so far is built to *critique*: `vicious-circle` reviews the
day's artifacts and roasts the verdicts; `conning-tower` publishes the best
line. Critique is one half of what the real Circle did. The other half was
**play** — language games and parlor games that were generative, social, and
fun, and that incidentally exposed which of the day's work was distinct,
legible, and sound.

`thanatopsis` builds that half. It takes the same day's-artifacts table that
`the-lunch` assembles and the same word primitives `bon-mot` provides, and runs
four games over them — charades (describe an artifact without naming it; others
guess), poker (personas *bet* on the best artifact and crown a winner with
stakes), a cross-word (build a small puzzle from the day's vocabulary), and
Murder (plant a flaw in an artifact; the table hunts the "murderer"). A fifth
PRD keeps a persistent ledger of every game played so the games compose with
`conning-tower` and the periodical's back-issues.

Where `vicious-circle` asks "is this any good?", `thanatopsis` asks "is this
*distinct*, is this *legible*, is this *sound*, and which of these is the table's
*favorite*?" — and answers by playing.

## End-state

At the games hour (after the lunch and the critique round), you point
`thanatopsis` at the day's table:

- `thanatopsis charades` — a persona is dealt one artifact and must describe it
  *without naming it or quoting it*; the other personas guess which artifact on
  the table was being described. A round scores how legibly the day's outputs
  read: if no one can guess, the artifact is muddy; if everyone guesses
  instantly, it is vivid (or trite).
- `thanatopsis poker` — each persona is staked a pot of chips and *bets* on
  which artifact is the best, raising and calling on each other's bets. The
  artifact that draws the most committed chips is crowned, with the betting
  history as a playful, stake-weighted alternative to `vicious-circle`'s
  straight scoring.
- `thanatopsis crossword` — harvests the day's vocabulary (the salient words
  that actually appeared in the artifacts) and lays a small interlocking
  cross-word grid with clues, reusing `bon-mot`'s word primitives.
- `thanatopsis murder` — deliberately plants one defect in an artifact (a broken
  rhyme, a swapped word, an off-by-one), shuffles it back onto the table, and
  the personas hunt the "murderer"; scores whether the table can find a planted
  flaw — a creative QA game.
- `thanatopsis ledger` — every game, its players, and its outcome are appended
  to a persistent record the conning-tower column and the periodical read.

The end-state is that the day's creative work doesn't only get *judged* — it
gets *played with*, and the play yields signal (legibility, favor, robustness)
that pure critique misses.

## Components (one bullet per PRD)

- **thanatopsis-charades** — a persona describes a dealt artifact without naming
  it; the others guess which artifact it was. Scores legibility/distinctness of
  the day's outputs. Reuses `bon-mot` word primitives to build the clue.
- **thanatopsis-poker** — personas are staked chips and bet on the best
  artifact; raises/calls resolve to a crowned winner with stakes. A playful,
  bet-weighted alternative to vicious-circle's straight critique.
- **thanatopsis-crossword** — harvest the day's vocabulary and lay a small
  interlocking cross-word with clues, reusing `bon-mot`'s word primitives.
- **thanatopsis-murder** — plant one deliberate defect in an artifact; the
  personas hunt the "murderer." A creative QA / find-the-planted-flaw game.
- **thanatopsis-parlor-ledger** — persistent append-only record of every game
  played and its outcome, so the games compose with `conning-tower` and
  `back-issues`.

## Order

```
the-lunch (table object) ──┐
                           ├──> thanatopsis-charades ──┐
bon-mot (word primitives) ─┘     thanatopsis-poker     ├──> thanatopsis-parlor-ledger
                                 thanatopsis-crossword  │
                                 thanatopsis-murder ────┘
```

- The four games depend only on the upstream table object (`the-lunch`) and, for
  charades/crossword, `bon-mot`'s word primitives. They are mutually
  independent and can ship in any order.
- `thanatopsis-parlor-ledger` consumes the games' outcome records; it ships
  after at least one game exists, but the ledger persists whatever `GameResult`
  values it is handed (real or fixture), so it can be built early against
  fixtures.

All five build into one repo, `~/wintermute/thanatopsis`, as a single CLI
(`thanatopsis`, lib `thanatopsis`) so the subcommands share the `Artifact` /
`Table` input types, the `GameResult` output type, and the ledger path. The
first PRD scaffolds the crate; later PRDs are `rust-extend` to keep the shared
types in one place.

## Open questions

- Do the games' descriptive lines (charades clues, poker trash-talk, murder
  accusations) come from the Claude API (real generated wit, cost per game) or
  from `bon-mot`'s deterministic grammar/word primitives? Default deterministic,
  seeded by the persona tic + artifact surface features; gate API behind a
  `--lavish` flag, mirroring the umbrella `bon-mot` tiering decision.
- How is "the day's table" located? Consume `the-lunch`'s table object (a JSON
  `Table { date, artifacts: [Artifact { id, source, kind, text, path }] }`);
  until `the-lunch` ships, accept a `--table <path>` JSON fixture and a glob
  fallback over the six creative repos' output dirs.
- Is the betting in poker chance-based (deal cards) or pure preference (bet on
  judgment)? Choose preference-betting: deterministic, reproducible, and it is
  the *artifact* being bet on, not a card hand. Pin the bet/raise/call mechanic
  concretely in `thanatopsis-poker`.
- Where do `bon-mot`'s word primitives live as a dependency — a path dep on the
  parallel `~/wintermute/bon-mot` crate, or a vendored minimal tokenizer? Prefer
  an optional path dep with a small built-in fallback so charades/crossword
  build before `bon-mot` lands.
