# Vision: homeward — the open layer that brings lost pets home

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-04
**Status:** active
**Seed:** jsy — *"a real-time database of missing dogs... research deeply for all
public information on all dogs being hosted in shelters... reconnect them with
their owners. We will need ML to match dogs by photos."* Extended same session:
*"also include cats."* Grounded in deep external research (four parallel
research agents, 2026-06-04; citations throughout the fleet PRDs).
**Note:** This is the **first outward-facing vision** in the autobuilder — gossip
(2026-06-xx) flagged that "ZERO of 27 visions reaches outward." homeward reaches
outward by design.

## TL;DR

When a dog or cat goes missing, the animal is very often *already in a shelter* —
picked up as a stray and logged within hours — but the owner never finds out in
time. The data exists publicly and the matching is tractable, yet the owner has
to manually check 5–8 sites and squint at photos. homeward aggregates, in
near-real-time, the companion animals currently held in US shelters and rescues
(prioritizing **stray/found intakes** — the someone's-lost-pet population),
embeds every photo with an open vision model, and lets an owner submit one photo
of their lost pet to get a ranked shortlist of visually-similar animals in
shelters near them — with a match alert the moment a new matching intake appears.
The wedge versus the incumbent (Petco Love Lost, which already does national AI
matching but as a closed walled garden) is to be **open**: an open API, open
data, federation of sources the incumbent doesn't ingest, owner-controlled
records, real-time delta freshness, and an auditable, open-source matcher with
published accuracy.

## Why now (Phase 1 research, 2026-06-04)

- **The canonical aggregation API died.** Petfinder's public API was **retired
  Dec 2, 2025**. The remaining free national JSON source is **RescueGroups.org**
  (JSON:API v5, free key, ToS explicitly permits cached derivative search
  products if refreshed ≥weekly and org-deletion honored in 1 business day).
- **The stray gold mine is municipal open data.** Socrata/SODA + OpenDataSoft
  portals (Austin `fdzn-9yqv`, Dallas `qgg6-h4bd`, Sonoma `924a-vesw`, Long
  Beach, Bloomington, and *hundreds* more) uniquely tag `Intake Type = STRAY`,
  often with `Found Location`, `Chip_Status` ("SCAN NO CHIP"), kennel/current
  status, and `:updated_at` for delta polling — free, no auth. These carry the
  records most likely to be someone's lost pet, which the adoptable-only feeds
  (RescueGroups, Adopt-a-Pet, Petango) do not isolate.
- **The incumbent is closed.** Petco Love Lost runs national facial-recognition
  matching (the former Finding Rover engine) across ~3,000+ shelters, but exposes
  **no public/owner query API**, no open data, a black-box model with no
  published accuracy, US-only, and admits its intake feeds are "very inconsistent"
  (it shows *adoptable* pets to compensate). Open + auditable + broader
  federation is genuine white space.
- **The ML is feasible on modest hardware.** A v1 of YOLO body-crop → **DINOv2
  ViT-B** (Apache-2.0, commercially usable) → cosine kNN over an HNSW index
  narrows tens of thousands of intake photos to a human-reviewable shortlist;
  query latency is dominated by one forward pass (<1s). The accuracy ceiling
  (ArcFace on PetFace: 99% verification AUC vs CLIP's 91.9%) motivates a v2
  fine-tune. **PetFace** (ECCV 2024) provides 46,755 dog individuals + cat
  individuals with identity labels — the dataset that makes both training and
  *honest held-out eval* possible. (Honest eval is non-negotiable here — see
  [[feedback_agent_written_fixtures_tautology]]: validate on truly held-out
  individuals, never self-generated fixtures.)
- **We already have the index pattern.** `~/wintermute/recall` ships a BGE
  embedder + vector index; homeward's photo index is the same shape with an image
  embedder instead of text. Not a from-scratch invention.

## End-state

When homeward ships:

1. **One normalized record** describes every sheltered pet — dog or cat —
   regardless of which of a dozen heterogeneous sources it came from, with
   honest provenance and a `stray | found | surrender | adoptable` status.
2. **Near-real-time aggregation** keeps that store fresh within the bound of the
   slowest upstream (minutes-to-hours), via conditional-request delta polling,
   per-source adaptive cadence, cross-source dedup, and departure detection so
   reclaimed/adopted animals expire instead of haunting a searching owner.
3. **Every photo is embedded** with an open vision model the moment it is
   ingested, indexed for sub-second similarity search.
4. **An owner submits one photo** of their lost dog or cat (+ coarse last-seen
   location, brokered contact) and gets a ranked shortlist of candidate matches
   in nearby shelters — framed as *possibilities for human confirmation*, never
   "we found your pet."
5. **A match alert fires** when a new intake scores above threshold against an
   open lost report, within minutes of that intake appearing.
6. **It is open:** a documented public query API and opt-in open-data export,
   an auditable open-source matcher with published held-out accuracy — the
   interoperability layer the incumbents are not.

## Components (one bullet per PRD)

- **homeward-schema** — the canonical `PetRecord` + `LostReport` types and source
  provenance model that every connector and the matcher share.
- **homeward-connectors** — the source-connector framework + first connectors
  (RescueGroups.org national JSON; municipal Socrata STRAY feeds), normalizing to
  `PetRecord`, ToS-compliant (conditional requests, image hotlinking, deletion).
- **homeward-ingest** — the freshness engine: adaptive-cadence orchestration,
  cross-source dedup (stable source-id + perceptual hash), departure detection,
  canonical store.
- **homeward-embed** — the ML photo pipeline (Python): YOLO dog/cat body-crop →
  DINOv2 embedding → vector index, with an honest held-out eval harness.
- **homeward-match** — combine visual kNN with structured filters (species,
  coarse geo, intake-date window, size/color) into a calibrated, candidate-not-
  confirmation ranked shortlist for a lost report.
- **homeward-report** — owner side: submit a lost report (EXIF-stripped,
  location-coarsened, brokered contact), continuous matching + match alerts,
  auto-expiry, and the open read API.

## Order

```
homeward-schema ─► homeward-connectors ─► homeward-ingest ─► homeward-embed ─► homeward-match ─► homeward-report
                                                  └────────────────┘ (embed consumes ingested photos)
```

schema is the foundation. connectors and ingest build the fresh store. embed
indexes the photos that store holds. match reads embed + schema. report sits on
top, owner-facing, and exposes the open API.

## Open questions

- **Repo home & language split.** Proposed new cargo workspace
  `~/wintermute/homeward/` with Rust crates for schema/connectors/ingest/match/
  report and a Python subtree (`homeward/embed/`) for the vision model. Is a
  Python+Rust split acceptable, or should embedding run as a sidecar service the
  Rust side calls over a socket? (Leaning: Python embed service + Rust calls it.)
- **Commercial vs research ML.** DINOv2/OpenCLIP are Apache/permissive
  (commercial-OK) but lower accuracy; MegaDescriptor and PetFace-trained weights
  are **non-commercial / research-gated**. v1 must use the permissive path; a v2
  fine-tune on PetFace would be research-licensed — does this stay non-commercial
  / nonprofit? That choice gates which weights are legal to ship.
- **Outward federation (future fleet).** The biggest practical gap is "one report,
  every channel" — syndicating a lost report out to PawBoost (843 FB pages, 50M
  reach), Pet FBI / Lost Dogs of America (nonprofit, open ethos, best first
  partner), Nextdoor — and pulling their matches back. Not yet thought through;
  left for `/dream extend homeward`.
- **Microchip federation.** AAHA's universal lookup is human-gated, routing-only,
  and missing AVID. A programmatic chip→registry layer needs partnerships, not
  scraping — a separate honest investigation.
- **Match-confidence calibration & the false-match-distress guardrail** — the UX
  contract that results are candidates for human review, never confirmations.
- **Stray-hold awareness** — never surface a stray as "adoptable" during its
  legal hold window (3–10 days, per-state); surface it on the found/lost side so
  the owner can reclaim it.

## Federation fleet (drafted 2026-06-10)

Fleet 1 shipped the core: the workspace exists (`~/wintermute/homeward/`, v0.9.0) with
all six crates built — schema, connectors (RescueGroups + Socrata STRAY), ingest
(dedup + departure), embed (Python DINOv2 sidecar + Rust client), match, report. The
"outward federation" open question above ("one report, every channel… not yet thought
through; left for `/dream extend homeward`") is the un-built frontier this fleet opens.

A federation research pass (2026-06-10, four lost-pet networks + microchip + Facebook +
interchange standards probed; citations in the PRDs) found the honest reality: **of every
lost-pet network, only Pet FBI's report widget feed has an open machine interface.** A
single pull from it federates four networks (Pet FBI + Helping Lost Pets + Lost Dogs of
America + Lost Cats of America — they share one backing database). Everything else is
gated or dead:

| Channel | Verdict | Disposition |
|---|---|---|
| Pet FBI / HeLP read feed | BUILDABLE-NOW (pull-IN) | **PRD: homeward-federation-petfbi** |
| Pet FBI / HeLP write | partnership-gated (open-ethos nonprofit; outreach) | gated adapter in **homeward-federation-export**, dry-run until endpoint exists |
| PawBoost | no third-party API (shelter-inbound only) | manual-only; no PRD |
| Nextdoor | partnership-approval-gated; no lost-pet write API | manual-only; no PRD |
| Facebook Groups | **API deprecated by Meta, April 2024** | dead; no PRD |
| Petco Love Lost / 24PetWatch | closed / shelter-software-gated | no PRD |
| Microchip (AAHA universal lookup, AKC/AVID/24PW) | HUMAN-ONLY (CAPTCHA web form, bots 403'd) or FTP/SMS-gated | separate honest investigation; no PRD |
| Open lost-pet interchange standard | none exists (schema.org has no lost/found type) | homeward defines its own JSON-LD in **homeward-federation-export** |

### Federation components (drafted this pass)

- **homeward-federation-petfbi** (rust-extend → homeward-connectors) — a `PetFbiConnector`
  pulling the Pet FBI/HeLP widget feed IN, normalizing lost/found/sighting reports to
  `LostReport`/`PetRecord` with honest federated provenance. One connector, four networks.
- **homeward-federation-dedup** (rust-extend → homeward-ingest) — reconcile federated
  community found-reports against shelter stray-intakes so one animal isn't double-listed;
  merge-with-both-provenances, thresholds biased toward false-split. Depends on petfbi.
- **homeward-federation-export** (rust-extend → homeward-report) — the outbound half, done
  honestly: a portable JSON-LD `LostReportExport` (no open standard exists, so homeward
  defines one) + a human-postable flyer + a `Syndicator` trait whose only machine target
  is a **dry-run-by-default** Pet FBI partner adapter. Gated channels are `ManualOnly`,
  never fictional transports. Independent of the pull-IN pair.

### Federation order

```
homeward-federation-petfbi  ─►  homeward-federation-dedup
homeward-federation-export  (independent — outbound, parallel)
```

### Still un-dreamt / partnership-asks (not PRDs)

- A Pet FBI/HeLP **partner write** agreement (email + credential) flips the export
  adapter from dry-run to live with no code change — an outreach, not a build.
- Nextdoor partner-API access; any Facebook Pages (not Groups) deal — both partnership,
  both off the critical path.
- Microchip federation remains "a separate honest investigation" (no open API anywhere;
  needs registry partnerships, per the original open question above).

## Operate fleet (drafted 2026-06-13)

Fleets 1 (core) and 2 (federation) shipped the *parts*: `~/wintermute/homeward`
v0.9.3 is six crates + a Python embed sidecar that compile, ship binaries
(`homeward-ingestd run`, `homeward-reportd serve`, `homeward-connectors`,
`homeward-match`, `homeward-embed-svc`), and have **never been run together**.
Phase-1 live inspection (2026-06-13) found the honest frontier is not a missing
feature — it is that homeward is a **library fleet that has never been operated,
provisioned, proven, or made to deliver**:

- **No deployment.** `find` for any compose/`.service`/Dockerfile in the repo hit
  only `.venv`. Three daemons meant to talk to each other have nothing standing
  them up, wiring their shared env, supervising them, or health-checking them.
- **The model is never provisioned.** `embedder.py:74` loads DINOv2 lazily via
  `from_pretrained` (silent download, no offline mode, no warmup); no proof it has
  ever embedded a real photo end-to-end with a measured latency. Same shape as the
  `wm-stt` stub→whisper gap ([[project_voice_input_null_detectors]]).
- **The eval never ran.** `eval.py` is a complete held-out harness that published
  no number and enforces its own disjoint-split warning only in prose. Honest
  accuracy is the vision's non-negotiable ([[feedback_agent_written_fixtures_tautology]]).
- **Alerts never deliver.** `alerts.rs` generates+dedups `MatchAlert`s with a
  brokered `contact_token` but has no transport — end-state #5 ("alert fires within
  minutes") is an object in a store, not a notification an owner receives.

### Operate components (drafted this pass)

- **homeward-orchestrate** (shell) — systemd-user units + a `homeward up/down/
  status/health` wrapper + an env contract that stand the three daemons up as one
  supervised `homeward.target`. Independent; the foundation for running anything.
- **homeward-embed-provision** (mixed → homeward) — deterministic DINOv2 prefetch
  + offline mode + a real enroll→query smoke on a bundled CC-licensed fixture that
  proves the index discriminates and records measured latency. Independent.
- **homeward-eval-harness** (mixed → homeward) — make `eval.py` runnable
  (`homeward-embed eval`), enforce gallery/query individual-disjointness *in code*
  (raise, not warn), ship a correctness fixture that proves the harness arithmetic,
  and commit `EVAL.md` documenting the real number + the PetFace manual path.
  Independent of orchestrate; benefits from embed-provision but builds standalone.
- **homeward-alert-delivery** (rust-extend → homeward-report) — a `Deliverer`
  trait (dry-run default, honest like the `Syndicator`), an email-relay adapter
  keyed by the brokered token (disabled→dry-run until a relay credential exists),
  an append-only delivery ledger, and the wiring that fires delivery on alert
  generation. Independent.

### Operate order

```
homeward-orchestrate     (foundation — stands the fleet up)
homeward-embed-provision ─► homeward-eval-harness   (eval benefits from a warmed model)
homeward-alert-delivery  (independent — rust-extend homeward-report)
```

All four are independent enough to ship in any order; the only soft edge is
eval-harness reusing the warmed model from embed-provision. orchestrate is
shell-only; embed-provision/eval-harness are mixed Python; alert-delivery is the
sole `rust-extend` of `homeward-report`.

### Still un-dreamt after operate

- A live PetFace held-out number (research-gated dataset = manual download, not a
  build) — the harness will be ready; obtaining the data is an outreach.
- A real relay credential to flip alert-delivery from dry-run to live (outreach).
- Public/remote exposure of the report API (a deliberate, gated decision; localhost
  by default until then).

## Deliver fleet (drafted 2026-06-13)

Fleets 1 (core), 2 (federation), and 3 (operate) shipped the parts and stood the
daemons up. Phase-1 live inspection (2026-06-13) of `~/wintermute/homeward` v0.9.x
found the honest frontier that operate left: **the embedding sidecar is never
actually called by any daemon.** homeward owns a complete async embed client and a
complete fusion matcher, but the wires between them are stubs:

- `EmbedClient` (`homeward-ingest/src/embed_client.rs`) — a full `/enroll`,
  `/query`, `/health` client for the Python DINOv2 sidecar — is **defined and
  never called.** Grep across the workspace: no `EmbedClient::new`, no `.enroll(`,
  no `.query(` outside the definition file. It is dead code.
- `homeward-match` fuses a **caller-supplied** visual-similarity score
  (`report.rs:28` — `visual_scores` keyed by `canonical_id`; `lib.rs:75`). Nothing
  in the owner path ever computes or supplies those scores, so the fusion runs
  geo+date only — the visual half of the matcher is inert.
- `homeward-reportd`'s match path is **stubbed**: `cmd_match` builds
  `make_stub_report(...)` + `make_stub_candidate(0.9)` ("stub for CLI pipeline
  demonstration", `reportd.rs:260-261,309-369`, `photos: vec![]`). The owner
  submits a real `--photo` (bytes are read at `reportd.rs:128`) and it goes
  nowhere near the matcher.
- `homeward-report` depends on **neither** `homeward-match` **nor** the embed
  client (empty grep over its `Cargo.toml`). The owner photo → embed `/query` →
  visual scores → match fusion → ranked shortlist chain has no wire end-to-end.

This is [[project_voice_input_null_detectors]] at fleet scale: every part exists,
the connections between them are placeholders. The deliver fleet wires them, on
both sides (gallery + owner), and proves it end-to-end on the bundled fixtures.

### Deliver components (drafted this pass)

- **homeward-embed-client** (rust-extend → new workspace crate) — lift
  `embed_client.rs` out of `homeward-ingest` into a small shared
  `homeward-embed-client` crate so both ingest (enroll) and report (query) can
  call the sidecar without report pulling in the ingest daemon. Foundation.
- **homeward-deliver-enroll** (rust-extend → homeward-ingest) — wire the ingest
  daemon to call `/enroll` on every new/changed intake photo so the gallery the
  matcher queries is actually populated; honest no-op when the sidecar is absent.
- **homeward-deliver-query** (rust-extend → homeward-report) — replace reportd's
  `make_stub_report`/`make_stub_candidate` with the real flow: stored
  `LostReport` photo → `/query` → `visual_scores` → `homeward-match` fusion → a
  real ranked shortlist. Removes the last owner-facing stub.
- **homeward-deliver-attest** (mixed → homeward) — an end-to-end smoke that stands
  the sidecar up, enrolls the bundled `eval-smoke/gallery`, submits the
  `eval-smoke/query` fixture through `homeward-reportd`, and asserts the correct
  individual ranks top with measured latency. The "made to deliver" capstone.

### Deliver order

```
homeward-embed-client ─► homeward-deliver-enroll ─┐
                      └─► homeward-deliver-query  ─┴─► homeward-deliver-attest
```

embed-client is the foundation both wires need. enroll (gallery) and query
(owner) are independent and parallelizable once it lands. attest depends on both
— it can only prove the round-trip once both halves are wired.

### Still un-dreamt after deliver (outreach, not builds)

- A live PetFace held-out accuracy number (research-gated dataset = manual
  download).
- A real email-relay credential to flip alert-delivery from dry-run to live.
- A Pet FBI/HeLP partner write agreement to flip the export adapter live.
- Public/remote exposure of the report API (a deliberate, gated decision).

## Catchment fleet (drafted 2026-06-13)

Fleets 1–4 (core, federation, operate, deliver) built and wired the whole
pipeline: a photo submitted by an owner now runs end-to-end through embed →
match → ranked shortlist, proven on fixtures. But Phase-1 live inspection
(2026-06-13) of `~/wintermute/homeward` found the funnel's **mouth is nailed
shut at four cities**, and coverage is the entire point of the vision (more
sheltered-pet sources = more reunions; end-state #1–#2):

- **Sources are compile-time constants.** `SocrataConfig` and
  `SocrataColumnMap` are structs of `&'static str`; `SocrataConfig::austin()`
  is a `const fn` (`socrata.rs:78`). Adding any of the "hundreds more" STRAY
  portals the vision names (Bloomington, and every Socrata/OpenDataSoft
  municipal animal-services dataset) means **editing Rust and recompiling.**
- **The registry is hand-listed.** `main.rs:24` builds the connector set from a
  literal `[austin(), dallas(), sonoma(), long_beach()]`. `ConnectorRegistry`
  is a runtime name→connector map, but nothing populates it from a file.
- **Onboarding a portal is manual archaeology.** Each `SocrataColumnMap` was
  hand-derived by reading one dataset's column docs. Nothing probes a candidate
  `{domain, dataset_id}` to tell you whether it even carries a STRAY intake-type
  column or which columns map to what.
- **No catchment map.** An operator running homeward cannot see which sources
  are live, which have never returned a record, or which US metros have no feed
  at all — so coverage holes stay invisible.

None of the four shipped fleets widened the funnel; they wired the pipe behind
it. This fleet makes **adding a sheltered-pet source a config/data operation,
not a recompile** — and gives the operator a map of where homeward can and
cannot yet bring a pet home.

### Catchment components (drafted this pass)

- **homeward-source-registry** (rust-extend → homeward-connectors) — make
  `SocrataConfig`/`SocrataColumnMap` deserializable from a `sources.toml` and
  load the connector set from `HOMEWARD_SOURCES` at startup, falling back to the
  four built-ins. Owned `String` config alongside the `const fn` built-ins.
  Foundation; everything else needs it.
- **homeward-source-probe** (rust-extend → homeward-connectors) — a
  `homeward-connectors probe <domain> <dataset_id>` subcommand that hits the SODA
  metadata/`$limit=1` endpoint, detects a STRAY-bearing intake-type column +
  the required columns, and emits a *draft* `sources.toml` entry or an honest red
  verdict. Turns onboarding into probe → review → commit. Depends on registry.
- **homeward-source-catalog** (mixed → homeward) — a committed, validated
  `sources.toml` seed catalog of known-good municipal STRAY feeds (the four
  built-ins + the next tier), each entry carrying its probe verdict +
  last-validated date, plus a parse-and-load test. The data deliverable that
  actually widens catchment. Depends on registry (format) + probe (validation).
- **homeward-coverage-report** (rust-extend → homeward-connectors) — a
  `homeward-connectors coverage` subcommand reporting per-source {last-success,
  record count, STRAY count, declared metro} and flagging catchment holes
  (registered-but-never-succeeded; named metro with no feed). The operator's
  map. Depends on registry; independent of probe/catalog.

### Catchment order

```
homeward-source-registry ─► homeward-source-probe ─► homeward-source-catalog
                        └─► homeward-coverage-report  (independent of probe/catalog)
```

registry is the foundation (the file format both probe and catalog speak).
probe validates candidate portals into catalog entries. coverage-report reads
the registry and the live store; it needs only the registry.

### Still un-dreamt after catchment

- OpenDataSoft + ArcGIS Open Data portals use a different query dialect than
  Socrata/SODA; a second connector family (not just config) widens catchment
  past Socrata-only municipalities — a future `/dream extend homeward` pass once
  the Socrata catalog is exhausted.
- Auto-discovery of new portals (crawl the Socrata federated catalog for
  animal-intake datasets) rather than hand-fed `{domain, dataset_id}` — research
  whether the catalog API exposes enough to filter to STRAY feeds.

## Reach fleet (drafted 2026-06-14)

Fleets 1–5 (core, federation, operate, deliver, catchment) built the pipeline and
made *adding a Socrata source* a config operation (`homeward-source-registry` loads
`deploy/sources.toml`; `homeward-source-probe` onboards a `{domain, dataset_id}`;
`homeward-coverage-report` maps the holes). But Phase-1 live inspection of
`~/wintermute/homeward` v0.24.0 (2026-06-14) confirms the catchment fleet's own
parting note — **the funnel is still Socrata-only**:

- `grep -rl 'Socrata|OpenDataSoft|ArcGIS' homeward-connectors/src` returns
  `socrata.rs`, `rescuegroups.rs`, `petfbi.rs` — and **nothing** for OpenDataSoft
  or ArcGIS. Three connector families exist; the two that cover the *other half* of
  US municipal open-data portals do not.
- `deploy/sources.toml` is `[[socrata]]` arrays only (`socrata.column_map.*`). A
  city whose animal-services dataset lives on an OpenDataSoft portal or an Esri
  ArcGIS Hub Feature Service is **unreachable at any config** — the loader has no
  family for it. `probe.rs` hits SODA metadata endpoints exclusively.
- The catchment fleet named exactly this gap as un-dreamt: *"OpenDataSoft + ArcGIS
  Open Data portals use a different query dialect than Socrata/SODA; a second
  connector family (not just config) widens catchment past Socrata-only
  municipalities"* and *"auto-discovery of new portals … rather than hand-fed
  `{domain, dataset_id}`."*

Coverage is the entire point (end-state #1–#2: more sources → more reunions). The
catchment fleet widened *how* sources are added; this fleet widens *which kinds of
source can be added at all*, and closes the manual-discovery gap. The `Connector`
trait (`connector.rs` — `poll(Cursor) -> Vec<PetRecord>` + `provenance` +
`cadence_hint`) is the clean seam: each new family is one more `impl Connector`,
no pipeline change downstream.

### Reach components (drafted this pass)

- **homeward-source-family** (rust-extend → homeward-connectors) — grow
  `deploy/sources.toml` and the registry loader to parse `[[opendatasoft]]` and
  `[[arcgis]]` table-arrays alongside `[[socrata]]`, each into a family-tagged
  config the registry dispatches to the right connector constructor. Existing
  `[[socrata]]` entries load unchanged (back-compat). Foundation — both new
  connectors need a way to be named in the catalog.
- **homeward-opendatasoft-connector** (rust-extend → homeward-connectors) — an
  `OpenDataSoftConnector` implementing `Connector` against the documented ODS
  Explore API v2.1 (`/api/explore/v2.1/catalog/datasets/{id}/records?where=…&
  order_by=…&limit=…`, ODSQL `where` dialect, `record.timestamp` watermark for the
  `Cursor`), normalizing animal-intake rows to `PetRecord`. Includes ODS-family
  recognition in `probe`. Depends on source-family.
- **homeward-arcgis-connector** (rust-extend → homeward-connectors) — an
  `ArcGisConnector` against the ArcGIS REST Feature Service query API
  (`/FeatureServer/0/query?where=…&outFields=*&f=geojson&resultOffset=…`,
  `EditDate`/`last_edited_date` watermark, `resultOffset` paging) for the many US
  shelters published on Esri Hub. Includes ArcGIS-family recognition in `probe`.
  Depends on source-family; independent of opendatasoft.
- **homeward-source-discover** (rust-extend → homeward-connectors) — a
  `homeward-connectors discover [--families …] [--metro …]` subcommand that crawls
  the Socrata federated catalog API (`api.us.socrata.com/api/catalog/v1?q=animal
  +intake&only=dataset`) and the ODS catalog discovery endpoint to emit a ranked
  list of *candidate* `{family, domain, dataset_id}` for `probe` to validate —
  turning "hand-fed dataset ids" into "discover → probe → review → commit." Honest:
  emits candidates, never auto-commits a source. Depends on source-family (the
  candidate shape it speaks).

### Reach order

```
homeward-source-family ─► homeward-opendatasoft-connector ─┐
                       ├─► homeward-arcgis-connector       ─┤
                       └─► homeward-source-discover         ┘
```

source-family is the foundation (the catalog format all three new pieces speak).
The two connectors and discover are mutually independent once it lands — three
parallel branches off one foundation.

### Still un-dreamt after reach

- Geo/PostGIS-backed catchment so coverage holes are a *map of metros*, not a list
  of sources — a visualization/data concern beyond connectors.
- A scheduler that raises a source's cadence when it's actively returning STRAY
  intakes and backs off silent ones (adaptive per-family) — `cadence_hint` exists
  but the orchestrator treats it as static.
- Non-US open-data portals (UK `data.gov.uk`, EU ODS instances) — the vision is
  US-framed; international reach is a deliberate scope decision, not a build.
