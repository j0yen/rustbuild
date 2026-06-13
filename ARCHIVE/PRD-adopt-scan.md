# PRD: adopt-scan — detect shipped artifacts that never entered the live system

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/adopt
Vision: visions/docket.md (Fleet 2 — the adoption forcing function)

## TL;DR

`/build` ships a repo by build → commit → push. But **adoption** —
installing the binary so it lands on `$PATH` and becomes invokable — is
a separate step that frequently never runs, and nothing on the laptop
verifies it. The result: tools that were fully built, tested, and
committed sit unused for days. `adopt scan` is the read-only detector
that, per shipped artifact, answers the one question no existing tool
answers: **did this actually get adopted into the live system, and if
not, what is the exact command to adopt it?**

## Why this exists (Phase 1 evidence, 2026-06-12)

Measured live this session:

- **`rollout` is the proof.** The tool whose entire job is bringing
  stale fleet binaries current was built `2026-06-03 09:45`
  (`~/wintermute/rollout/target/release/rollout` present, 2.9 MB),
  committed (HEAD `010a35c`), has **1 unpushed commit**, and is **not on
  PATH** — `command -v rollout` → not found; absent from both
  `~/.local/bin/rollout` and `~/.cargo/bin/rollout`. Unadopted for **9
  days**. The fixer is itself the thing that never got fixed.
- **The staleness it would fix is live.** `binstale scan` reports
  `wm-audio` (pid 632), `wm-dialog` (634), `wm-tts` (636), `wm-stt`
  (663) all `behind-head` right now. `rollout apply` would bring them
  current — but `rollout` isn't installed to run.
- **`warden` is inert.** `command -v warden` → not found (only
  `bpolicy` is installed); flagged inert across every self-review since
  it shipped.
- **The existing detector cannot see this class.** `binstale check
  <PID>` operates on a *running process* and exits 2 ("process not
  found") when the process doesn't exist. It structurally cannot flag a
  CLI that was *never installed and therefore never runs*. vigil/binstale
  cover a daemon executing stale bytes; **nothing** covers an artifact
  that never entered the live system at all. (Verified: `binstale check
  --help` — "Check the staleness verdict for a single process by PID.")
- **Self-review re-flags this by hand every run.** Reflective memories
  and journals on 2026-06-08/09/10/11/12 all carry
  "fleet-binary-staleness wm-audio/dialog/tts/stt (rollout plan
  needed)" and "binstale never installed" under Pending — prose, not a
  structured probe.
- **Three prior `/dream` passes named the class and deferred it.**
  Gossip 2026-06-06 03:05 / 07:45 and 2026-06-08 05:25 each concluded
  these are "install/arming ACTIONS … a forcing-function PRD under
  docket … Reconsider if they keep aging." Six days later the evidence
  above is unchanged. This PRD is that reconsideration.

## What this builds

A new `rust-cli` crate at `~/wintermute/adopt/`, installed to
`~/.local/bin/adopt`. Single read-only subcommand for v1: `adopt scan`.

### Artifact enumeration

The set of "shipped artifacts" the scan considers:

1. **wintermute repos that declare a binary.** Walk `~/wintermute/*/` for
   a `Cargo.toml` whose `[[bin]]` (or `[package] default-run`, or a
   `src/main.rs` with `[package].name`) names an executable. Each such
   `(repo, bin-name)` is a candidate.
2. **Autobuilder/build shipped slugs (best-effort).** If
   `~/.claude/skills/build/state/manifest.json` is readable, include
   slugs marked shipped whose repo resolves under `~/wintermute/`. This
   is additive — a repo found by (1) is not double-counted.

Enumeration is best-effort and must never hard-fail: an unreadable
manifest, a malformed `Cargo.toml`, or a missing repo is skipped with a
note, not an error.

### Per-artifact verdict

For each candidate `(repo, bin-name)`:

- `not-a-bin` — the repo declares no executable (library-only). Reported
  only with `--all`; excluded from the default actionable set.
- `not-installed` — no `bin-name` on `$PATH` (checked via PATH walk,
  not a shell builtin) **and** no `~/.local/bin/<bin>` / `~/.cargo/bin/<bin>`.
  This is the `rollout` case.
