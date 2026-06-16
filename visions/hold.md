# Vision: hold — one shared cargo hold, not 190 private ones

**Author:** /dream (Claude Sonnet 4.6), for jsy
**Created:** 2026-06-16
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection. The 2026-06-16 self-review
named "DISK 92% (+6% in one day)" the headline; `df` now reads **97% used, 18G
free of 468G**. The ballast vision answers this *curatively* (measure → reap
fossils). hold answers it *preventively*: stop manufacturing the duplication in
the first place.

## TL;DR

`du -sch ~/wintermute/*/target` = **214G across 190 `target/` directories**, and
there is **no `CARGO_TARGET_DIR` and no local `sccache`** (`sccache not found`).
Every one of the 190 repos compiles its dependency graph into its own private
`target/`: `recall/target/debug/deps` alone is **9.2G**, `wintermute-brain` is
5.7G deps + 3.3G incremental, `agorabus` 3.1G deps. These graphs overlap heavily
— `serde`, `tokio`, `clap`, `anyhow`, `serde_json` are locked in nearly every
`Cargo.lock` (recall 406 pkgs, agorabus 92, mqo-bench 65) — so the same crate is
recompiled and re-stored tens of times across the fleet. The cloud build box
already shares a single `sccache-dist` cache (per the constellation vision); the
laptop that does the local builds has nothing. **hold brings the cloud's dedup
discipline home: measure the duplication honestly, anchor one shared build
directory, wire a size-capped local `sccache`, migrate existing repos onto the
hold safely, and bound the hold so it can never regrow to 214G.** Where ballast
*jettisons* fossil weight, hold stops the weight from accumulating.

## End-state

When this vision is fulfilled:

1. **The duplication is quantified, not guessed.** `hold-survey --json` reports,
   across all repos under `~/wintermute`, the *sum* of per-repo dependency-build
   bytes vs the *union* (deduplicated by package+version+features), so the
   reclaimable-by-sharing figure is a measured number, not a hunch. It deletes
   nothing.
2. **One build directory, not 190.** A `~/wintermute/.cargo/config.toml` anchors
   `CARGO_TARGET_DIR` to a single shared hold (`~/wintermute/.hold/target`).
   cargo namespaces artifacts by fingerprint, so a dependency built once for one
   crate is reused by every other crate that locks the same version — the union,
   not the sum, lands on disk.
3. **Compilation is cached locally, like it already is in the cloud.** A
   size-capped local `sccache` is wired as `RUSTC_WRAPPER`, so even repos that
   keep a private `target/` (to avoid the shared-target build lock) share the
   expensive *rustc* outputs. This is the low-contention lever: cache hits, no
   serialization.
4. **Existing private targets drain into the hold safely.** `hold-migrate`
   moves each repo onto the shared hold only after confirming its installed
   binary is current (reusing ballast's fossil classification), `cargo clean`s
   the private `target/`, and lets the first shared build repopulate the union.
   Gated, logged, reversible-in-spirit.
5. **The hold stays bounded.** `hold-guard` caps the shared directory and
   LRU-evicts stale crate fingerprint subdirs when it crosses the cap, emitting
   the same structured SLO event schema `ballast-guard` already uses (compose,
   don't duplicate) so a notifier can surface it. The commons cannot silently
   regrow to 214G.

## Components (PRD-sized)

- **hold-survey** — KEYSTONE. Read-only. Walks every repo's `target/` + parses
  each `Cargo.lock`; computes sum-of-dep-builds vs deduplicated-union and the
  reclaimable-by-sharing estimate; emits JSON. Measurement before action.
- **hold-anchor** — establishes the shared `CARGO_TARGET_DIR` via a discovered
  `~/wintermute/.cargo/config.toml`; idempotent; verifies cargo picks it up;
  documents and detects the parallel-build target-lock tradeoff.
- **hold-sccache** — installs (cloud-build to avoid local compile) + wires a
  size-capped local `sccache` as `RUSTC_WRAPPER`; the low-contention dedup lever;
  reports cache hit-rate. Complements the cloud `sccache-dist`.
- **hold-migrate** — drains existing private `target/` dirs into the hold,
  per-repo, gated on installed-binary-current (borrowing ballast's fossil check),
  with an append-only ledger of reclaimed bytes.
- **hold-guard** — caps the shared hold; LRU-evicts crate fingerprint dirs over
  the cap; emits the ballast-guard SLO event schema. Keeps the commons bounded.

## Order

```
hold-survey ──► hold-anchor ──► hold-migrate ──► hold-guard
                     │
                     └────────► hold-sccache  (independent of migrate; needs anchor only for the shared-hold sccache dir, but can wire against private targets too)
```

- hold-survey is independent (read-only); build first to size the prize.
- hold-anchor is the enabler everything else assumes.
- hold-sccache and hold-migrate both consume anchor but are independent of each
  other; either order.
- hold-guard is last — it bounds the hold that migrate fills.

## Open questions (for jsy / next /dream pass)

1. **Shared-target lock vs parallel /build.** A single `CARGO_TARGET_DIR`
   serializes concurrent `cargo` invocations on the package-level build lock.
   `/build` advances up to 30 PRDs in parallel — but most heavy compiles route
   to Hetzner (`AUTOBUILDER_CLOUD=1`), so local parallel cargo is already thin.
   Does the shared-target lock actually bite here, or is sccache (no shared
   lock) the safer primary lever? hold-survey should measure local concurrent
   build frequency to settle this before hold-anchor flips the switch.
2. **hold vs ballast boundary.** ballast reaps; hold prevents. hold-guard
   should emit ballast-guard's event schema rather than invent its own — confirm
   that schema is stable enough to depend on, or factor it into a shared crate.
3. **Hold location.** Same `nvme0n1p2` as everything else, or a dedicated
   subvolume/mount so a runaway shared build can't fill `/`? Same disk for now;
   revisit if hold-guard's cap proves hard to hold.
4. **Cross-toolchain targets.** Repos pinned to rustc 1.85 vs 1.88 (per the
   multi-toolchain note) produce incompatible artifacts; a shared hold may
   thrash if two toolchains fight over the same fingerprint dirs. hold-survey
   should bucket repos by pinned toolchain so anchor can decide whether to shard
   the hold per-toolchain.
