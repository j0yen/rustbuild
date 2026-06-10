# PRD: homeward-federation-petfbi — pull lost/found reports IN from the one open network

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md
Depends: none (extends shipped homeward-connectors v0.8.0; implements the existing
  `Connector` trait at homeward-connectors/src/connector.rs:31, registers like the
  shipped RescueGroups/Socrata connectors).

## TL;DR

homeward today ingests *shelter* data (RescueGroups national JSON + municipal Socrata
STRAY feeds). It sees nothing of the *community* lost/found population — the reports
owners file themselves when a pet goes missing or a neighbor finds a stray. The
homeward vision's federation open question asked how to reach those networks. A
federation research pass (2026-06-10, four channels probed) found that of every
lost-pet network, **exactly one has an open machine-readable feed: Pet FBI's report
widget data feed**, and a single pull from it covers **Pet FBI + Helping Lost Pets
(HeLP) + the Lost Dogs of America / Lost Cats of America** networks, because those
share one backing database via a formal data partnership. This PRD adds a
`PetFbiConnector` that pulls that feed, normalizes its lost/found/sighting reports to
homeward's `LostReport` / `PetRecord`, and stamps honest federated provenance — the
single highest-leverage, lowest-friction outward integration available.

## Why this exists

Phase-1 evidence (federation research 2026-06-10, citations below; direct grep of the
shipped homeward workspace):

- **The community-report population is invisible to homeward today.** The shipped
  connectors (`homeward-connectors/src/connectors/{rescuegroups,socrata}.rs`,
  registered in `main.rs:23-43`) cover adoptable + municipal stray *shelter* intakes
  only. An owner's "I lost my dog" report and a finder's "I found this cat" report
  live on community networks homeward never reads.
