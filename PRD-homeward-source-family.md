# PRD: homeward-source-family

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/homeward
**Vision:** visions/homeward.md (Reach fleet)

## TL;DR

`deploy/sources.toml` and the homeward-connectors registry loader understand
exactly one source family: Socrata (`[[socrata]]` table-arrays). A US city whose
animal-services dataset lives on an OpenDataSoft portal or an Esri ArcGIS Hub
Feature Service cannot be added at any configuration — there is no family for it.
This PRD grows the catalog format and the registry loader to parse
`[[opendatasoft]]` and `[[arcgis]]` entries alongside `[[socrata]]`, each into a
family-tagged config the registry can dispatch to the right connector. Existing
`[[socrata]]` entries must load byte-for-byte unchanged. This is the foundation
the two new connectors and the discovery subcommand all build on — no connector
can be *named* in the catalog until the loader has a slot for its family.

## Why this exists

Phase-1 live inspection of `~/wintermute/homeward` v0.24.0 (2026-06-14):

- `deploy/sources.toml` is `[[socrata]]` arrays only, with a `[socrata.column_map]`
  sub-table per entry (`animal_id`/`animal_type`/`intake_type` required). The
  header comment documents only the Socrata format.
- `homeward-connectors/src/registry.rs` is a `HashMap<String, Box<dyn Connector>>`
  with `register`/`get`/`names` — family-agnostic at the trait level, but nothing
  populates it from a multi-family file. The shipped `homeward-source-registry`
  work loads `SocrataConfig` from `[[socrata]]` only.
- `grep -rl 'OpenDataSoft|ArcGIS' homeward-connectors/src` is empty: the catchment
  fleet's parting note ("a second connector family — not just config — widens
  catchment past Socrata-only municipalities", visions/homeward.md Catchment
  "still un-dreamt") is unaddressed.

Until the catalog format can *name* a non-Socrata source, the OpenDataSoft and
ArcGIS connectors (sibling PRDs) have nowhere to be configured. This PRD is
deliberately small and is the hard dependency for the rest of the Reach fleet.

## What this builds

A family discriminator in the catalog + a dispatching loader, in
`homeward-connectors`.

### Catalog format

Extend `deploy/sources.toml` to additionally accept:

```toml
[[opendatasoft]]
name        = "example-ods"
base_url    = "https://data.example.org"   # ODS portal root
dataset_id  = "animal-shelter-intakes"
[opendatasoft.column_map]
animal_id   = "animal_id"
animal_type = "species"
intake_type = "intake_condition"
# … same optional slots as socrata.column_map

[[arcgis]]
name        = "example-arcgis"
service_url = "https://services.arcgis.com/XXXX/arcgis/rest/services/Intakes/FeatureServer/0"
[arcgis.column_map]
animal_id   = "AnimalID"
animal_type = "Species"
intake_type = "IntakeType"
```

### Modules / shape

- A `SourceFamily` enum (`Socrata | OpenDataSoft | ArcGis`) and a
  `SourceCatalog` deserialize target with three `#[serde(default)] Vec<_>` fields
  (`socrata`, `opendatasoft`, `arcgis`), so a file with only `[[socrata]]`
  deserializes with the other two empty.
- `OpenDataSoftConfig` / `ArcGisConfig` structs mirroring `SocrataConfig`'s owned-
  `String` shape, each with a `column_map`. (The actual `impl Connector` for these
  ships in the sibling connector PRDs; this PRD only defines the *config* types and
  the loader so the catalog can hold them. A config whose family has no connector
  yet must load and round-trip, and surface a clear "connector for family X not yet
  built" error only if something tries to instantiate it.)
- A `load_catalog(path) -> Result<Vec<(String, SourceFamily, FamilyConfig)>>` (or
  equivalent) that the registry-building code calls, dispatching each entry to its
  family. Socrata entries dispatch to the existing `SocrataConfig` path unchanged.
- Reuse the existing `ConnectorError` taxonomy; add one variant if a family is
  named but its connector is not yet registered.

### Constraints

- MSRV 1.85, no let-chains. Strict clippy is the repo bar (the workspace denies
  `unwrap`/`expect`/`panic` outside tests — use `?`/`Result`, no `unwrap` in
  non-test code).
- No network in this PRD — it is pure config parsing + dispatch. Tests are
  fixture-TOML round-trips.

## Acceptance criteria

1. `deploy/sources.toml` gains a documented `[[opendatasoft]]` and `[[arcgis]]`
   example entry (commented or live), and its header comment documents all three
   families' required/optional column-map slots.
2. A `SourceCatalog` type deserializes a TOML file containing any mix of
   `[[socrata]]`, `[[opendatasoft]]`, `[[arcgis]]` entries; absent sections
   default to empty vectors (no error).
3. **Back-compat:** the pre-existing all-`[[socrata]]` `deploy/sources.toml` loads
   into the same set of Socrata connectors as before this PRD — a regression test
   asserts the loaded Socrata source names/domains/dataset_ids are unchanged.
4. `OpenDataSoftConfig` and `ArcGisConfig` are defined with owned-`String` fields
   and a `column_map`, and round-trip through serde (deserialize→serialize→
   deserialize equality on a fixture).
5. Loading a catalog with an `[[opendatasoft]]` or `[[arcgis]]` entry succeeds at
   parse time even though those connectors are not built yet; attempting to
   *instantiate* a not-yet-built family returns a clear `ConnectorError` naming the
   family, never a panic.
6. `cargo test` passes for `homeward-connectors`; `cargo clippy` introduces no new
   warnings against the existing repo bar.
