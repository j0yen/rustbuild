# Vision: roundtable — the Algonquin comes to wintermute

> Drafted 2026-06-08 from the Algonquin Hotel, NYC. Seed: the Round Table —
> the "Vicious Circle" that lunched here daily 1919–1929. Dorothy Parker,
> Robert Benchley, George S. Kaufman, Alexander Woollcott, Edna Ferber,
> Franklin P. Adams, Harold Ross (who founded *The New Yorker* from this
> room in 1925). What they actually did: gathered daily, played language
> games, critiqued each other's work mercilessly and wittily, and published
> the best of it.

## TL;DR

wintermute already has a **creative wing** — but every voice in it works
alone. `day-haiku` writes a haiku to itself. `conversations-zine` extracts
moments to a file no one answers. `letters-we-never-sent` drafts letters
that, by design, no one reads. `self-portrait` watches CLAUDE_SELF diffs in
silence. `ambient` turns telemetry into sound for an empty room.

The Algonquin Round Table was the missing thing: **not another soloist, but
the lunch table itself** — a circle of distinct voices that *convene* around
the day's work, *play* with language, *critique* each other sharply, and
*publish* the wit that survives the table. `roundtable` builds that table.

It is the social, critical, and playful layer over the solo creative outputs
already on this laptop. Where `/build` and `/dream` are the workshop,
`roundtable` is the lunch — the place the work goes to be argued with.

## End-state

At noon (a timer, after the Round Table's real cadence), the day's creative
artifacts — the haiku, the zine excerpts, the unsent letter, the self-portrait
diff, the ambient cues — are pulled onto a table. A circle of **personas**,
each with a distinct critical voice modeled on a real member, reviews them:
Parker's acid epigram, Benchley's gentle absurdism, Woollcott's grandiose
enthusiasm, Kaufman's structural eye, Ferber's narrative realism. They roast
each other's verdicts. They play the Round Table's games — "I can give you a
sentence," charades, the cross-word, the game of Murder. The single best line
of the day — the **bon mot** — is crowned, attributed, and published into a
running column (`conning-tower`, after FPA's). Columns accumulate into issues
of a small periodical (`new-yorker`) with a house style and a masthead.

The end-state is that creativity on this laptop stops being a pile of
write-only artifacts and becomes a *conversation* — voices answering voices,
the best of it printed, the rest composted with a witticism.

## Sub-visions (the fleet, ~30 PRDs)

The Round Table was many institutions at once. Each becomes a sub-vision with
its own doc and PRD cluster:

- **vicious-circle** — the critique ensemble. A registry of persona voices and
  a critique round where each reviews the day's artifacts, then roasts the
  others' takes. (~5 PRDs) → `visions/vicious-circle.md`
- **bon-mot** — the wit engine. Language games and epigram generation under
  constraint: "I can give you a sentence," the epigram forge, anagrams, the
  telegram game, a wit-scorer, cross-form transforms. (~6 PRDs) →
  `visions/bon-mot.md`
- **conning-tower** — publishing the best, after FPA's syndicated column.
  Compose the day's crowned lines into a column, attribute contributors,
  syndicate it, run the "Constant Reader" feedback loop, archive back-issues.
  (~5 PRDs) → `visions/conning-tower.md`
- **the-lunch** — the daily convening ritual. Pull the day's artifacts onto
  the table at noon, seat the right personas, set the agenda, keep the minutes,
  wire the standing invitation (timer/hook). (~5 PRDs) → `visions/the-lunch.md`
- **thanatopsis** — the games club (after the Thanatopsis Literary & Inside
  Straight poker club). Charades, betting-poker on the best artifact, a
  collaborative cross-word from the day's vocabulary, the game of Murder
  ("find the planted flaw"). (~5 PRDs) → `visions/thanatopsis.md`
- **new-yorker** — the durable publication Harold Ross founded from this very
  room. A masthead, a house-style enforcer ("not for the old lady in
  Dubuque"), an issue-assembler that binds columns into a periodical, a
  generative cover. (~4 PRDs) → `visions/new-yorker.md`

## Order

```
the-lunch (convening) ──┐
                        ├──> vicious-circle (critique) ──> conning-tower (publish) ──> new-yorker (bind)
bon-mot (wit engine) ───┘                            └──> thanatopsis (games)
```

- `the-lunch` and `bon-mot` are the foundations: the gathering and the wit
  primitives. Either can ship first.
- `vicious-circle` consumes both: it convenes (lunch) and uses wit primitives
  (bon-mot) to critique.
- `conning-tower` and `thanatopsis` consume the circle's verdicts.
- `new-yorker` binds the columns; it ships last.

## Why this is honest, not fan-fiction

Every sub-vision cites real artifacts already on this laptop:
- `~/wintermute/day-haiku/`, `conversations-zine/`, `letters-we-never-sent/`,
  `self-portrait/`, `ambient/`, `wintermute-music/` — the solo creative outputs
  the table critiques.
- `~/wintermute/concord/` — an existing de-escalation/persona-tone crate the
  vicious-circle personas can lean on.
- `~/wintermute/recall/` — episodic memory for the column's back-issues and the
  Constant Reader loop.
- The `cadence` vision + `daily-receipt` already establish the noon-render
  ritual `the-lunch` extends.

## Open questions

- Should personas call the Claude API (real generated wit, cost per lunch) or
  be template/grammar-driven (deterministic, free)? Likely a tier: cheap
  grammar default, API on a `--lavish` flag. Decide in `bon-mot`.
- Is the periodical published anywhere outward (a static site, a gist) or kept
  local like the zine? Defer to `new-yorker`; default local.
- One table per day, or convene on-demand when N new artifacts accumulate?
  Decide in `the-lunch`.