- **Pet FBI is the only open machine surface.** Research verdict: PawBoost has no
  third-party API (shelter-inbound only, built on ShelterLuv); Nextdoor's write path
  is partnership-gated with no lost-pet endpoint; Facebook Groups API was deprecated
  by Meta in April 2024; Petco Love Lost is closed/partner-only. Pet FBI exposes a
  **report widget feed**: request a `data-file` UUID from "Pet FBI Central," then
  fetch live lost/found/sighting reports filterable by type/species/source over its
  `/api/` paths. (Source: https://petfbi.org/info-for-shelters/pet-fbi-org-report-widget/)
- **One connector, four networks.** Pet FBI and Helping Lost Pets auto-populate **both**
  databases from a single filing (formal partnership), and Lost Dogs of America /
  Lost Cats of America are Facebook-amplification layers riding the **HeLP** store —
  not separate data sources. So this one feed federates four networks at once.
  (Source: https://lostdogsofamerica.org/pet-fbi-and-helping-lost-pets-launch-nationwide-lost-and-found-pet-database-collaboration/)
- **The provenance model already has a slot for it.** `homeward-schema/src/provenance.rs`
  ships `Provenance`, `SourceId`, and `TosClass` (re-exported at lib.rs:19). A federated
  source is a new `SourceId` with its own `TosClass`; no schema change beyond adding the
  variant. `LostReport` (`homeward-schema/src/lost.rs:88`) and its `LostStatus`
  (`lost.rs:74`) are the normalization target for owner reports; finder/stray reports
  normalize to `PetRecord` (`schema/src/lib.rs:113`) exactly as shelter intakes do.
- **The polite-client + paging plumbing already exists.** `homeward-connectors/src/http.rs`
  ships `PoliteClient` (conditional requests, rate-courtesy) and the RescueGroups
  connector already does JSON paging — this connector reuses both rather than inventing
  HTTP handling.

## What this builds

A new connector module `homeward-connectors/src/connectors/petfbi.rs` plus a registry
entry, in the shape of the shipped connectors:

- **`PetFbiConfig`** — `data_file` UUID (required), optional species/type/source filters,
  optional geographic bbox if the feed supports it; `from_env()` reading
  `HOMEWARD_PETFBI_DATA_FILE`. The UUID is requested once from Pet FBI Central and
  treated as a credential (config, not hardcoded).
- **`PetFbiConnector`** implementing `Connector` (`connector.rs:31`): fetches the feed
  via `PoliteClient`, deserializes the report records, maps each to either a
  `LostReport` (type=lost) or a `PetRecord` (type=found/sighting/stray) with
  `LostStatus` / status set accordingly, and stamps `Provenance { source: SourceId::PetFbi
  | SourceId::HelpingLostPets, … }` reflecting the report's own `source` field so HeLP-
  origin and Pet-FBI-origin records are distinguishable.
- **New `SourceId` + `TosClass` variants** in `homeward-schema/src/provenance.rs`:
  `SourceId::PetFbi`, `SourceId::HelpingLostPets`; a `TosClass` reflecting the nonprofit
  feed's terms (attribution-required, refresh-respecting). This is the only schema touch.
- **Registry wiring** in `homeward-connectors/src/main.rs`: register the connector when
  `HOMEWARD_PETFBI_DATA_FILE` is set, mirroring the RescueGroups `from_env` gate at
  `main.rs:37-43` (no UUID ⇒ connector silently absent, never a hard error).
- **ToS compliance**: conditional requests via `PoliteClient`, no image rehosting
  (hotlink upstream photo URLs, matching the connectors' existing image policy),
  attribution carried in provenance, refresh cadence respecting the feed.

Out of scope (documented-not-built, per the research verdict — these have no open
machine interface and would be fiction as connectors): PawBoost, Nextdoor, Petco Love
Lost, Facebook Groups, microchip registries. They are recorded in the vision's updated
Federation section as partnership-asks, not PRDs.

## Acceptance criteria

1. `homeward-connectors` builds clean (`cargo build -p homeward-connectors`) and the
   workspace `cargo test` is green, including new unit tests for the petfbi mapping.
2. `PetFbiConnector` implements `Connector`; a fixture feed payload (committed under
   `tests/fixtures/petfbi_*.json`, captured from the documented widget response shape)
   maps to the expected mix of `LostReport` and `PetRecord` values — asserted field by
   field (species, status, location, photo URL, source).
3. A `type=lost` report normalizes to a `LostReport` with the correct `LostStatus`;
   a `type=found` / stray report normalizes to a `PetRecord`. The mapping is covered by
   at least one test per branch.
4. Provenance is honest: a HeLP-origin record carries `SourceId::HelpingLostPets` and a
   Pet-FBI-origin record carries `SourceId::PetFbi`, derived from the feed's own `source`
   field, not hardcoded to one value.
5. `homeward-schema` gains `SourceId::PetFbi`, `SourceId::HelpingLostPets`, and a
   matching `TosClass`; existing schema tests still pass and the new variants serialize
   round-trip.
6. The connector is registered only when `HOMEWARD_PETFBI_DATA_FILE` is set; with the
   env unset, `homeward connectors` runs exactly as before (no panic, no empty-UUID
   request). Asserted by a CLI smoke test.
7. No image bytes are rehosted: the connector stores upstream photo URLs by reference,
   matching the existing connectors' hotlink policy (grep-assertable — no image
   download in this module).
8. `clippy -D warnings` and `cargo deny check bans licenses sources` pass for the
   crate (workspace baseline; no new advisory-flagged deps — `async`/`reqwest`/`serde`
   are already in-tree).

## Notes for /build

- Model the module on `homeward-connectors/src/connectors/rescuegroups.rs` — same
  `Connector` impl shape, same `PoliteClient` use, same `from_env` registration gate.
- The exact field names of the widget feed JSON should be pinned from a real captured
  response before finalizing the serde structs; until then the fixture defines the
  contract and the structs match the fixture. If the live shape differs, it's a fixture
  update, not a redesign.
- This is pull-IN only. The outbound (post-OUT) syndication half is a separate PRD
  (homeward-federation-export) because every post-OUT channel is partnership-gated and
  must not ship as a fictional transport.
