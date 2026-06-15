# PRD: homeward-arcgis-connector

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/homeward
**Vision:** visions/homeward.md (Reach fleet)

## TL;DR

Many US shelters and animal-services departments publish their intake data through
**Esri ArcGIS Hub** as a REST Feature Service, which speaks yet another dialect:
`/FeatureServer/{layer}/query?where=…&outFields=*&f=geojson`. homeward has no
ArcGIS connector, so every Esri-published city is unreachable. This PRD adds an
`ArcGisConnector` implementing the existing `Connector` trait against the
documented ArcGIS Feature Service query API, normalizing intake features to
`PetRecord`, and teaches `probe` to recognize an ArcGIS Feature Service endpoint.
Third family behind the same trait seam; no downstream change.

## Why this exists

Phase-1 live inspection (2026-06-14):

- `homeward-connectors/src/connectors/` has Socrata, RescueGroups, PetFbi — no
  ArcGIS. The Catchment fleet named "ArcGIS Open Data portals use a different query
  dialect … a second connector family widens catchment past Socrata-only
  municipalities" as un-dreamt (visions/homeward.md).
- The `Connector` trait (`connector.rs`) absorbs a third family with no change to
  ingest/dedup/match downstream — same `poll → Vec<PetRecord>` contract Socrata and
  the ODS sibling use.
- `homeward-source-family` (dependency) makes an `[[arcgis]]` entry nameable in
  `deploy/sources.toml`; this PRD makes it work. Independent of the ODS connector —
  the two are parallel branches off the foundation.
- The ArcGIS REST Feature Service query API is a stable, documented, public dialect
  distinct from both SODA and ODS: features are queried at
  `{service_url}/query` with a SQL-ish `where=` clause, `outFields=*`, `f=geojson`
  (or `f=json`), and paged with `resultOffset`/`resultRecordCount`; an `EditDate`
  or `last_edited_date` field (when present) gives the incremental watermark via
  `where EditDate > {epoch_ms}`. Different request and response shape than SODA/ODS,
  which is why a config flag cannot cover it.

## What this builds

`homeward-connectors/src/connectors/arcgis.rs` + `probe` extension.

### The connector

- `ArcGisConnector` built from the `ArcGisConfig` defined in homeward-source-family
  (`service_url` ending in `/FeatureServer/{layer}`, `column_map`, optional layer
  index).
- `impl Connector`:
  - `poll(since)` GETs `{service_url}/query` with `where` filtering to STRAY-
    bearing intake-type values (reuse `STRAY_VALUES`), `outFields=*`,
    `f=geojson`; when `since` is `Cursor::Timestamp(t)` adds
    `EditDate > {t_as_epoch_ms}` (ArcGIS edit timestamps are epoch-ms); pages via
    `resultOffset` until `exceededTransferLimit` is false / fewer than the page
    size return; maps `feature.properties` through `column_map` into `PetRecord`
    with `Provenance` tagged `arcgis:{service_url}`.
  - On no new features return `Ok(vec![])`.
  - `cadence_hint` returns a sane default.
  - Returns the next `Cursor` watermark from the max edit timestamp seen.
- Reuse `homeward-connectors/src/http.rs` `PoliteClient`.
- Register `ArcGisConfig → Box<dyn Connector>` for the family-dispatching loader.

### Probe extension

- Extend `probe` to accept an ArcGIS target (e.g.
  `probe --family arcgis <service_url>`), fetch the layer metadata
  (`{service_url}?f=json` → field list) and a one-feature sample
  (`{service_url}/query?where=1=1&outFields=*&resultRecordCount=1&f=geojson`),
  classify fields into column-map slots, and emit a GREEN `[[arcgis]]` TOML block
  or an honest RED verdict — mirroring the Socrata/ODS probe output.

### Constraints

- MSRV 1.85, no let-chains. Strict clippy bar (no `unwrap`/`expect`/`panic`
  outside tests).
- Tests run against recorded GeoJSON/JSON fixtures or a mock, never the live
  ArcGIS API. Ship a captured Feature Service `query` GeoJSON sample as a fixture.
- No fabricated live `service_url`s in committed config — real endpoints come from
  `probe`/`discover`.

## Acceptance criteria

1. `ArcGisConnector` implements `Connector` and, given a fixture Feature Service
   `query` GeoJSON response, `poll(None)` returns the expected `Vec<PetRecord>`
   with `feature.properties` mapped through `column_map` and provenance tagged with
   the ArcGIS family + service url.
2. `poll(Some(Cursor::Timestamp(t)))` constructs a `where` clause comparing the
   edit-timestamp field to `t` as epoch-ms (assert the request query a mock
   receives); an empty result returns `Ok(vec![])`.
3. Paging is correct: a fixture that reports `exceededTransferLimit: true` causes a
   second `query` request at the next `resultOffset`, and the records from both
   pages are concatenated.
4. Only STRAY-bearing intake records survive normalization (mixed fixture →
   STRAY-only), reusing `STRAY_VALUES`.
5. The connector uses the shared `PoliteClient` (no direct `reqwest::Client::new`).
6. `probe --family arcgis <service_url>` against a fixture emits a valid
   `[[arcgis]]` TOML block (GREEN) for a STRAY-bearing layer and a RED verdict for
   a layer with no intake-type field.
7. An `[[arcgis]]` entry in `deploy/sources.toml` loads (via homeward-source-family)
   and instantiates a working `ArcGisConnector`.
8. `cargo test` passes for `homeward-connectors`; no new clippy warnings.
