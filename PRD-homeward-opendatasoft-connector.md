# PRD: homeward-opendatasoft-connector

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/homeward
**Vision:** visions/homeward.md (Reach fleet)

## TL;DR

A large share of US municipal animal-services datasets are published not on
Socrata/SODA but on **OpenDataSoft** portals, which speak an entirely different
query dialect (the ODS Explore API v2.1, ODSQL `where` clauses). homeward has no
connector for them, so those cities are unreachable. This PRD adds an
`OpenDataSoftConnector` implementing the existing `Connector` trait against the
documented ODS Explore API, normalizing animal-intake records to `PetRecord`, and
teaches the `probe` subcommand to recognize and classify an ODS dataset. It is one
more `impl Connector` behind the same seam every other connector uses — nothing
downstream of the trait changes.

## Why this exists

Phase-1 live inspection (2026-06-14):

- `homeward-connectors/src/connectors/` holds `socrata.rs`, `rescuegroups.rs`,
  `petfbi.rs` — no OpenDataSoft. The vision's Catchment fleet named this exact gap
  as un-dreamt (visions/homeward.md: "OpenDataSoft + ArcGIS Open Data portals use
  a different query dialect than Socrata/SODA; a second connector family … widens
  catchment past Socrata-only municipalities").
- The `Connector` trait (`homeward-connectors/src/connector.rs`) is the proven
  extension seam: `async fn poll(&self, since: Option<Cursor>) -> Result<Vec<
  PetRecord>>`, `fn provenance(&self)`, `fn cadence_hint(&self)`. Socrata already
  implements it; a second family is additive, not invasive.
- `homeward-source-family` (sibling PRD, dependency) makes an `[[opendatasoft]]`
  entry nameable in `deploy/sources.toml`; this PRD makes such an entry *work*.
- The ODS Explore API v2.1 is a stable, documented, public, no-auth-for-public-
  datasets dialect distinct from SODA: records live at
  `/api/explore/v2.1/catalog/datasets/{dataset_id}/records`, filtered with an
  ODSQL `where=` clause, ordered with `order_by=`, paged with `limit`/`offset`,
  and carry a `record.timestamp` usable as an incremental watermark. This is a
  different query/response shape than SODA, which is precisely why config alone
  cannot cover it.

## What this builds

`homeward-connectors/src/connectors/opendatasoft.rs` + `probe` extension.

### The connector

- `OpenDataSoftConnector` built from the `OpenDataSoftConfig` defined in
  homeward-source-family (`base_url`, `dataset_id`, `column_map`, optional
  app-token env name for higher rate limits).
- `impl Connector`:
  - `poll(since)` issues a GET to the Explore v2.1 `/records` endpoint with an
    ODSQL `where` filtering to STRAY-bearing intake-type values (reuse the
    `STRAY_VALUES` set already in `probe.rs`) and, when `since` is
    `Cursor::Timestamp(t)`, `where … and {ts_field} > '{t}'`; pages through
    results; maps each record's fields through `column_map` into `PetRecord` with
    `Provenance` tagged `opendatasoft:{base_url}/{dataset_id}`.
  - Honor conditional/ETag semantics where ODS exposes them; on no-new-records
    return `Ok(vec![])`.
  - `cadence_hint` returns a sane default (e.g. the same order as Socrata's).
  - Returns the next `Cursor` watermark from the max `record.timestamp` seen.
- Reuse `homeward-connectors/src/http.rs` `PoliteClient` (rate-limited, polite
  UA) — do not hand-roll a `reqwest::Client`.
- Register `OpenDataSoftConfig → Box<dyn Connector>` construction so the
  family-dispatching loader from homeward-source-family can instantiate it.

### Probe extension

- Extend the `probe` subcommand (`homeward-connectors/src/probe.rs`) to accept an
  ODS target (e.g. `probe --family opendatasoft <base_url> <dataset_id>`), fetch
  the ODS dataset's field metadata + a one-record sample via the Explore v2.1
  metadata/records endpoints, classify columns into the column-map slots using the
  same heuristics, and emit a GREEN `[[opendatasoft]]` TOML block or an honest RED
  verdict — mirroring the existing Socrata probe output.

### Constraints

- MSRV 1.85, no let-chains. Strict clippy bar (no `unwrap`/`expect`/`panic`
  outside tests).
- Network calls in tests must be against recorded fixtures / a mock, not the live
  ODS API (deterministic). Ship at least one real captured ODS `/records` JSON
  sample as a fixture and assert the normalization against it.
- Do **not** fabricate specific live dataset ids in committed config — real ids
  come from `probe`/`discover`. Fixtures may use a captured anonymized sample.

## Acceptance criteria

1. `OpenDataSoftConnector` implements `Connector` and, given a fixture ODS Explore
   v2.1 `/records` JSON response, `poll(None)` returns the expected `Vec<PetRecord>`
   with fields correctly mapped through `column_map` and provenance tagged with the
   ODS family + dataset id.
2. `poll(Some(Cursor::Timestamp(t)))` constructs an ODSQL `where` clause that
   filters to records newer than `t` (assert the request URL/query a mock receives);
   an empty result returns `Ok(vec![])`, not an error.
3. Only STRAY-bearing intake-type records survive normalization (a fixture mixing
   STRAY and ADOPTABLE rows yields only the STRAY rows), reusing the shared
   `STRAY_VALUES` set.
4. The connector uses the shared `PoliteClient` from `http.rs` (no direct
   `reqwest::Client::new` in the connector).
5. `probe --family opendatasoft <base_url> <dataset_id>` against a fixture emits a
   valid `[[opendatasoft]]` TOML block (GREEN) for a STRAY-bearing dataset and a
   RED verdict for a dataset with no intake-type column.
6. An `[[opendatasoft]]` entry in `deploy/sources.toml` loads (via
   homeward-source-family) and instantiates a working `OpenDataSoftConnector`.
7. `cargo test` passes for `homeward-connectors`; no new clippy warnings against
   the repo bar.
