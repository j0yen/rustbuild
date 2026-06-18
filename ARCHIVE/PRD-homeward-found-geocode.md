# PRD: homeward-found-geocode — give "found at..." a place on the map

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

For a stray, *where the animal was found* is often closer to the owner's
last-seen location than *where the shelter is* — but homeward throws that signal
away. `PetRecord.found_location_text` is a free-form string the connectors
populate and nothing ever parses (homeward-schema). Meanwhile match geo-filters
on `location: Option<ShelterLocation>`, which for a stray is the shelter, not
the find site. This PRD adds an **offline** geocoder that resolves
`found_location_text` to a coarse point and fills `location` when it is absent —
no external geocoding API (ToS + offline-first), coarsened to the same privacy
precision the rest of homeward uses.

## Why this exists

Phase-1 research (2026-06-17):

- `PetRecord` (homeward-schema lib.rs:113-206) has both
  `location: Option<ShelterLocation>` (coarse lat/lon, precision 2, geo.rs:11-57)
  and `found_location_text: Option<String>` — the latter "free-form found
  location description," parsed by nobody.
- The municipal STRAY feeds homeward prizes (Austin, Dallas, Sonoma, Long
  Beach…) frequently carry a `Found Location` column (the vision's Phase-1
  research cites this explicitly). The connectors map it into
  `found_location_text` and stop.
- The matcher (homeward-match) and the report API (`GET /intake` ZIP/state
  filters, `POST /search`) lean on `location` for geo relevance. A stray whose
  `location` is the shelter 30 km from where it was found is geo-mismatched
  against the owner's last-seen ZIP — exactly the case homeward exists to win.
- An external geocode API would add a network dependency, a ToS surface, and a
  rate limit to the ingest hot path. The US Census Gazetteer (places + ZCTAs)
  is public-domain, small, and resolves the common
  "city, ST" / "ZIP" / "NEIGHBORHOOD, city" forms offline.

## What this builds

Extends `homeward` (a `homeward-geocode` module, used by the connectors'
normalization step or by ingest at upsert time):

- A **bundled offline gazetteer**: a compact lookup built from the public-domain
  US Census Gazetteer (place names + ZCTA centroids → coarse lat/lon). Ship a
  trimmed, checked-in data file (or a build step that derives one); load into a
  normalized in-memory index at startup.
- A **parser/normalizer** for the common `found_location_text` shapes: ZIP
  (`78701`), "City, ST", "City ST", bare city within a known source state,
  and "<something>, City, ST". Tolerant, returns `None` on no confident match
  rather than guessing.
- A **resolver**: `found_location_text` → `ShelterLocation`-shaped coarse point,
  rounded to the existing privacy precision (±~1.1 km). Tag the provenance so
  it is distinguishable from a shelter-reported location (e.g.
  `location_source: FoundText | Shelter`).
- **Enrichment policy**: fill `PetRecord.location` from the found-text geocode
  ONLY when `location` is currently `None` (never overwrite a real shelter
  location); always record that the fill came from found-text. Applied at the
  normalization/ingest boundary, idempotent.

Deps: no network. A CSV/embedded data reader; reuse serde + existing crates.

## Acceptance criteria

1. An offline gazetteer loads from a checked-in public-domain data file with no
   network access (grep-asserted); load succeeds in a test from a fixture file.
2. The parser resolves a 5-digit ZIP to a coarse centroid; unit-tested for a
   known ZIP against an expected coarse cell.
3. The parser resolves "City, ST" and "City ST" forms; unit-tested for ≥3
   real city/state pairs from the existing source metros (Austin/Dallas/etc.).
4. Ambiguous or unrecognized text returns `None` (no guessing); tested with
   garbage input and an unknown city.
5. Resolved points are rounded to the existing privacy precision (±~1.1 km),
   asserted by checking the coordinate decimal places match `ShelterLocation`'s
   default precision.
6. Enrichment fills `location` only when it is `None`; a record with an
   existing `ShelterLocation` is left byte-identical (no overwrite), tested.
7. The fill is provenance-tagged so downstream can tell a found-text geocode
   from a shelter-reported location; asserted on an enriched record.
8. Enrichment is idempotent: running it twice yields the same record, tested.
9. A record enriched from found-text becomes geo-filterable by the existing
   `GET /intake` ZIP/state path (integration test or a documented manual check
   if the report binary is out of unit scope).
10. `cargo test` green; `cargo build` green for the workspace.

## Honesty notes

- Coarse-only and privacy-preserving: never store a precise find address; round
  to the same precision as every other homeward coordinate.
- Never overwrite a shelter-reported location; found-text geo is a *fallback*,
  always tagged as such, so a downstream consumer can weight it differently.
- Offline by construction: the gazetteer is bundled; no per-record network
  call, no third-party geocode ToS in the ingest path.
- The parser must prefer `None` over a low-confidence guess — a wrong point is
  worse than no point for an owner chasing a match.
