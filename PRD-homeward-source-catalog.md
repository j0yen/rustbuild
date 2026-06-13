# PRD: homeward-source-catalog — the committed map of where homeward listens

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

homeward-source-registry makes a `sources.toml` loadable and homeward-source-probe
authors validated entries — but the repo ships **no catalog**, so a fresh deploy
still listens to only the four hardcoded cities. This PRD commits a real,
validated `deploy/sources.toml` covering the known-good municipal STRAY feeds
(the four built-ins promoted to data + the next tier), each entry stamped with its
probe verdict and a last-validated date, plus a test that the shipped catalog
parses and loads. This is the data deliverable that actually widens catchment.

## Why this exists

- The registry (foundation) and probe (validator) are *mechanisms*; without a
  committed catalog they widen nothing on their own. The funnel stays at four
  cities until someone fills the file.
- Phase-1 inspection (2026-06-13): the only Socrata sources that exist anywhere in
  the repo are the four `const fn`s in `socrata.rs`. There is no `sources.toml` on
  disk (`deploy/` holds only systemd units + `homeward.env.sample`).
- The vision's research already named the validated tier: Austin `fdzn-9yqv`,
  Dallas `qgg6-h4bd`, Sonoma `924a-vesw`, Long Beach, plus Bloomington and "and
  *hundreds* more" — this PRD promotes the verified ones into data and adds the
  next municipalities the probe greenlights, so the knowledge is shipped, not
  trapped in four constructors.
- Honest provenance: each entry records *when* it was last validated and *how*
  (probe GREEN vs hand-authored), because a municipal dataset can be renamed or
  retired and a stale entry silently stops returning records ([[feedback_verify_before_concluding]]).

## What this builds

- **`deploy/sources.toml`** — the committed catalog. Starts with the four
  built-ins migrated to the registry's TOML format (byte-for-byte equivalent
  column maps), then adds every additional municipality whose `probe` returns
  GREEN. Each `[[socrata]]` entry carries comment metadata:
  ```toml
  # validated: 2026-06-13  via: probe-green  metro: "Austin, TX"
  ```
- **A small generator/refresh path** — a `just`/shell recipe or a documented
  `homeward-connectors probe … >> deploy/sources.toml` workflow (no new binary if
  the probe already prints paste-ready blocks) so re-validating the catalog is a
  one-command sweep, not hand-editing.
- **`deploy/CATCHMENT.md`** — documents the catalog: which metros are covered,
  each source's last-validated date + verdict, the re-validation command, and an
  explicit list of metros known to lack a Socrata feed (the honest holes, so
  coverage gaps are visible — pairs with homeward-coverage-report).
- **A load test** — a Rust test (in `homeward-connectors`) that parses the shipped
  `deploy/sources.toml` via the registry loader and asserts: it parses, every
  entry has the three required column-map fields, and the four built-in cities are
  present and match their `const fn` equivalents (so the migration is faithful).
- **Wire the default** — `deploy/homeward.env.sample` and the orchestrate
  `homeward` wrapper default `HOMEWARD_SOURCES=deploy/sources.toml` so a real
  deploy listens to the full catalog, not the four-city fallback.

## Acceptance criteria

1. `deploy/sources.toml` exists, parses through homeward-source-registry's
   `SourceCatalog::from_path` with zero errors, and contains at least the four
   built-in cities plus **at least two** additional municipalities not previously
   hardcoded.
2. The four migrated built-in entries match `SocrataConfig::austin()/dallas()/
   sonoma()/long_beach()` field-for-field (name, domain, dataset_id, every column
   map slot) — proven by a test, so the migration introduces no drift.
3. Every `[[socrata]]` entry carries a `validated:` date and a `via:` provenance
   comment; entries added beyond the built-ins are marked `probe-green`.
4. `deploy/CATCHMENT.md` lists each source's metro + last-validated date, the
   re-validation command, and a "known gaps" section naming at least one major
   metro with no Socrata STRAY feed.
5. `deploy/homeward.env.sample` sets `HOMEWARD_SOURCES` to the shipped catalog, and
   the `homeward` orchestrate wrapper passes it through, so a default deploy loads
   the catalog (not the four-city fallback).
6. `cargo test` green (the catalog-load test runs in CI); the additional
   municipalities are real datasets whose ids resolve (validated by probe at
   authoring time — recorded in CATCHMENT.md, not asserted via live network in
   tests). MSRV 1.85.

## Out of scope

- Auto-discovering portals (un-dreamt; needs catalog-API research).
- Non-Socrata source families (OpenDataSoft/ArcGIS — future fleet).
- The coverage/health view of a running store (homeward-coverage-report).
