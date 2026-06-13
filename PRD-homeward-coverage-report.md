# PRD: homeward-coverage-report — a map of where homeward can bring a pet home

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

Once sources are data (registry) and the catalog is wide (catalog), an operator
needs to see where homeward actually listens and where it's blind. This PRD ships
`homeward-connectors coverage`: for every registered source it reports
{last-success, total records, STRAY records, declared metro} and flags catchment
holes — sources registered but never returning a record, and named metros with no
feed at all. It turns "we have N connectors" into "here is the map of the country
homeward can and cannot reach."

## Why this exists

- The vision's end-state #1–#2 is near-real-time coverage of sheltered pets; you
  cannot manage coverage you cannot see. Phase-1 inspection (2026-06-13): the
  orchestrate fleet shipped `homeward health` (liveness of the three daemons) but
  nothing reports **catchment** — which sources are productive vs inert, or where
  the geographic holes are.
- A registered-but-silent source is the dangerous failure: a renamed/retired
  municipal dataset keeps its `[[socrata]]` entry, returns zero records, and the
  store quietly stops covering that metro — indistinguishable from "that metro had
  no strays today" unless something tracks last-success per source.
- This is the operator-facing complement to homeward-source-catalog's
  `CATCHMENT.md` (the *intended* map): coverage-report is the *actual* map, read
  from the registry + the live store.

## What this builds

A `coverage` subcommand on the `homeward-connectors` binary:

- **`homeward-connectors coverage [--store <path>] [--json]`** — reads the loaded
  registry (the same `HOMEWARD_SOURCES` catalog) and, when a store path is given,
  the canonical record store the ingest daemon writes.
- **Per-source row:** `name`, declared `metro` (from a catalog field/comment if
  present, else blank), `last_success` timestamp, `record_count`, `stray_count`,
  and a status: `LIVE` (recent success + records), `STALE` (last success older
  than the source's cadence × K), `SILENT` (registered, zero records ever),
  `UNREACHABLE` (last poll errored).
- **Catchment-hole flags:**
  - any `SILENT`/`UNREACHABLE` source listed under a "needs attention" heading;
  - a "known gaps" passthrough from the catalog's documented holes (a metro with
    no feed), so the report unifies intended + actual gaps in one place.
- **`--json`** emits a structured `{sources: [...], holes: [...]}` object for the
  orchestrate `homeward status`/health surface to consume later.
- **Honest when the store is absent:** with no `--store`, report registry-only
  facts (which sources are registered, declared metros, catalog gaps) and clearly
  mark record/last-success columns as `unknown` — never fabricate counts.
- Reuses the store-reading shape ingest already uses; no new store format, no
  network in the report path (it reads local state).

## Acceptance criteria

1. `homeward-connectors coverage` lists every registered source with its status;
   against a fixture store, counts (`record_count`, `stray_count`) and
   `last_success` match the fixture data per source.
2. A source registered but absent from the store reports `SILENT` and appears
   under "needs attention"; a source whose last success exceeds its cadence×K
   reports `STALE`.
3. `--json` emits a parseable `{sources, holes}` object; a round-trip test
   deserializes it and asserts one known SILENT source and one catalog-documented
   metro gap both appear in `holes`.
4. With no `--store`, the command still runs, lists registered sources + declared
   metros + catalog gaps, and marks record/last-success columns `unknown` rather
   than `0` (no fabricated coverage).
5. The status thresholds (STALE multiplier K, "recent" window) are explicit
   constants documented in `--help`, not magic numbers buried in code.
6. `cargo test` green with fixture stores (no live network); `cargo build
   --release` ships the subcommand. MSRV 1.85, no let-chains, `sigpipe::reset()`
   already first in `main`.

## Out of scope

- Time-series/historical coverage trends (this is a point-in-time snapshot).
- Alerting on coverage loss (a future tie-in to the pulse/health channel, not
  here).
- Probing/onboarding new sources (homeward-source-probe).