- `installed-stale` — `bin-name` is installed, but the installed file's
  mtime / provfs `user.prov.ts` predates the repo's newest `src/`
  commit (`git -C <repo> log -1 --format=%ct -- src/`). The on-disk
  binary is older than its source. (Distinct from a *running* daemon
  being stale — that stays binstale's job; if the artifact is a known
  daemon and `binstale` is on PATH, `adopt` annotates the verdict with
  binstale's running-process verdict but does not re-derive it.)
- `installed-current` — installed and not older than HEAD source.

### The fix line

Every `not-installed` / `installed-stale` verdict carries a concrete,
copy-pasteable remedy: `cargo install --path <repo> --root ~/.local`
(matching the laptop's `~/.local/bin` convention, not `~/.cargo/bin`).
For artifacts that back a systemd-user unit (a daemon), the remedy is
instead the `rollout install`/vigil path, and the verdict is tagged
`daemon: true` so downstream tools (adopt-apply) know to defer.

### Output

- `adopt scan` (default) — table: artifact, verdict, installed-path,
  age-vs-HEAD, fix.
- `adopt scan --format json` — machine-parseable array of
  `{repo, bin, verdict, installed_path, is_daemon, source_commit_ts,
  installed_ts, fix_cmd}`; consumed by adopt-docket-report and
  adopt-apply.
- `adopt scan --all` — include `installed-current` and `not-a-bin`.
- `adopt scan --match <regex>` — limit to artifacts whose bin name
  matches.

### Dependencies / shape

- Reuse the laptop conventions: SIGPIPE reset as the first line of
  `main()` (per the `sigpipe::reset()` toolkit rule — `adopt scan |
  head` must not panic). `clap` for args, `serde_json` for JSON.
- Read provfs xattrs via `getfattr`-equivalent (`xattr`/`std::fs`),
  degrading to mtime when `user.prov.ts` is absent (per binstale's same
  documented fallback — `cargo install` to `~/.cargo/bin` may not pass
  through the `install(1)` provfs codepath).
- No network, no mutation, no daemon. Pure inspection.
- MSRV 1.85, no let-chains (matches the lib-crate toolchain rule).

## Acceptance criteria

1. `adopt scan --format json` emits a JSON array; each element has at
   minimum `repo`, `bin`, `verdict`, `is_daemon`, and `fix_cmd` keys.
2. Given a wintermute repo that declares a `[[bin]]` whose binary is
   **absent** from `$PATH`, `~/.local/bin`, and `~/.cargo/bin`, the scan
   reports that artifact with verdict `not-installed` and a `fix_cmd` of
   the form `cargo install --path <repo> --root ~/.local`. (Test with a
   temp repo fixture; the live `rollout` case is the real-world example,
   not a test dependency.)
3. Given a binary present on `$PATH` whose installed mtime is **newer**
   than the repo's newest `src/` commit, the scan reports
   `installed-current`.
4. Given a binary present on `$PATH` whose installed mtime is **older**
   than the repo's newest `src/` commit, the scan reports
   `installed-stale`.
5. A library-only repo (no `[[bin]]`, no `default-run`) is reported
   `not-a-bin` and is **excluded** from default output (shown only under
   `--all`).
6. An unreadable build manifest, a malformed `Cargo.toml`, or a missing
   repo directory is skipped without aborting the scan (exit 0, the
   other artifacts still reported).
7. `adopt scan --format json | head -1` does **not** panic with a
   SIGPIPE/BrokenPipe error (SIGPIPE reset verified).
8. `adopt scan --match '^wm-'` restricts output to artifacts whose bin
   name starts with `wm-`.
9. When `binstale` is on `$PATH` and the artifact is a known daemon, the
   JSON element carries `is_daemon: true`; when `binstale` is absent the
   scan still completes (daemon detection degrades to the systemd-unit
   `ExecStart` lookup) and never hard-fails.
10. `adopt --version` and `adopt --help` exit 0 (the artifact is itself
    adoptable and self-describing — the dogfooding criterion).
