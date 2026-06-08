# PRD: tend-gitignore

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/tend
Vision: visions/tend.md

## TL;DR

The artifact noise that `tend survey --classify` finds is the same set every
run — `.build-worktrees/`, `.cache/`, `__pycache__/`, `skill/state/`,
`.run-*/` — and the right fix is a one-time per-repo `.gitignore` entry, not a
perpetual hand-walk. `tend gitignore --plan` reads the classified survey and
prints, per repo, the `.gitignore` lines that would silence its `Artifact`-class
dirt (deduped against the repo's existing `.gitignore`), as a proposal — it
prints the diff, it does not write.

## Why this exists

Captured live during the dream that drafted this PRD (2026-06-08):

- autobuilder carries six untracked artifact paths
  (`.build-worktrees/`, `.cache/`, `.run-ambient/`, `skill/state/`,
  `skill/scripts/__pycache__/`, `skill/tests/__pycache__/`) that recur in every
  self-review. None is in its `.gitignore`; each shows up as "dirty" forever.
- The 2026-06-06/-07/-08 reflections re-list the same artifact-dominated dirty
  counts run after run. A `.gitignore` patch is a one-time edit that removes the
  recurring noise permanently — exactly the kind of structural fix self-review
  is told not to apply on its own ("do not auto-stash").
- Proposal-first is the established pattern for this kind of fix on this laptop:
  `tide-window`, `muster-reap`, and `recourse-contest` all print the command/
  patch and leave the apply to a human. tend-gitignore follows it.

## What this builds

`tend-gitignore` rust-extends `~/wintermute/tend` (created by
[[PRD-tend-survey]], classified by [[PRD-tend-classify]]). Do not start until
both exist and build. It is independent of [[PRD-tend-push]] (both consume the
classified survey; neither depends on the other). Conventions inherited: rustc
1.85, no let-chains, cloud-build-safe, `sigpipe::reset()` in `main()`.

### Modules

- `gitignore.rs`:
  - `propose(repo: &RepoState) -> GitignorePlan` — from the repo's
    `Artifact`-class dirt, derive a minimal, **deduped** set of `.gitignore`
    patterns. Directory artifacts become a trailing-slash entry
    (`.build-worktrees/`); file artifacts collapse to a glob when a clear family
    exists (`__pycache__/` rather than each `.pyc`). Reads the existing
    `.gitignore` (if any) and emits only patterns not already covered (honor
    existing dir/glob entries so a present `target/` isn't re-proposed).
  - `GitignorePlan { repo, existing_path, additions: Vec<String>, diff:
    String }` where `diff` is a unified-style preview of appending `additions`.
  - **`--write` is optional and gated.** Default is proposal-only (print the
    diff). If `--write` is passed, append `additions` to the repo's
    `.gitignore` (creating it if absent) and print what was written. Writing a
    tracked-config file is reversible/low-blast-radius (vision open question
    resolved toward an opt-in `--write`), but the default never writes.
- extend `render.rs`: human prints the per-repo proposed additions + a one-line
  "N repos, M total patterns"; `--json` emits `Vec<GitignorePlan>`.
- extend `main.rs`: `tend gitignore [--root <p>]... [--plan] [--write]`
  (`--plan` is the default and may be implicit).

### UX

```
$ tend gitignore
autobuilder  → .gitignore (+4)
  + .build-worktrees/
  + .cache/
  + .run-ambient/
  + skill/state/
  (skill/**/__pycache__/ already covered by existing *.pyc rule)
3 repos, 9 patterns proposed.  Run with --write to apply.
$ tend gitignore --write
autobuilder  → wrote 4 lines to .gitignore
```

### Dependencies

No new crates (reuses classify output + std fs). No network.

## Acceptance criteria

1. `propose` is unit-tested against a fixture `RepoState` whose `Artifact` dirt
   includes a directory (`.build-worktrees/`), a `__pycache__/` dir, and a
   `.cache/` dir, and produces minimal trailing-slash patterns with no
   duplicates.
2. Dedup honors the existing `.gitignore`: given a fixture repo whose
   `.gitignore` already contains `target/` and `*.pyc`, those (and paths they
   cover) are **omitted** from `additions`; only genuinely new patterns appear.
3. `Source`- and `Generated`-class dirt is **never** proposed for ignoring — a
   fixture repo with a `*.staged.md` WIP file produces no `.gitignore` line for
   it.
4. Default invocation (`tend gitignore`, no `--write`) makes **no filesystem
   change**: a test runs it against a temp repo and asserts the `.gitignore`
   file is byte-identical (or still absent) afterward, and the diff was printed.
5. `--write` appends exactly `additions` (idempotent: a second `--write` run
   adds nothing because the patterns now dedup against the just-written file),
   creating `.gitignore` if absent; asserted on a temp repo.
6. `tend gitignore --json` emits a stable `GitignorePlan[]` schema validated by
   a serde round-trip test.
7. No git subcommand is invoked by gitignore (it edits `.gitignore` text
   directly, not via git); the read-only-git guard from [[PRD-tend-survey]] AC7
   still passes.
8. `README.md` documents the proposal-first default, the `--write` opt-in and
   its idempotency, and that only `Artifact`-class dirt is ever proposed.
