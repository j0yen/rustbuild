# PRD: homeward-catchment-geo — coverage as a map of geography, not a list of names

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

homeward already knows *which sources* it ingests, but it does not know *what
geography* those sources actually cover. Today a "coverage hole" is hand-typed:
`coverage.rs` maps a source *name* to a metro label (`metro_from_name`,
coverage.rs:374-391) and `catalog_documented_gaps()` (coverage.rs:397-416)
lists LA/NY/Chicago/Houston/Phoenix as known-missing by hand. Meanwhile every
`PetRecord` already carries a coarse `ShelterLocation { lat, lon, precision }`
(homeward-schema geo.rs:11-57) that nobody rolls up. This PRD computes
coverage from those real points: bin the ingested population into geographic
cells, so a coverage hole becomes *a populated region with no feed* — derived
from data, not a hardcoded list. Offline only; no PostGIS server (this is a
CPU-only laptop).

## Why this exists

Phase-1 research (2026-06-17) mapped the built homeward (v0.29.0, 22 PRDs
shipped) and found the coverage story is name-derived, not geo-derived:

- `coverage.rs:374-391` `metro_from_name()` is a hardcoded `&str → label`
  table ("austin" → "Austin, TX", "sonoma" → "Sonoma County, CA"). A new
  source with an unrecognized name gets no metro; coverage cannot grow without
  editing this table.
- `coverage.rs:397-416` `catalog_documented_gaps()` returns a hand-written list
  of unserved metros. It cannot notice a *new* hole, and it cannot tell whether
  a "covered" metro is actually getting records there.
- The data to do better is already present: `PetRecord.location:
  Option<ShelterLocation>` with coarse `lat`/`lon` (precision 2, ~±1.1 km;
  geo.rs:11-57). The ingest store (homeward-ingest, 174K+ animals) holds them.

The vision's wedge against the closed incumbent is *open, auditable coverage*.
"Here is a computed map of where we do and don't see shelter animals, with
counts" is exactly that wedge — and the un-dreamt frontier item this vision
flagged ("coverage holes are a *map of metros*, not a list of sources").

## What this builds

Extends `homeward` (new module in `homeward-connectors`, where `coverage.rs`
already lives; or a small `homeward-geo` submodule it re-exports):

- A **geographic cell model**: quantize coarse `(lat, lon)` into stable cells.
  Use a simple equal-angle grid (e.g. 0.5° lat/lon buckets) — deterministic,
  offline, no spatial DB, no external crate beyond what is already vendored.
  Each cell carries `{ cell_id, center_lat, center_lon, record_count,
  stray_count, distinct_sources, last_seen }`.
- A **coverage rollup** over the ingest store: stream all `PetRecord`s with a
  `location`, accumulate per-cell counts and the set of contributing sources.
- A **hole heuristic**: a cell (or contiguous metro region) is a *hole* when a
  known population signal exists (a neighboring high-count cell, or an entry in
  the existing documented-gaps list as a seed) but `distinct_sources == 0` or
  `record_count` is far below neighbors. Emit holes ranked by estimated missed
  population, not by hand-ordering.
- An extended `coverage` report: keep the current name/metro JSON for
  backward compatibility (AC-gated byte-stability of the existing fields), and
  ADD a `cells: [...]` and `geo_holes: [...]` section.
- A subcommand surface: `homeward-connectors coverage --geo [--min-count N]
  [--format json|table]` reading the ingest SQLite store read-only.

Deps: reuse existing workspace deps (serde, rusqlite/the store accessor,
chrono). No PostGIS, no network, no new heavy crate.

## Acceptance criteria

1. A geographic cell type exists with a deterministic `cell_id` derived purely
   from a coarse `(lat, lon)`; the same point always maps to the same cell, and
   a unit test asserts two points ~1 km apart in a dense metro share or
   neighbor cells as designed.
2. A rollup function consumes an iterator of `PetRecord`s (or reads the ingest
   store read-only) and produces per-cell `{record_count, stray_count,
   distinct_sources, last_seen}` correctly; verified on a fixture of records
   spanning ≥3 cells with mixed `IntakeType`.
3. `stray_count` counts only `IntakeType::Stray` (and `FoundReport`) records,
   proven distinct from `record_count` on a mixed fixture.
4. The hole heuristic flags a populated-but-unsourced region and does NOT flag
   a well-covered region; tested on a fixture with one genuine hole and one
   covered cell, asserting exactly the hole is returned.
5. Holes are returned ranked by estimated missed population (descending),
   asserted by a fixture with two holes of different magnitude.
6. The existing `coverage` JSON report's pre-existing fields remain
   byte-identical for an unchanged input (no regression to name/metro
   coverage); the geo data is purely additive under new keys.
7. `coverage --geo` runs against the real ingest SQLite store read-only
   (opens with a read-only connection; never writes), and emits valid JSON.
8. Records with `location == None` are counted in an `ungeocoded` tally rather
   than silently dropped, and that tally appears in the report.
9. No network access and no PostGIS dependency anywhere in the new code
   (grep-asserted in the test/CI notes); fully offline.
10. `cargo test` green for the new module; `cargo build` green for the
    workspace.

## Honesty notes

- Coarse-only: cells are built from already-coarsened (±~1.1 km) points; this
  PRD never increases location precision.
- The hole heuristic is an *estimate* and must be labeled as such in output
  (a `confidence` or `basis` field), never asserted as ground-truth coverage.
- Seed the population signal from the existing documented-gaps list so reach-3
  composes with prior work rather than discarding it.
