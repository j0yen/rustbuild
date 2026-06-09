# PRD: conning-tower-constant-reader — the feedback loop that answers the column

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** (existing repo) `~/wintermute/conning-tower`
**Vision:** visions/conning-tower.md

## TL;DR

Every artifact in wintermute's creative wing is write-only because nothing answers
it. A published column is no different *until a reader replies*. Dorothy Parker
reviewed books in *The New Yorker* under the byline "Constant Reader" — the
table's own readership made visible ("This is not a novel to be tossed aside
lightly. It should be thrown with great force."). This PRD builds that reader: a
response filed against a published column, recorded persistently, and exported as
a bias signal that weights tomorrow's crowning toward reader-approved voices.

## Why this exists

The umbrella [roundtable.md](visions/roundtable.md) names the core disease —
"a pile of write-only artifacts" — and the cure as "a *conversation* — voices
answering voices." The whole `conning-tower` chain so far (column → contributors →
syndicate) is still one-directional: the table speaks, no one replies. The
upstream `PRD-vicious-circle-crown` ranks the bon mot "by score plus peer
reaction"; a Constant Reader response is a *reader* reaction the crown can also
weigh — closing the loop. `~/wintermute/recall` is the natural store for the
running reader record (episodic memory). This PRD is the first link in the chain
that writes *back* toward the upstream circle instead of only consuming it.

## What this builds

Extends `~/wintermute/conning-tower` (lib `conning_tower`, CLI `conning-tower`).

- **`reader.rs`** — the `Response` type: `{ column_date, line_ref, reader, verdict:
  Landed|Missed|Indifferent, note, ts }`. `line_ref` points at a line in the
  named column (the bon mot or a runner-up, by index). Responses append to a local
  `responses.jsonl` (default under the columns dir, `--responses` overridable).
- **`bias.rs`** — aggregate responses into a per-persona bias export:
  `{ persona, net_landed, response_count }` summed over the response history,
  written as a JSON the upstream `vicious-circle-crown` can read (a stable
  documented path/format; conning-tower only *produces* it, never imports the
  crown).
- **CLI:**
  - `conning-tower read --date YYYY-MM-DD [--line N] --verdict landed|missed|
    indifferent [--note TEXT] [--reader user|<name>]` — file a response against a
    syndicated column (default line = the bon mot, index 0). `--reader user`
    records the passed note verbatim; persona-generated responses are deferred to a
    `--lavish` API tier per the vision (flag accepted, returns "deferred" for now).
  - `conning-tower bias [--responses PATH] [--out PATH|-] [--format json]` — emit
    the aggregated per-persona bias export.
- Deps: reuse existing; optional `recall` mirroring is an additive flag
  (`--mirror-recall`) that shells to the `recall` binary if present, never fatal.

## Acceptance criteria

1. `conning-tower read --date 2026-06-08 --verdict landed --reader user --note "this
   one landed" --responses <tmp>` appends one JSON line to `<tmp>` with
   `{column_date:"2026-06-08", line_ref:{index:0}, reader:"user",
   verdict:"landed", note:"this one landed", ts}` and exits 0; a second `read`
   appends a second line (append-only, never truncates).
2. `--line N` records `line_ref.index = N`; an `--verdict` outside the three allowed
   values is a usage error (exit 2) and writes nothing.
3. Filing a response for a `--date` with no syndicated column in the columns dir
   exits non-zero (exit 3) naming the missing column, and writes nothing —
   responses can only answer a published column.
4. `conning-tower bias --responses <fixture>` aggregates per persona: `net_landed`
   = (#landed − #missed) for lines that persona placed, `response_count` = total
   responses touching that persona's lines; output is sorted persona ascending and
   is byte-stable across runs. (Persona is resolved by reading the referenced
   column's attribution from `PRD-conning-tower-contributors`.)
5. `conning-tower bias --out -` writes the JSON to stdout; `--out PATH` writes to
   PATH and creates the parent dir. The exported schema is documented in the repo
   README as the contract the upstream crown reads.
6. `--mirror-recall` when the `recall` binary is absent prints a warning and still
   completes the local append with exit 0 (best-effort, never fatal).
7. `--reader <non-user>` without `--lavish` exits 0 having recorded nothing and
   prints `persona responses deferred` (the deferred tier per the vision).
8. Each AC has a matching integration test under `tests/acceptance_ac<n>.rs` using
   committed fixtures and temp dirs.
