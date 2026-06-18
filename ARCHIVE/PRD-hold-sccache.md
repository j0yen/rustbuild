# PRD: hold-sccache — wire a size-capped local compilation cache

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/hold.md
**Repo:** j0yen/hold-sccache (NEVER AtScaleInc)

## TL;DR

The cloud build box already shares one `sccache-dist` cache (constellation
vision); the laptop that runs local builds has none (`which sccache` → not
found), so every local `cargo build` recompiles dependencies from scratch.
hold-sccache is the low-contention dedup lever: it installs `sccache` (built in
the cloud to avoid a heavy local compile), wires it as `RUSTC_WRAPPER` with a
size-capped on-disk cache, and reports the hit-rate. Unlike the shared
`CARGO_TARGET_DIR` (hold-anchor), sccache imposes **no shared build lock** — each
repo can keep its own `target/` and still share the expensive rustc outputs, so
it is safe even under parallel `/build`.

## Why this exists

Verified 2026-06-16:

- `which sccache` → **not found**; no local compilation cache exists.
- `recall/target/debug/deps` = 9.2G, `wintermute-brain` 5.7G deps — the same
  common crates (`serde`, `tokio`, `clap`) are recompiled in 190 repos.
- The constellation vision already standardizes `sccache-dist` for the cloud
  fleet; the local box is the asymmetry. Bringing the same caching home is the
  lowest-risk lever because it does not serialize parallel builds the way a
  single `CARGO_TARGET_DIR` does (see hold-anchor's lock caveat).

## What this builds

A single Rust CLI `hold-sccache` (clap) that manages a local sccache install +
its wiring + reporting. It does not vendor sccache source; it installs a binary.

**Modules**
- `install` — ensure an `sccache` binary on `$PATH` (default `~/.local/bin`).
  Prefer building via `/cloudbuild` (`cargo install sccache` on the Hetzner box,
  pull the binary back) per the user's cloud-first build rule; fall back to a
  documented `cargo install sccache` only with `--local-build`. Idempotent: skip
  if a current `sccache` is already present.
- `wire` — write/merge `RUSTC_WRAPPER = "sccache"` into
  `~/wintermute/.cargo/config.toml` under `[build]` (composing with hold-anchor's
  `target-dir`; never clobber it), and set the sccache cache dir + size cap via a
  config file at `~/.config/sccache/config` (or `SCCACHE_DIR` /
  `SCCACHE_CACHE_SIZE`). Default cap: 20G (overridable).
- `stats` — shell out to `sccache --show-stats`, parse, and emit JSON
  (`cache_hits`, `cache_misses`, `hit_rate`, `cache_size`, `max_size`).
- `unwire` — remove the `RUSTC_WRAPPER` key (idempotent); never deletes the
  cache contents.

**Deps:** `clap`, `serde`/`serde_json`, `toml`/`toml_edit`, `anyhow`.

**UX**
- `hold-sccache install` → ensure binary present (cloud-built by default).
- `hold-sccache wire --max-size 20G` → set RUSTC_WRAPPER + cache cap.
- `hold-sccache stats` → JSON hit-rate report.
- `hold-sccache status` → installed? wired? cache size vs cap?
- `hold-sccache unwire` → remove the wrapper key.

## Acceptance criteria

1. `hold-sccache --help` lists `install`, `wire`, `unwire`, `stats`, `status`;
   exits 0.
2. `wire` merges `RUSTC_WRAPPER = "sccache"` under `[build]` in the fleet
   `config.toml` **without** removing a pre-existing `target-dir` key written by
   hold-anchor (verify both keys present after).
3. `wire` is idempotent: a second run leaves `config.toml` byte-identical.
4. `wire --max-size 20G` records a 20G cap in the sccache config / env file that
   `status` reads back as `max_size`.
5. `unwire` removes only `RUSTC_WRAPPER` (other `[build]` keys preserved), is
   idempotent, and does not delete the cache directory.
6. `stats` parses a fixture `sccache --show-stats` output (captured sample, no
   live sccache required in test) into JSON with `cache_hits`, `cache_misses`,
   and a computed `hit_rate` in `[0,1]`.
7. `status` reports `installed`, `wired`, `cache_bytes`, `max_bytes` and exits 0
   whether or not sccache is actually installed (degrades, does not panic).
8. `install` with no network / no cloud reachable fails with a clear actionable
   message (how to `--local-build`) and a non-zero exit — it never leaves a
   half-wired config (wire is a separate step).
