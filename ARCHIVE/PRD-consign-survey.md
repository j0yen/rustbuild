# PRD: consign-survey — accurate fleet push-debt enumerator

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/consign
Vision: visions/consign.md

## TL;DR

Self-review reports "8 unpushed repos" every day, but that number is a
systematic undercount: it measures only `@{u}..HEAD` ahead-ness and silently
drops any repo whose branch has no upstream, or no remote at all. A live walk
on 2026-06-18 found 7 ahead, **5 unpushed-with-no-upstream**, **2 with no
remote**, and 1 diverged — work that exists only on this laptop's SSD.
consign-survey is the honest enumerator: it walks every fleet git repo and
classifies its push-debt into named buckets, emitting JSON for tooling and a
table for humans. It is the foundation of the consign vision (a NEW repo).

## Why this exists

Live evidence (commands reproducible today):

- `for d in ~/wintermute/*/; do git -C "$d" rev-list @{u}..HEAD --count; done`
  → **7** repos ahead of upstream (≈ the "8 unpushed" self-review reports).
- The same loop **skips** repos where `git rev-parse @{u}` fails (no upstream):
  `constellation-burst-builder`, `doxa`, `homeward`, `mqo-chart-caption` (4
  commits), `wm-skills` — **5** repos, invisible to the current counter.
- `git -C <repo> remote | wc -l == 0` → `colophon`, `headway` — **2** repos
  with no remote at all (freshly built 2026-06-18, never published).
- `git log --branches --not --remotes` across the fleet counts **29** repos
  holding ≥1 commit on no remote.
- Self-review journals 2026-06-14..18 repeat "8 unpushed" — the undercount is
  load-bearing: it makes a 14-to-29-repo durability gap look like a rounding
  error.

The `disk-critical` near-miss (2026-06-17, 99% full) underlines that this
laptop is a single point of failure; un-mirrored commits die with the disk.

## What this builds

NEW repo `~/wintermute/consign/` — `consign` binary + library crate.

- `consign survey [--root <dir>]... [--format json|table]` — default root
  `~/wintermute`. Walk each immediate child dir that is a git repo.
- Per-repo classification (a `RepoDebt` struct, library-exported):
  - `clean` — HEAD on a remote, nothing ahead.
  - `ahead(n)` — upstream set, `n = rev-list @{u}..HEAD`.
  - `no-upstream(n)` — has a remote, no upstream branch, `n` commits on no
    remote (`git log --branches --not --remotes` for the current branch).
  - `no-remote` — `git remote` empty.
  - `diverged(a/b)` — upstream set, both ahead `a` and behind `b` > 0.
- Each `RepoDebt` carries: path, current branch, default-branch guess, remote
  URL (or none), class, counts, and a `branch_is_default` bool (so downstream
  policy can flag worktree/feature branches).
- `--format json` emits a stable array; `table` prints an aligned summary with
  a totals footer (`N repos: X clean, Y ahead, Z no-upstream, …`).
- Pure read-only: no `git push`, no writes to any repo. `git` invoked via
  `std::process::Command` (no libgit2 dependency required).
- `sigpipe::reset()` as the first line of `main()` (see memory
  `self_sigpipe_panic_toolkit` — `consign survey | head` must not panic).

## Acceptance criteria

1. `consign survey --format json` on `~/wintermute` returns a JSON array with
   one object per immediate-child git repo; each object has `path`, `branch`,
   `class`, and class-appropriate counts.
2. The five known no-upstream repos (`constellation-burst-builder`, `doxa`,
   `homeward`, `mqo-chart-caption`, `wm-skills`) are classified `no-upstream`
   with a non-zero unpushed count — NOT `clean`. (This is the undercount fix;
   assert via a fixture repo set in tests, not the live fleet.)
3. A repo with no remote is classified `no-remote`; a repo with upstream and
   commits ahead is `ahead(n)` with correct `n`; a repo both ahead and behind
   is `diverged(a/b)` with correct `a` and `b`. (Test against constructed
   fixture repos.)
4. `consign survey --format table` prints an aligned table and a totals footer
   counting each class; the totals sum to the repo count.
5. `--root <dir>` overrides the default and may be passed multiple times;
   non-git child dirs are skipped silently; an unreadable root is a structured
   error, not a panic.
6. `consign survey | head -1` does not panic with a broken pipe (SIGPIPE reset).
7. `cargo test` green; `cargo build --release` produces `target/release/consign`;
   `consign --help` lists the `survey` subcommand. (Build via /cloudbuild per
   the standing hard rule — never local cargo.)
