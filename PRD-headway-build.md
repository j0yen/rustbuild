# PRD: headway-build — the sanctioned build primitive (cloudbuild, never local)

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/headway
Vision: visions/headway.md

## TL;DR

The fleet has no reusable primitive that recompiles a crate the *sanctioned*
way. The only automated rebuild path that exists — `rollout` — builds locally
with `cargo build --release`, which violates the standing hard rule that all
cargo routes through cloudbuild. This PRD builds `headway`, a new Rust CLI +
library whose one job is: given a crate directory, route the build through
`cloudbuild.sh build <crate>`, pull the artifact, and emit a structured verdict.
It is the foundation crate the rest of the `headway` fleet extends.

## Why this exists

Phase-1 inspection, 2026-06-18:

- `rollout/src/fleet.rs:71` hardcodes the default `build_cmd` as
  `"cargo build --release"` — a **local** compile. The standing rule
  (`feedback_cloudbuild_over_build`) is that ALL cargo must route through
  `cloudbuild.sh` (Hetzner); local `/autobuilder` execution is disabled.
- The cloudbuild path was *intended* but left manual: `rollout/src/warmswap.rs:630`
  carries an `#[ignore]`d placeholder test commenting "CI (cloudbuild) stays
  hermetic. Run manually with: …" — the wiring was never done.
- `cloudbuild.sh` is the working automation: `bash ~/.claude/skills/cloudbuild/cloudbuild.sh
  build <crate>` does "up → build → pull → DOWN" (skill doc; `cloudbuild.sh:512`
  dispatches the `build` verb, `cmd_build` at `:323`/`:386` handles the pull).
- The recurring self-review item "agorabus 3d behind source; needs cloudbuild +
  reload --apply" (journal 2026-06-18) is unactionable precisely because no
  primitive wraps the cloudbuild build step for the freshness loop to call.

A primitive that every freshness consumer (agorabus reload --build, rollout,
headway-verify) can call means the cloudbuild-only rule is enforced in *one*
place instead of re-litigated per caller.

## What this builds

New repo `~/wintermute/headway/` (Rust workspace, MSRV 1.85, edition 2021;
autobuilder scaffold incl. `sigpipe::reset()` as the first line of `main` per the
standing SIGPIPE-panic lesson).

- **Model:** a `BuildPlan { crate_dir, crate_name, source_head: String,
  installed_version: Option<String> }` and a `BuildVerdict { crate_name,
  artifact_path: Option<PathBuf>, version_before: Option<String>, version_after:
  Option<String>, cloudbuild_status: Status, elapsed_ms, status }` where
  `Status` ∈ `{built, no-op-fresh, cloudbuild-unreachable, build-failed}`.
- **`build(plan, cfg) -> BuildVerdict`** — shells to
  `cloudbuild.sh build <crate>` (path resolved from `HEADWAY_CLOUDBUILD` env or
  the default `~/.claude/skills/cloudbuild/cloudbuild.sh`), captures exit + the
  pulled artifact path, and stamps version before/after.
- **CLI:** `headway build <crate-dir> [--dry-run] [--format json|table]`.
  `--dry-run` is the default posture (print the plan + the cloudbuild command
  that *would* run, mutate nothing), matching the agorabus reload precedent.
- **Hard guard:** there is NO local-cargo code path. If `cloudbuild.sh` is
  missing or returns "unreachable" (cannot reach Hetzner / SSH fails), `build`
  returns `cloudbuild-unreachable` with a structured error and a non-zero exit —
  it never falls back to `cargo build`.

Deps: `clap`, `serde`/`serde_json`, `anyhow` (or `thiserror`). No new external
services; cloudbuild is invoked as a subprocess.

## Acceptance criteria

1. `headway build <crate-dir> --dry-run --format json` prints a `BuildPlan` plus
   the exact `cloudbuild.sh build <name>` command line that would run, mutates
   nothing, and exits 0. (Default posture is dry-run.)
2. With `--no-dry-run` (apply), `build` invokes `cloudbuild.sh build <name>` as a
   subprocess and returns a `BuildVerdict` whose `cloudbuild_status` reflects the
   subprocess outcome. (Tested with a stub cloudbuild script via
   `HEADWAY_CLOUDBUILD` pointing at a fixture.)
3. There is no code path that runs `cargo build`/`cargo install` locally. A grep
   in the test suite asserts the crate contains no local-cargo invocation in the
   build path; the only compiler invocation is via the cloudbuild subprocess.
4. When the configured cloudbuild script is absent or exits with the
   unreachable code, `build` returns `status: cloudbuild-unreachable`, emits a
   structured error to stderr, and exits non-zero — never silently builds local.
5. When `installed_version == source_head` (already fresh), `build` returns
   `status: no-op-fresh` without invoking cloudbuild, unless `--no-require-fresh`
   is passed (matches the agorabus reload `--require-fresh` precedent).
6. `BuildVerdict` records `version_before` and `version_after`; on a successful
   build against a fixture they differ, on a no-op they are equal.
7. `headway --version` and `headway build --help` work on the freshly-built
   binary; `cargo test` is green and `clippy` produces no new warnings over the
   repo baseline.
