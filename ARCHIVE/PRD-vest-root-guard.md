# PRD: vest-root-guard — adopt must never install outside the real prefix, and must clean what the tilde bug left

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/vest.md

## TL;DR

The `adopt-cron` run at 2026-06-13 02:42 installed `ac-judge` to
`/home/jsy/~/.local/bin/` — a directory whose first path component is a
**literal tilde** — because a `--root ~/.local` string reached `cargo
install` without shell expansion, and `cargo` does not expand `~`. It
created a junk prefix off PATH, left 4.7M of debris, and the install
then read back as FAILED. Nothing in `adopt` guards the install prefix,
and nothing detects or cleans the debris. `vest-root-guard` adds a
prefix validator that refuses any `--root` containing a literal `~` or
resolving outside `$HOME`, and an `adopt doctor` that detects the
mis-installed tree and (opt-in) removes only junk that duplicates a
correctly-installed twin.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The junk tree is real and measured.** `find /home/jsy/~` →
  `/home/jsy/~/.local/bin/ac-judge`, `/home/jsy/~/.local/.crates.toml`,
  `/home/jsy/~/.local/.crates2.json`. `du -sh /home/jsy/~` = **4.7M**.
  All entries timestamped `Jun 13 02:43`.
- **The cron log shows the mechanism.** `journalctl --user -u
  adopt-cron.service`: `Installing /home/jsy/~/.local/bin/ac-judge`
  followed by `warning: be sure to add /home/jsy/~/.local/bin to your
  PATH`. cargo treated `~/.local` as a *relative* path and joined it to
  the service cwd `/home/jsy`.
- **The failure cascades.** `adopt/src/apply.rs:248-265` calls
  `is_invokable(&artifact.bin)` after a successful `cargo install`;
  `is_invokable` (`scan.rs:95`) probes the *real* `~/.local/bin` and
  `~/.cargo/bin`. The binary is in neither (it's under `/home/jsy/~/…`),
  so the result is `FAILED: install exited 0 but --version / --help
  failed` — masking a path bug as a smoke failure.
- **The fix_cmd builder.** `adopt/src/scan.rs:308` builds
  `cargo install --force --path {repo} --root {home_dir().join(".local")}`.
  `home_dir()` (`scan.rs:18`) reads `$HOME`; when `$HOME` is unexpanded
  or contains a `~`, the resulting `--root` carries the literal tilde
  straight to cargo. There is no validation between build and exec.
- **No guard, no cleanup exist today.** `adopt --help` lists only
  `scan | apply | report`. Nothing inspects the resolved install prefix;
  nothing knows the `/home/jsy/~/` tree exists.

## What this builds

Extend the existing `adopt` crate (`~/wintermute/adopt`). No new crate.

### 1. Install-prefix guard (in `apply.rs`, before exec)

A `fn validate_root(root: &str) -> Result<PathBuf, AdoptError>` that runs
on the `--root <X>` token parsed from each `fix_cmd` before
`Command::new(prog)` executes:

- Reject if any path component is exactly `~` or begins with `~`
  (literal tilde — cargo will not expand it).
- Reject if `$HOME` is unset/empty (would silently relativize).
- Canonicalize against `$HOME`; reject if the resolved absolute path is
  not under `$HOME`.
- On rejection, the artifact's `ApplyResult` verdict becomes a new
  `ApplyOutcome::BadPrefix { resolved: String }` — never executed,
  never `cargo install`-ed into the void.

### 2. `adopt doctor` subcommand

`adopt doctor [--clean]`:

- Scans for adopt-created debris: any path under `$HOME` whose first
  component after `$HOME` is a literal `~` (e.g. `$HOME/~/.local/bin/*`),
  plus a `cargo`-style `.crates.toml` sibling confirming adopt/cargo
  authored it.
- For each junk binary, check whether a correctly-installed twin exists
  in the real `~/.local/bin` or `~/.cargo/bin`.
- Default (no flag): **report only** — print a table `JUNK_PATH | TWIN?
  | SIZE`, exit non-zero if any debris found (so the cron / self-review
  can detect it).
- `--clean`: remove only junk binaries that have a verified twin (and
  the now-empty junk prefix once drained). Junk with **no** twin is
  reported and left in place (it may be the only copy) — never blindly
  `rm`-ed. Print every removal.

### Shape

- `adopt/src/doctor.rs` (new module), wired into the clap command enum
  in `main.rs` alongside `scan|apply|report`.
- `validate_root` lives in `apply.rs` (or a small `prefix.rs`) and is
  unit-tested with both the literal-`~` case and the out-of-`$HOME`
  case.
- Reuse `home_dir()`, `local_bin()`, `cargo_bin()` from `scan.rs` rather
  than reimplementing path logic.
- No new heavy deps; `walkdir` is acceptable if not already present.

## Acceptance criteria

1. `validate_root("~/.local")` returns `Err` (literal tilde rejected);
   `validate_root("/home/jsy/.local")` returns `Ok` when `$HOME=/home/jsy`.
2. A unit test asserts that an artifact whose `fix_cmd` carries
   `--root ~/.local` yields `ApplyOutcome::BadPrefix` and that
   `Command` is **never** spawned for it (no `cargo install` runs).
3. `validate_root` rejects a root that canonicalizes outside `$HOME`
   (e.g. `--root /tmp/evil`) with `Err`.
4. `adopt doctor` (no flag) detects a fixture junk tree
   (`$TMPHOME/~/.local/bin/<fakebin>` + `.crates.toml`) and exits
   non-zero, printing the junk path and whether a twin exists.
5. `adopt doctor --clean` removes a junk binary **only** when a twin
   exists in the real `~/.local/bin`; a junk binary with no twin is
   reported and left on disk. Verified by a test with one of each.
6. `adopt doctor --clean` removes the junk prefix directory only after
   it is empty; never recursively `rm`s a non-empty tree containing
   twin-less binaries.
7. `cargo test` green; `cargo build --release` clean; `adopt doctor
   --help` and `adopt --help` list the new subcommand.

## Out of scope

- Autonomous `rm` of twin-less junk (stays user-gated; vision Open Q#1).
- Fixing `$HOME` resolution in the systemd unit itself (that is
  PRD-vest-path's concern).
- Incremental skip logic (PRD-vest-incremental).
