# PRD: thanatopsis-poker — personas bet chips on the best artifact and crown a winner with stakes

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** (existing repo) `~/wintermute/thanatopsis`
**Vision:** visions/thanatopsis.md

## TL;DR

`vicious-circle` crowns the day's best line by *straight scoring* — each persona
assigns a number, highest wins. That is a critic's verdict; it is not how the
real Thanatopsis club decided anything. They played poker. This PRD builds a
betting round: each persona is staked a pot of chips and *bets* on which artifact
on the day's table is the best, raising and calling on each other's bets. The
artifact that draws the most committed chips is crowned — a playful,
stake-weighted alternative to a critic's flat score, where conviction (how much a
persona is willing to commit) matters as much as opinion, and the betting history
is itself the entertainment.

## Why this exists

The day's table holds the six soloists' outputs — `~/wintermute/day-haiku`,
`~/wintermute/conversations-zine`, `~/wintermute/letters-we-never-sent`,
`~/wintermute/self-portrait`, `~/wintermute/ambient`, `~/wintermute/wintermute-music`.
The umbrella vision `visions/roundtable.md` frames the games as the generative,
playful counterpart to `vicious-circle`'s vicious critique; the Thanatopsis club
was, literally, a poker game. Poker gives the table a *second*, livelier way to
crown a favorite that complements (and can disagree with) the critique round's
flat scoring — disagreement between the two crownings is itself signal worth
publishing.

This PRD extends the `thanatopsis` crate scaffolded by `thanatopsis-charades`,
reusing its `Table`/`Artifact` input types (consumed from `the-lunch`, drafted
in parallel, slug `the-lunch`) and emitting the shared `GameResult` type that
`thanatopsis-parlor-ledger` persists. Persona conviction lines (the trash-talk
on a raise) reuse `bon-mot` word primitives (drafted in parallel, slug
`bon-mot`) for flavor; the betting *amounts* are deterministic, not random.

## What this builds

Extends `~/wintermute/thanatopsis` (lib `thanatopsis`, bin `thanatopsis`) with
the `poker` subcommand.

Modules:
- `src/poker.rs` —
  - `Stake` / `Chips` newtypes; each persona starts with an equal pot.
  - `preference(persona_seed, artifact) -> u32`: a deterministic preference
    score for one persona over one artifact, derived from the artifact's surface
    features (length, salient-word richness via `words`, `kind`) weighted by the
    persona's stance. This is the *judgment* the bet expresses — no card chance.
  - `bet_round(table, personas) -> BettingHistory`: each persona, in seat order,
    commits chips to its top-preferred artifact; on a later persona out-betting
    an artifact a persona already backed, that persona may *raise* (commit more)
    or *fold* its backing, by a deterministic conviction rule (commit more iff
    its preference margin exceeds the current high bet's margin). Records every
    bet/raise/call/fold as a `BetEvent { persona, artifact_id, action, chips }`.
  - `resolve(history) -> PokerOutcome { winner_artifact, pot_on_winner,
    runner_up, committed_by }`: the artifact holding the most committed chips at
    close is crowned.
  - `play(table) -> GameResult`: wraps the round into the shared `GameResult`
    with `game == "poker"`.
- `src/main.rs` — add the `poker` subcommand to the existing clap CLI.

CLI subcommands:
- `thanatopsis poker play --table <path> [--pot <chips>] [--seed <n>] [--json]`
  — runs a betting round; prints the betting history (who bet/raised/called/
  folded on what) and the crowned artifact with the pot on it; `--json` emits
  the `GameResult`.
- `thanatopsis poker odds --table <path> [--seed <n>]` — prints each persona's
  preference ranking of the table (the "tells") without running the betting.

Deps: reuse the crate's existing `clap`/`serde`/`serde_json`/`anyhow`/`sigpipe`;
optional `bon-mot` feature for conviction-line flavor.

The betting MUST be deterministic given `(table, seed, pot)`: same inputs always
crown the same artifact with the same history — poker here is preference-betting,
not chance.

## Acceptance criteria

1. `cargo build` and `cargo test` succeed; clippy adds no new warnings beyond
   the repo baseline.
2. `thanatopsis poker play --table tests/fixtures/table.json --seed 1 --json`
   emits a valid `GameResult` with `game == "poker"`, a `winner_artifact` that
   is an id present on the table, and a non-empty betting history.
3. Determinism: a test asserts two runs of `play` with identical
   `(table, seed, pot)` produce byte-identical `GameResult` JSON.
4. Conservation: a test asserts total chips committed across all `BetEvent`s
   never exceeds the sum of the personas' starting pots (no chips invented).
5. A test constructs a fixture where one artifact is clearly strongest for a
   majority of personas and asserts it is crowned `winner_artifact`; a second
   fixture where preferences split shows a different, contested winner with a
   smaller `pot_on_winner` margin.
6. The betting history contains at least one `raise` or `fold` event on a
   fixture designed to provoke one (a persona out-bet on its backed artifact),
   proving the raise/fold rule fires, not just opening bets.
7. `thanatopsis poker odds` prints a per-persona ranking covering every artifact
   on the table; the crate still builds `--no-default-features` and
   `--features bon-mot`.
