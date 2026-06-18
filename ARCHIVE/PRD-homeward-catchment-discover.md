# PRD: homeward-catchment-discover — turn computed coverage holes into candidate feeds

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

homeward already has a `discover` subcommand that crawls the Socrata/ODS
catalogs for candidate animal-intake datasets (shipped:
ARCHIVE/PRD-homeward-source-discover). And, once homeward-catchment-geo lands,
it will compute *where* coverage holes actually are from real shelter points.
This PRD closes the loop: feed the computed holes into `discover` so "we have a
populated region with no feed in metro X" automatically becomes "search the
open-data catalogs for candidate feeds in metro X" — ranked candidates for a
human to probe and commit, never an auto-commit.

## Why this exists

Phase-1 research (2026-06-17):

- `discover` exists but is seeded by hand: an operator passes `--metro` /
  families and it queries the federated Socrata catalog
  (`api.us.socrata.com/api/catalog/v1?q=animal+intake`) and the ODS catalog,
  emitting `{family, domain, dataset_id}` candidates for probe→commit.
- Coverage holes are *also* hand-seeded today (`coverage.rs:397-416`
  `catalog_documented_gaps()` lists LA/NY/Chicago/Houston/Phoenix literally).
  Nothing connects "here is a hole" to "go discover a feed for it."
- After homeward-catchment-geo, holes are *computed* from the ingest store's
  real `ShelterLocation` points — a live, growing list, not a frozen constant.
  Wiring that list into `discover` makes coverage self-extending: the system
  notices its own gaps and proposes how to fill them.
- This is the vision's "discover → probe → review → commit" discipline applied
  to *geographically-identified* need rather than operator intuition.

## What this builds

Extends `homeward-connectors` `discover`:

- A new mode `discover --from-holes [--top N] [--families …]` that:
  1. reads the computed geo holes (from homeward-catchment-geo's coverage
     `--geo` output, or directly via its rollup API),
  2. maps each hole region to query terms (metro/place names from the
     gazetteer or the cell's reverse label),
  3. runs the *existing* catalog-crawl per hole, and
  4. emits ranked `{hole_region, estimated_missed_population, candidate:
     {family, domain, dataset_id, title}}` rows.
- Candidates ranked first by the hole's estimated missed population (so the
  biggest gaps surface first), then by catalog relevance score.
- Strictly **read-only / propose-only**: output is a candidate list for
  `homeward-connectors probe` then human commit — this PRD never writes to
  `sources.toml` and never auto-registers a connector.
- Output formats: `--format json|table`, stable for piping into `probe`.

Deps: reuse the existing catalog-crawl HTTP path from `discover` and the
rollup from homeward-catchment-geo. No new heavy deps.

## Acceptance criteria

1. `discover --from-holes` consumes the geo-hole list produced by
   homeward-catchment-geo (via its output or rollup API) without re-deriving
   coverage itself; tested against a fixture hole list.
2. Each hole region is mapped to ≥1 catalog query term derived from the hole's
   geography (not a hardcoded metro table); tested for a fixture hole.
3. The existing catalog-crawl is reused (not reimplemented) per hole; verified
   by a test that mocks the catalog endpoint and asserts the same request shape
   the standalone `discover` already issues.
4. Candidates are ranked by the hole's estimated missed population descending,
   then catalog relevance; asserted on a fixture with two holes of differing
   magnitude.
5. Output is propose-only: a test asserts the command performs no write to
   `sources.toml` and registers no connector (read-only file assertion).
6. Output (`--format json`) is shaped so it can be fed to the existing `probe`
   path; an integration or documented manual check shows a candidate flowing
   discover→probe.
7. When the hole list is empty (full coverage), the command exits cleanly with
   an empty candidate set and a clear message, not an error.
8. Network access is limited to the catalog endpoints already used by
   `discover` (no new external service); grep/review-asserted.
9. `cargo test` green; `cargo build` green for the workspace.

## Dependencies

- **Depends on homeward-catchment-geo** (consumes its computed holes). Build
  catchment-geo first; this PRD is the second node in the reach-3 chain.

## Honesty notes

- Propose-only is load-bearing: a wrong auto-committed source would poison the
  store. The command emits candidates; a human probes and commits. Do not
  weaken this for green tests.
- A "hole" is an estimate (inherited from catchment-geo); candidates must carry
  that uncertainty forward, never assert "metro X is missing" as fact.
- Respect catalog ToS/rate limits already honored by `discover`; reuse its
  client, don't open a faster path.
