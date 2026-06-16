# PRD: hold-anchor — establish one shared CARGO_TARGET_DIR for the fleet

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/hold.md
**Repo:** j0yen/hold-anchor (NEVER AtScaleInc)

## TL;DR

hold-survey measures the duplication; hold-anchor removes its cause. It writes a
discovered cargo config at `~/wintermute/.cargo/config.toml` that sets
`build.target-dir` to a single shared hold (`~/wintermute/.hold/target`), so
every `cargo` invocation under `~/wintermute` builds into one directory and
reuses any dependency another crate already built. cargo namespaces artifacts by
fingerprint, so this is safe: the *union* of dependency builds lands on disk
instead of the *sum*. hold-anchor is idempotent, verifies cargo actually picks
the config up, and refuses to act blindly — it documents and detects the
parallel-build target-lock tradeoff the vision flags.

## Why this exists

Verified 2026-06-16: `echo $CARGO_TARGET_DIR` is empty and no
`~/wintermute/.cargo/config.toml` exists, so each of the 190 repos builds into
its own `target/`. cargo discovers `.cargo/config.toml` by walking *up* from the
crate dir, so a single file at the `~/wintermute` root applies to every repo
beneath it without touching any repo's own files — the least-invasive anchor
point. hold-survey quantifies the prize (sum-vs-union dep bytes); hold-anchor is
the one switch that realizes it.

cargo 1.96 is installed (`cargo 1.96.0`). `build.target-dir` is stable and the
canonical way to redirect the target directory globally for a subtree.

## What this builds

A single Rust CLI `hold-anchor` (clap), no network. It manages exactly one file
and verifies behavior; it never deletes existing `target/` dirs (that is
hold-migrate's job, gated).

**Modules**
- `paths` — resolve the anchor root (default `~/wintermute`), the config path
  (`<root>/.cargo/config.toml`), and the hold dir (default
  `<root>/.hold/target`, overridable).
- `config` — read any existing `<root>/.cargo/config.toml` (TOML). If
  `build.target-dir` is already the hold, report `already-anchored` and exit 0
  (idempotent). If a *different* `build.target-dir` exists, refuse and report a
  conflict unless `--force`. Otherwise merge a `[build] target-dir = "<hold>"`
  key, preserving any other keys, and write atomically (tmp + rename).
- `verify` — run `cargo metadata`/`cargo config get build.target-dir` (or a tiny
  throwaway `cargo locate-project` in a temp crate) from inside a repo under the
  root and confirm the effective target-dir resolves to the hold; report the
  observed value.
- `lockcheck` — warn (don't block) if the parallel-build lock is a live risk:
  consult hold-survey's `concurrency` estimate if a survey JSON is passed via
  `--survey <path>`, else print the static caveat that a shared target serializes
  concurrent `cargo` builds on the same package.
- `status` — `hold-anchor status` prints whether the hold is anchored, the hold
  path, its current size, and the verify result. Read-only.

**Deps:** `clap`, `serde`/`serde_json`, `toml`/`toml_edit`, `anyhow`.

**UX**
- `hold-anchor apply` → create/merge the config; verify; print result.
- `hold-anchor apply --hold <dir>` → custom hold location.
- `hold-anchor apply --force` → overwrite a conflicting `build.target-dir`.
- `hold-anchor status` → read-only report.
- `hold-anchor unset` → remove the `target-dir` key (restore per-repo targets),
  idempotent; never deletes the hold contents.

## Acceptance criteria

1. `hold-anchor --help` lists `apply`, `status`, `unset`; exits 0.
2. `apply` against a fixture root with no `.cargo/config.toml` creates
   `<root>/.cargo/config.toml` containing `[build]` `target-dir = "<hold>"` and
   creates the hold dir; exits 0.
3. Running `apply` a second time is a no-op reporting `already-anchored`; the
   config file's contents are byte-identical to after the first run (idempotent).
4. `apply` against a root whose config already has a *different*
   `build.target-dir` exits non-zero with a conflict message and leaves the file
   unchanged; the same call with `--force` overwrites it and exits 0.
5. `apply` preserves unrelated pre-existing keys in `config.toml` (e.g. a
   `[net]` or `[registries]` table is still present after the merge).
6. `verify` reports the effective target-dir resolves to the hold when run from a
   fixture crate beneath the anchored root (use a real `cargo` invocation, not a
   string check of the file).
7. `status` on an anchored root reports `anchored: true`, the hold path, and a
   size; on an un-anchored root reports `anchored: false`. Exits 0 in both.
8. `unset` removes only the `target-dir` key (other keys preserved), is
   idempotent, and does not delete the hold directory or its contents.
9. With `--survey <hold-survey.json>` passed, `apply` echoes the
   concurrency/lock-contention caveat sourced from that JSON.
