# PRD: ballast-cloudaware — flag the fossil targets the cloud already replaced

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/ballast-survey
**Vision:** visions/ballast.md
**Repo:** j0yen/ballast-survey (extends — NEVER AtScaleInc)

## TL;DR

ballast-survey tells you a `target/` is 13G and three weeks old. It does *not*
tell you the safest fact of all: that the binary it built is already installed,
newer than the target, and now rebuilt in the cloud — so the local target is
pure fossil. ballast-cloudaware adds that classification dimension to survey: it
cross-references each Rust crate against (a) its installed binary's mtime on
`$PATH`/`~/.local/bin`, and (b) whether the crate routes its builds to Hetzner
(`AUTOBUILDER_CLOUD`). A crate whose installed binary is newer than its local
`target/` is flagged `fossil` — the highest-confidence, biggest-win reap
candidate. This is the dimension that turns a raw size list into a safety
ranking.

## Why this exists

Verified on 2026-06-16, the motivating case:

- `wintermute-brain/target` → **13G, last written 2026-05-29**.
- `~/.local/bin/wmd` (the brain binary) → **installed 2026-06-15**, 8.8MB.
- The installed binary is **17 days newer** than the target that nominally
  produced it. The running system depends on `wmd`, not on the local target —
  the live binary came from `/cloudbuild` (heavy compiles route to Hetzner via
  `AUTOBUILDER_CLOUD=1`; see memory `feedback_cloudbuild_over_build`). The local
  target is dead weight that can be reaped with near-zero risk and reclaim 13G.

Without this dimension, ballast-reap can only reason about age — a blunt
instrument that would treat a stale-but-uninstalled crate (where the target is
the *only* copy of work) the same as brain's safely-reproducible fossil. The
`installed-and-newer` signal is what makes aggressive reclamation safe.

## What this builds

Extends the `classify` module of `ballast-survey` with a `fossil` assessment;
adds no new binary.

**New classification fields** on each `rust-target` entry:
- `installed_bin` — resolved path of the crate's installed binary, if any
  (search `~/.local/bin` by crate/bin name; honor a `bin_name` override in
  config for crates whose binary ≠ crate name, e.g. `wintermute-brain` → `wmd`).
- `installed_mtime` / `installed_newer_than_target` (bool).
- `cloud_built` (bool) — true if the crate is known to route to cloud
  (detected via an `AUTOBUILDER_CLOUD` marker file, a `cloud:` field in
  autobuilder manifest, or a config allowlist — document the precedence).
- `fossil` (bool) — `installed_newer_than_target && installed_bin.is_some()`.
  cloud_built strengthens confidence but is not required (the mtime proof stands
  alone).
- `reap_safety` — enum `fossil` > `stale-installed` > `stale-uninstalled` >
  `recent` — a single ranked field reap/guard sort on.

**Integration**
- Reuse `adopt` (`~/.local/bin/adopt` — "Detect shipped wintermute artifacts
  that never got installed") as the source of truth for install state where its
  output is available; fall back to direct `~/.local/bin` mtime probe. Document
  which path was used per entry (`install_source: adopt|probe`).
- `--no-cloudaware` flag preserves the pure-size survey behavior for callers
  that don't want the cross-reference cost.

**Deps:** no new heavy deps; shell out to `adopt` only if present (graceful
absence), else filesystem probe.

## Acceptance criteria

1. A `rust-target` entry whose crate has an installed binary newer than the
   target mtime is flagged `fossil: true` with `reap_safety: "fossil"`.
2. A crate with no installed binary is `fossil: false`,
   `reap_safety: "stale-uninstalled"` (the protect-this case).
3. `bin_name` config override resolves a crate whose binary name differs from
   the crate name (fixture: crate `alpha` → binary `a` installed and newer →
   `fossil: true`).
4. `cloud_built` is set from the documented detection precedence and recorded;
   its absence does not by itself clear `fossil` (mtime proof is sufficient).
5. `install_source` records whether install state came from `adopt` or a direct
   probe; with `adopt` absent the tool still classifies via probe and exits 0.
6. `--no-cloudaware` reproduces the v0.1 survey output exactly (no new fields).
7. Entries are sortable by `reap_safety` rank; `ballast-survey --json` round-trips
   the new fields.
8. Existing ballast-survey ACs still pass; `cargo test` + `cargo clippy` green.
