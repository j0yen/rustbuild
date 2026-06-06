# PRD: relay-directory

Status: Draft v0.1
build_target: rust-cli
Vision: visions/relay.md

## TL;DR

A frontline helper can't refer someone to a service they can't find. Resource
directories today are scattered across PDFs, county websites, and stale 211
spreadsheets. `relay-directory` is the foundation of the `relay` workspace: it
ingests human-services resource listings from open, standard sources, normalizes
them into one local, queryable schema, and answers "what services exist for need
X near location Y that this person is eligible for?" — entirely on-device. This
PRD creates the `~/wintermute/relay` cargo workspace and the `relay` binary.

## Why this exists

- **Evidence — the homeward pattern is proven here.** `~/wintermute/homeward`
  already ingests open civic data (Socrata, RescueGroups) into a normalized
  local store and queries it (memory `project_homeward_lost_pets`,
  homeward-schema/connectors/ingest crates shipped 2026-06-05). `relay-directory`
  reuses that exact ingest→normalize→query shape, pointed at human services.
- **Evidence — a real interop standard exists.** Open Referral's **HSDS** (Human
  Services Data Specification) is the established schema for 211s and resource
  directories; anchoring on it means real data exists to ingest, not a toy.
- **Why local:** a helper in a rural clinic or a pop-up mutual-aid site has no
  reliable internet and no budget. An on-device directory works offline and free.

## What this builds

- New cargo workspace at `~/wintermute/relay` (Rust 2021, rust-toolchain pinned),
  with a `relay-directory` lib crate and a thin `relay` binary (`[[bin]]`) that
  hosts the workspace's subcommands (`relay directory ...` to start).
- **Schema** (`relay-directory` lib): a normalized `Resource` model — name, org,
  service types (taxonomy: food, shelter, legal, health, benefits, ...),
  location (lat/lon + address), contact, hours, eligibility flags, languages,
  source provenance. Mirrors the HSDS core entities (organization / service /
  location / service_at_location) collapsed to what a matcher needs.
- **Ingest**: an `Ingestor` trait + an HSDS-JSON ingestor (parse a published HSDS
  export → `Vec<Resource>`), plus a simple CSV ingestor for 211-style sheets.
  Dedup near-identical entries (same org+service+location) deterministically.
- **Store**: local SQLite (rusqlite) at `~/.local/share/relay/directory.db`;
  upsert by stable source id; full-text + structured query.
- **Query API + CLI**: `relay directory query --need food --near <lat,lon>
  --radius-km N --eligibility <flags> --json` → ranked-by-proximity results.
  `relay directory import <hsds.json|csv>`; `relay directory stats`.

## Acceptance criteria

1. `cargo build --release` produces a `relay` binary; `relay directory --help`
   lists `import`, `query`, `stats`.
2. Importing a fixture HSDS JSON export yields the expected normalized
   `Resource` rows (golden test: known input → known schema output).
3. The CSV ingestor parses a 211-style sheet into the same schema; malformed rows
   are skipped with a counted warning, never panic.
4. Dedup: two source records for the same org+service+location collapse to one
   `Resource` (deterministic, tested on a fixture with known duplicates).
5. `query --need food --near <lat,lon> --radius-km 10` returns only food
   resources within 10km, ordered by distance (golden test on a seeded store).
6. Eligibility filtering: `--eligibility under18` excludes resources whose
   eligibility flags exclude minors (tested on fixtures).
7. Store is durable: import, drop the process, re-open, query — rows persist.
8. SIGPIPE-safe (`sigpipe::reset()` in main) so `relay directory query | head`
   doesn't panic; strict clippy (`unwrap/expect/panic = deny`) clean.

All ACs are deterministic and require no network or LLM → fully cloud-build-safe.
