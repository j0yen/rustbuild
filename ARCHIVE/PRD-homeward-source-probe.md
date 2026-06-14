# PRD: homeward-source-probe — onboard a portal by probing, not by hand

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

Adding a Socrata source still requires a human to read one dataset's column docs
and hand-derive a `SocrataColumnMap`. This PRD ships `homeward-connectors probe
<domain> <dataset_id>`: it hits the SODA metadata + a one-row sample, decides
whether the dataset is a usable STRAY-bearing animal-intake feed, and emits either
a **draft `sources.toml` entry** (best-guess column mapping for a human to review)
or an **honest red verdict** explaining why it isn't usable. Onboarding becomes
probe → review → commit instead of manual archaeology.

## Why this exists

Phase-1 inspection (2026-06-13) + the registry PRD:

- Every `SocrataColumnMap` in `socrata.rs` (Austin `fdzn-9yqv`, Dallas
  `qgg6-h4bd`, Sonoma `924a-vesw`, Long Beach) was hand-mapped from that
  dataset's column names — `intake_type` here is `Intake_Type` there is
  `intake_condition` somewhere else. The vision says hundreds of heterogeneous
  portals exist; hand-mapping each is the bottleneck that keeps catchment at four.
- homeward-source-registry (its dependency) makes a `sources.toml` *loadable*, but
  a human still has to author each entry. The probe authors the draft.
- The vision's own "Why now" research established the discriminating signal:
  municipal STRAY feeds uniquely carry an **intake-type column whose values
  include `STRAY`** (often with `Found_Location`, `Chip_Status`). That is exactly
  what a probe can detect mechanically.
- Honest verdicts matter: a portal with no intake-type column is an adoptable-only
  feed, not a stray feed, and must be reported as such — never silently mapped to
  a wrong column ([[feedback_verify_before_concluding]]).

## What this builds

A `probe` subcommand on the `homeward-connectors` binary:

- **Input:** `homeward-connectors probe <domain> <dataset_id> [--name <slug>]
  [--json]`.
- **Fetch (polite, reusing the crate's existing HTTP core):**
  - the SODA column metadata (`https://<domain>/api/views/<dataset_id>/columns.json`
    or the dataset metadata endpoint), and
  - a one-row sample (`https://<domain>/resource/<dataset_id>.json?$limit=1`)
    to read real column names + a value sample.
  - conditional/identifying user-agent, honor 429/`Retry-After`, the same
    politeness the connectors already implement. No bulk fetch.
- **Classify columns** by name+value heuristics into the `SocrataColumnMap` slots:
  - required: `animal_id`, `animal_type`/species, `intake_type`. A candidate
    `intake_type` column is **confirmed STRAY-bearing** only if a cheap
    `$where`/`$select=distinct` (or the sampled value) shows a `STRAY`/`FOUND`
    value — name-match alone is a *guess*, value-match is a *confirmation*, and the
    verdict must distinguish the two.
  - optional: `intake_date`, `found_location`, `chip_status`, `kennel_status`,
    `breed`, `name`, `color`, `outcome_date`.
- **Emit one of:**
  - **GREEN** — a ready-to-paste `[[socrata]]` TOML block (the registry's format)
    with every mapped column, each annotated `# confirmed` or `# guessed` so the
    reviewer knows what to check.
  - **RED** — a verdict naming the missing required signal (e.g. "no column whose
    values include STRAY; this is an adoptable-only feed" or "no animal_id-like
    column"). Exit non-zero so scripted onboarding can branch.
- **No writes.** The probe prints; the human commits. (The catalog PRD is what
  collects committed entries.)

## Acceptance criteria

1. `homeward-connectors probe data.austintexas.gov fdzn-9yqv` against a recorded
   fixture of Austin's columns/sample emits a GREEN `[[socrata]]` block whose
   required mappings (`animal_id`, `animal_type`, `intake_type`) match the
   hand-authored `SocrataConfig::austin()` column map.
2. The emitted block is valid input to homeward-source-registry's loader: a test
   feeds the probe's GREEN output through `SourceCatalog::from_path` and gets a
   usable config back (round-trip probe → registry).
3. A fixture with no STRAY-bearing intake-type column yields a RED verdict naming
   the missing signal and a non-zero exit code; a fixture missing `animal_id`
   likewise REDs with a distinct message.
4. Each mapped column in GREEN output is annotated `confirmed` (value-verified) vs
   `guessed` (name-only); the `intake_type` slot is `confirmed` only when a STRAY
   value was actually observed in the sample/distinct check.
5. Probe failures (404 dataset, network error, throttling) are reported as a
   typed error with the dataset id, not a panic; `--json` emits a structured
   `{verdict, reason, draft}` object.
6. `cargo test` green using **recorded HTTP fixtures** (no live network in tests);
   `cargo build --release` ships the subcommand. MSRV 1.85, no let-chains.

## Out of scope

- Auto-discovering which `{domain, dataset_id}` pairs to probe (left un-dreamt in
  the vision — needs Socrata federated-catalog research).
- OpenDataSoft/ArcGIS dialects (Socrata/SODA only here).
- Committing the catalog (homeward-source-catalog).
