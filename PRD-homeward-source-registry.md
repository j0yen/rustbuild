# PRD: homeward-source-registry — sources are data, not a recompile

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

homeward's whole purpose is coverage — the more sheltered-pet sources it ingests,
the more lost pets it can bring home — yet its Socrata sources are **compile-time
constants**. Adding a city means editing Rust and shipping a new binary. This PRD
makes `SocrataConfig`/`SocrataColumnMap` loadable from a `sources.toml` file at
startup (`HOMEWARD_SOURCES`), so onboarding a municipal STRAY feed becomes a
config commit. The four built-in cities remain as a fallback when no file is set —
zero behavior change for existing deployments.

## Why this exists

Phase-1 live inspection of `~/wintermute/homeward` (2026-06-13):

- `homeward-connectors/src/connectors/socrata.rs:62` — `SocrataConfig` is a struct
  of `&'static str` fields (`domain`, `dataset_id`, `name`, `app_token_env`) with
  a `&'static str` `SocrataColumnMap`. `SocrataConfig::austin()` (`socrata.rs:78`)
  is a `const fn`. Sources are baked into the binary.
- `homeward-connectors/src/main.rs:24` — `build_registry()` hand-lists
  `[SocrataConfig::austin(), dallas(), sonoma(), long_beach()]`. A fifth city is a
  source edit + recompile + redeploy.
- The vision names "hundreds more" Socrata/OpenDataSoft municipal portals that
  uniquely tag `Intake Type = STRAY` — the someone's-lost-pet population. Every
  one of them is currently unreachable without a developer.
- `homeward-connectors/src/registry.rs` already has a runtime `ConnectorRegistry`
  (name → `Box<dyn Connector>`); the missing half is a way to *populate* it from a
  file instead of from constructors.

This is the same shape as the wider autobuilder lesson that the unread config is
doing work just by existing — but here the config doesn't exist yet, so the work
can't happen.

## What this builds

Extends `homeward-connectors`:

- **An owned-config mirror.** A `SocrataColumnMapOwned`/`SocrataConfigOwned` (or
  convert `SocrataColumnMap`/`SocrataConfig` to owned `String` fields with the
  `const fn` built-ins reworked to `fn` returning the owned form — implementer's
  choice, whichever keeps the `SocrataConnector::new` seam unchanged). The
  connector must build identically from a built-in and from a file-loaded config.
- **`serde::Deserialize` on the config.** A `sources.toml` shaped as:

  ```toml
  [[socrata]]
  name = "austin"
  domain = "data.austintexas.gov"
  dataset_id = "fdzn-9yqv"
  app_token_env = "SOCRATA_APP_TOKEN"   # optional
  [socrata.column_map]
  animal_id = "animal_id"
  animal_type = "animal_type"
  intake_type = "intake_type"
  intake_date = "datetime"              # optional fields omitted = None
  found_location = "found_location"
  breed = "breed"
  name = "name"
  color = "color"
  ```

- **A loader** `SourceCatalog::from_path(&Path) -> Result<Vec<SocrataConfig…>,
  ConnectorError>` with a typed error on malformed TOML / missing required field
  (`name`, `domain`, `dataset_id`, and the three required column-map fields
  `animal_id`/`animal_type`/`intake_type`).
- **Wire `build_registry()`** to read `HOMEWARD_SOURCES`: if set and readable,
  register the file's sources; if unset, register the four built-ins exactly as
  today. A malformed file is a hard error (eprintln + skip that source, never
  silently register a half-parsed config), not a silent fallback to built-ins —
  the operator must know their catalog didn't load.
- **No new dependency** beyond `toml` + `serde` (already in the workspace via
  other crates; add to this crate's `Cargo.toml` if absent). No network, no I/O
  beyond reading the one file.

## Acceptance criteria

1. `SocrataConfig` (or an owned sibling consumed by `SocrataConnector::new`) can be
   constructed from a deserialized `sources.toml` entry, and a connector built
   from a file-loaded config behaves identically to one built from the matching
   `const fn` built-in (same `name`, `domain`, `dataset_id`, column map).
2. `SourceCatalog::from_path` parses a well-formed multi-`[[socrata]]` TOML file
   into N configs; a round-trip test loads a file written from the four built-ins
   and asserts the four configs match the built-ins field-for-field.
3. A malformed file (bad TOML, or a `[[socrata]]` missing a required field) yields
   a typed `ConnectorError` naming the offending source/field — not a panic, not a
   silent drop.
4. With `HOMEWARD_SOURCES` pointing at a file containing only `dallas`,
   `build_registry()` registers exactly `dallas` (verified via
   `ConnectorRegistry::names()`); with `HOMEWARD_SOURCES` unset, it registers the
   four built-ins as before.
5. Optional column-map fields omitted from TOML deserialize to `None`; required
   fields absent are a load error (AC3), never `Some("")`.
6. `cargo test` green; `cargo build --release` produces a `homeward-connectors`
   binary. MSRV 1.85, no let-chains, `sigpipe::reset()` already first in `main`.

## Out of scope

- Probing/validating a candidate portal (that is homeward-source-probe).
- Shipping the actual catalog of cities (that is homeward-source-catalog).
- RescueGroups/PetFbi config-from-file (env-keyed already; Socrata is the
  recompile pain). A follow-on can generalize if it earns it.
