# PRD: tend-survey

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/tend
Vision: visions/tend.md

## TL;DR

`~/wintermute/` is a fleet of ~30 git repos, and every self-review hand-walks
them to count dirty working trees and unpushed commits, then dumps the list
under "Pending your call" with no tool behind it. `tend survey` is the
foundation of the tend vision: a read-only CLI that walks the fleet, parses each
repo's `git status --porcelain` plus its ahead/behind relationship to upstream,
and emits one structured report — per repo: dirty count, untracked paths,
modified paths, unpushed-commit count — as a human table and `--json`.

## Why this exists

Captured live during the dream that drafted this PRD (2026-06-08):

- The 2026-06-06, -07, and -08 self-reviews each list, under "Pending your
  call," a dirty-tree roll-call and an unpushed-commit list. The 2026-06-08
  journal records it verbatim: *"Dirty git trees: autobuilder(32), build-skill(16),
  keel(7), concord(2), …"* and *"Unpushed commits: build-skill(1), rollout(1),
  wintermute-desktop(2)"*. It is the most durable recurring "your call" item
  after the pacman queue (now owned by [[tide]]).
- A live survey this pass found **13** repos under `~/wintermute/` dirty or
  ahead — `autobuilder(8)`, `build-skill(16,+1)`, `concord(2)`, `coda(1)`,
  `dream-skill(1)`, `wintermute-almanac(1)`, `wintermute-reach(1)`,
  `wintermute-desktop(1,+2)`, `rollout(+1)`, `cradle-…-bak(3)`, others — counted
  by an ad-hoc shell loop with no reusable primitive behind it.
- `loom` (the only git-touching vision) is strictly /build-internal worktree
  integration, not fleet hygiene. Nothing surveys the fleet's working trees.

So a structural, honest survey of fleet git state is genuinely missing, and the
evidence motivating it is unambiguous and recurring.

## What this builds

A new Rust CLI `tend` at `~/wintermute/tend`, `cargo install`-able into
`~/.cargo/bin`, following the local-toolkit conventions
([[feedback_local_tools.md]]): `sigpipe::reset()` as the **first line of
`main()`** (it pipes to `head`/`jq` — see [[self_sigpipe_panic_toolkit]]); rustc
1.85, **no let-chains**; cloud-build-safe (no network during `cargo test`).

This PRD creates the workspace + binary and the `survey` subcommand only; later
PRDs (`tend-classify`, `tend-gitignore`, `tend-push`, `tend-report`)
rust-extend into this repo.

### Modules

- `model.rs` — core types reused by the whole vision:
  - `DirtStatus` = `Untracked | Modified | Added | Deleted | Renamed | Conflict`
    (mapped from porcelain XY codes).
  - `DirtPath { path: String, status: DirtStatus }`.
  - `RepoState { name, path, branch, dirty: Vec<DirtPath>, ahead: u32, behind:
    u32, upstream: Option<String> }`.
  - `FleetReport { root: String, repos: Vec<RepoState>, scanned: usize,
    dirty_repos: usize, ahead_repos: usize }`.
- `git.rs` — read-only git readers, each via `std::process::Command` with the
  repo as `-C`:
  - porcelain reader: `git -C <repo> status --porcelain=v1 -z` (NUL-delimited,
    robust to spaces/renames), parsed into `Vec<DirtPath>`.
  - ahead/behind: `git -C <repo> rev-list --left-right --count @{u}...HEAD`
    (no upstream → `upstream: None`, ahead/behind 0).
  - branch: `git -C <repo> symbolic-ref --short HEAD` (detached → branch =
    short SHA).
  - **No mutating git invocation anywhere** (no `add`/`commit`/`push`/`stash`/
    `clean`/`checkout`).
- `walk.rs` — enumerate immediate child dirs of the root that contain a `.git`
  (dir or file, to support worktrees/submodules). Root defaults to
  `~/wintermute`; overridable via `--root <path>` (repeatable) so the skill
  repos can be added later (see vision open question).
- `render.rs` — human table + `--json` (serde). Human form lists only
  dirty-or-ahead repos, one line each, with a trailing summary count; `--all`
  includes clean repos.
- `main.rs` — clap; `tend survey [--root <path>]... [--json] [--all]`.

### UX

```
$ tend survey
root: /home/jsy/wintermute  (28 repos, 13 need attention)
autobuilder        main   dirty 8   ahead 0
build-skill        main   dirty 16  ahead 1
wintermute-desktop main   dirty 1   ahead 2
rollout            main   dirty 0   ahead 1
…
$ tend survey --json | jq '.repos[] | select(.ahead > 0) | .name'
"build-skill"
"wintermute-desktop"
"rollout"
```

### Dependencies

`clap`, `serde`/`serde_json`, `sigpipe`. No network crates. git invoked via
`std::process::Command`.

## Acceptance criteria

1. `cargo build --release` produces a `tend` binary; `cargo install --path .`
   places it in `~/.cargo/bin`. `tend --help` lists the `survey` subcommand.
2. `sigpipe::reset()` is the first statement in `main()`; `tend survey --json |
   head -1` does not panic or print a `BrokenPipe` backtrace.
3. The porcelain parser is unit-tested against a **fixture** `git status
   --porcelain=v1 -z` byte string covering untracked, modified, staged-added,
   renamed (`R old -> new`), and a path containing a space, and maps each XY
   code to the correct `DirtStatus` with the correct path. Tests run fully
   offline (no real git invocation).
4. The ahead/behind parser is unit-tested against fixture `rev-list
   --left-right --count` output (`"2\t5"` → behind 2, ahead 5) and the
   no-upstream case (reader error/empty → `upstream: None`, ahead 0, behind 0).
5. `tend survey --json` emits a stable schema (`root`, `scanned`,
   `dirty_repos`, `ahead_repos`, `repos[]{name,path,branch,upstream,ahead,
   behind,dirty[]{path,status}}`) validated by a serde round-trip test.
6. The walker is unit-tested against a fixture directory layout (temp dir with
   child dirs, some containing a `.git` marker, one without) and returns exactly
   the repo-bearing children; `--root` overrides and accepts repeats.
7. No mutating git subcommand string appears in the binary: a test asserts the
   `git.rs` command set is a subset of {`status`, `rev-list`, `symbolic-ref`,
   `rev-parse`} and a grep-style guard in the test rejects `commit|push|add|
   stash|clean|checkout|reset`.
8. `README.md` documents the read-only guarantee, the default root and `--root`
   override, the `--json` schema, and notes rustc 1.85 / no let-chains and that
   no git working tree is ever mutated.
