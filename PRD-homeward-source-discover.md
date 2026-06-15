# PRD: homeward-source-discover

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/homeward
**Vision:** visions/homeward.md (Reach fleet)

## TL;DR

Onboarding a new shelter feed today starts with a human already knowing a
`{domain, dataset_id}` — `probe` validates it, but nobody hands you the candidates.
Finding the "hundreds more" STRAY portals the vision names is manual archaeology.
This PRD adds a `homeward-connectors discover` subcommand that crawls the public
open-data **catalog** APIs (the Socrata federated catalog and the OpenDataSoft
catalog discovery endpoint) for animal-intake datasets and emits a ranked list of
*candidate* `{family, domain/base_url, dataset_id}` for `probe` to validate. It
turns "hand-fed dataset ids" into a `discover → probe → review → commit` pipeline.
It never auto-commits a source — discovery proposes, a human + `probe` dispose.

## Why this exists

Phase-1 live inspection (2026-06-14):

- The Catchment fleet shipped `probe` (validate a known `{domain, dataset_id}`) and
  `coverage` (map live/silent sources) but explicitly left as un-dreamt:
  *"Auto-discovery of new portals (crawl the Socrata federated catalog for
  animal-intake datasets) rather than hand-fed `{domain, dataset_id}` — research
  whether the catalog API exposes enough to filter to STRAY feeds."*
  (visions/homeward.md, Catchment "still un-dreamt").
- `deploy/sources.toml`'s header literally instructs a human to find and add
  sources by hand. The funnel's growth rate is bottlenecked on manual discovery.
- The Socrata federated catalog API (`api.us.socrata.com/api/catalog/v1`) supports
  full-text `q=` and `only=dataset` filtering and returns each dataset's domain +
  resource id + column names — enough to filter to animal-intake datasets and rank
  by whether a STRAY-bearing intake-type column is plausibly present. OpenDataSoft
  portals expose an analogous catalog discovery endpoint. This is the research
  question the Catchment fleet flagged, answered: the catalog APIs *do* expose
  enough to filter to candidates (final STRAY confirmation stays with `probe`,
  which samples a live record).
- `homeward-source-family` (dependency) defines the `{family, …}` candidate shape
  discover emits and probe consumes.

## What this builds

A `discover` subcommand in `homeward-connectors` (`src/discover.rs` or alongside
`probe.rs`).

### Shape

- `homeward-connectors discover [--families socrata,opendatasoft] [--metro <str>]
  [--limit N] [--json]`:
  - Queries the Socrata federated catalog API with animal-intake search terms
    (`animal intake`, `stray`, `animal services`, `shelter`), `only=dataset`,
    paged; collects `{domain, dataset_id (resource id), name, column names}`.
  - Queries the OpenDataSoft catalog discovery endpoint similarly when
    `opendatasoft` is in `--families`.
  - Scores each candidate on signals available *without* a per-dataset probe:
    name/description match to animal-intake, presence of an intake-type-like column
    name, presence of an animal-type-like column. Drops obvious non-matches.
  - Emits a ranked candidate list — human-readable table by default, `--json` for
    piping — each row carrying `{family, domain|base_url, dataset_id, score,
    matched_signals}` and the exact `probe` command to validate it.
  - **Honest:** discover never writes `deploy/sources.toml` and never claims a
    candidate is a working source — only `probe` (which samples a live record) can
    confirm STRAY-bearing. Output explicitly frames rows as "candidates to probe."
- Reuse `homeward-connectors/src/http.rs` `PoliteClient`; reuse the column-name
  heuristics already in `probe.rs` where they apply to catalog column lists.

### Constraints

- MSRV 1.85, no let-chains. Strict clippy bar (no `unwrap`/`expect`/`panic`
  outside tests).
- Tests run against a recorded catalog-API JSON fixture, never the live catalog.
  Ship a captured `api.us.socrata.com/api/catalog/v1` sample containing a mix of
  animal-intake and unrelated datasets and assert the ranking/filtering.
- ArcGIS Hub has no single federated catalog of the same shape; `--families` for
  discover covers `socrata` and `opendatasoft`. ArcGIS candidates remain a
  `probe --family arcgis <service_url>` path (documented as a known limit, logged —
  per [[feedback_verify_before_concluding]], state the coverage gap, don't imply
  ArcGIS is auto-discovered when it isn't).

## Acceptance criteria

1. `homeward-connectors discover` against a fixture Socrata catalog response emits
   a ranked candidate list in which animal-intake datasets outrank unrelated ones,
   and obvious non-matches (e.g. a parking dataset) are dropped.
2. Each emitted candidate carries `{family, domain|base_url, dataset_id, score,
   matched_signals}` and a ready-to-run `probe` command string for that family.
3. `--json` emits valid JSON parseable into the candidate list; `--limit N` caps
   the output to N candidates; `--families` filters which catalogs are queried.
4. `--families opendatasoft` queries the ODS catalog discovery endpoint (assert
   against an ODS catalog fixture) and emits `opendatasoft`-family candidates with
   a `base_url`.
5. discover writes nothing to `deploy/sources.toml` and its output explicitly
   labels rows as unvalidated candidates requiring `probe`; a test asserts no file
   under `deploy/` is modified by a discover run.
6. The ArcGIS coverage limit (no federated catalog) is documented in `--help` and
   logged when `arcgis` is requested in `--families`, rather than silently
   returning empty.
7. `cargo test` passes for `homeward-connectors`; no new clippy warnings against
   the repo bar.
