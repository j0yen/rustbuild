# PRD: vicious-circle-review — each persona reviews one artifact

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** `~/wintermute/vicious-circle`
**Vision:** visions/vicious-circle.md

## TL;DR

A registry of critics is inert until they critique something. This PRD makes the
table actually review the day's work: given an artifact (a haiku, a zine
excerpt, an unsent letter), each persona emits a structured `Verdict` — a witty
critique line in their own voice, a numeric score, and their stance. This is the
core read that turns write-only creative output into something answered.

## Why this exists

The six creative repos on this laptop — `~/wintermute/day-haiku`,
`~/wintermute/conversations-zine`, `~/wintermute/letters-we-never-sent`,
`~/wintermute/self-portrait`, `~/wintermute/ambient`,
`~/wintermute/wintermute-music` — produce artifacts no one responds to;
`visions/roundtable.md` names this the write-only problem. `review` is the first
response: it reads an artifact and produces verdicts.

The persona voices come from the registry built in
`PRD-vicious-circle-personas.md`. Critique-line generation is deterministic and
template/grammar driven by default — seeded by the persona's `tic` and the
artifact's surface features (length, line count, vocabulary) — in the spirit of
`~/wintermute/concord`'s `concord-deescalate/src/prompt.rs` tone templating, so
it runs free and reproducibly. A `--lavish` flag is reserved (no-op stub here)
for later Claude-API generation, mirroring the umbrella's `bon-mot` API tier.

## What this builds

Extends `~/wintermute/vicious-circle` (lib `vicious_circle` + `vicious-circle`
CLI).

Modules:
- `src/verdict.rs` — `Verdict { persona: String, target: String, line: String,
  score: f32, stance: Stance }` (serde). `target` is the artifact id/path.
  Score is clamped to `0.0..=10.0`.
- `src/artifact.rs` — `Artifact { id, source, kind, text }` and
  `Artifact::from_path(path) -> Result<Artifact>` (reads a text file; `kind`
  inferred from parent repo name / extension — haiku, letter, zine, etc.).
- `src/critic.rs` — `fn review(persona: &Persona, art: &Artifact) -> Verdict`.
  Deterministic: picks a line template keyed on `persona.stance` + the artifact
  surface metrics, fills the persona `tic`, and computes a stance-flavored score
  (e.g. Parker scores terse work higher, Kaufman penalizes structural slack).
- `src/main.rs` — add the `review` subcommand.

Deps (added): none beyond personas PRD; reuse `serde`, `clap`, `anyhow`.

CLI:
- `vicious-circle review <artifact-path>` — every persona reviews the artifact;
  prints a table of verdicts (persona, line, score).
- `vicious-circle review <artifact-path> --json` — emit `Vec<Verdict>` as JSON.
- `vicious-circle review <artifact-path> --persona <id>` — only that persona.
- `--lavish` flag parsed and accepted (documented as reserved; falls back to
  deterministic with a stderr note).

## Acceptance criteria

1. `cargo build` and `cargo test` succeed in `~/wintermute/vicious-circle`.
2. `vicious-circle review <file>` on a small text fixture produces exactly one
   `Verdict` per registered persona (5 by default), each with a non-empty
   `line`, a `score` in `0.0..=10.0`, and the persona's own `stance`.
3. `review` is deterministic: a unit test reviews the same fixture twice and
   asserts byte-identical `Vec<Verdict>` output.
4. Each persona's `line` contains a marker derived from its `tic` so verdicts are
   distinguishable by voice; a test asserts the five lines are pairwise distinct
   for one fixture.
5. `Verdict` round-trips through serde JSON; `review --json` emits a JSON array
   that re-parses to `Vec<Verdict>`.
6. `Artifact::from_path` correctly infers `kind` for a path under a
   `day-haiku`-style directory vs a `letters-we-never-sent`-style directory
   (test with two temp paths).
7. `review --persona parker <file>` emits exactly one verdict, by Parker.
8. `--lavish` is accepted without error and, with no API key configured, falls
   back to deterministic review (test asserts no panic, prints stderr note).
