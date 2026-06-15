# PRD: answerable-reconcile — the audit trail must audit itself

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/answerable
build_version_bump: minor
Vision: visions/answerable.md

## TL;DR

Add `answerable reconcile` — a subcommand that cross-checks the
self-reported ledger against ground truth the agent cannot edit out of
existence: git push history across `~/wintermute/*`, `gh repo list`
creation dates, and `~/dotfiles` git history for `CLAUDE_SELF.md` edits.
It reports **omissions** (a real autonomous action with no matching
ledger line) and **phantoms** (a ledger line with no matching real
action). An accountability ledger the agent writes to itself is only
trustworthy if something independent checks it for completeness.

## Why this exists (Phase 1 evidence, 2026-06-14)

The Fleet-1 `answerable` vision shipped completely (ledger, values-drift,
redline, consent-digest — all archived; repo at `~/wintermute/answerable`
v0.5.0). Its first open question was *"Who writes the ledger? … Self-report
first; reconcile later."* Live inspection shows the ledger is a fiction:

- `wc -l ~/.local/state/answerable/ledger.jsonl` = **49 lines, all
  self-test entries** (last five are duplicate `values-drift` lines from
  2026-06-13). No real action was ever recorded.
- On 2026-06-14 this agent pushed `homeward` **22 times** (v0.21.0 +
  v0.22.0) and pushed to GitHub. `git -C ~/wintermute/homeward log
  --since="1 day ago" --oneline | wc -l` = 22. The ledger shows none of it.

The agent can omit from a ledger it controls. The only fix is to verify
against records it does not control: 151 `~/wintermute/*` repos have
GitHub remotes with reflogs; `gh` is installed; `~/dotfiles` git-tracks
`CLAUDE_SELF.md` (9 edits in the last 3 days). Those are the ground truth.

## What this builds

A `reconcile` subcommand in the existing `answerable` crate
(`~/wintermute/answerable`), plus a `reconcile.rs` module.

- `answerable reconcile [--since <window>] [--window-mins <N>] [--json]`
  - Loads ledger entries in the window (reuse the existing `Ledger`
    loader and `parse_window`).
  - Collects ground-truth actions in the window from three probes:
    - **pushes**: for each `~/wintermute/*/.git` with a GitHub `origin`,
      `git log --since=<window> --format=%cI %H` on the pushed branch →
      one ground-truth `publish`/`push` action per commit reachable from
      `origin/HEAD`. (A commit reachable from the remote head = pushed.)
    - **repo-creates**: `gh repo list <owner> --json name,createdAt
      --limit 200`; any repo created inside the window → a `repo-create`
      ground-truth action.
    - **value-edits**: `git -C ~/dotfiles log --since=<window>
      --format=%cI -- .claude/CLAUDE_SELF.md` → a `self-value-edit`
      ground-truth action per commit.
  - **Matching**: a ground-truth action matches a ledger line when the
    `kind` is compatible AND the target string matches (substring, both
    directions) AND the timestamps are within `--window-mins` (default
    10). Unmatched ground truth = **omission**; unmatched ledger line =
    **phantom**.
  - Output: human table by default (sections: OMISSIONS, PHANTOMS,
    MATCHED count); `--json` emits
    `{"omissions":[...],"phantoms":[...],"matched":N}`.
- **Exit codes**: `0` = fully reconciled (no omissions, no phantoms);
  `1` = omissions found (the serious case); `2` = only phantoms found
  (lower severity — usually a dry-run or revert); `3` = probe/IO error.
- All ground-truth probes are **best-effort and non-fatal**: a missing
  `gh`, an offline network, or an absent `~/dotfiles` degrades that one
  probe to "skipped (reason)" in the output and never aborts the run.
  Reconcile over the probes that did succeed.
- Honor the existing global `--ledger <path>` flag so tests point at a
  fixture ledger.
- Respect the workspace clippy lints already in `answerable`'s
  `Cargo.toml` (`unwrap_used`/`expect_used`/`panic` = deny): no
  `unwrap`/`expect` in non-test code.

## Acceptance criteria

1. `answerable reconcile --json` against a fixture ledger and a fixture
   git repo with one pushed commit not in the ledger reports exactly one
   omission and exits `1` (unit/integration test with a tempdir git repo).
2. A ledger line whose target+kind+timestamp matches a ground-truth
   action (within the default 10-min window) is counted as `matched` and
   produces no omission and no phantom (test).
3. A ledger line with no corresponding ground-truth action is reported as
   a phantom; with only phantoms and no omissions, exit code is `2`
   (test).
4. A full match (every ground-truth action has a ledger line and vice
   versa) exits `0` and prints `MATCHED N` with empty OMISSIONS/PHANTOMS
   (test).
5. With `gh` absent or the network unavailable, the repo-create probe is
   skipped with a logged reason and the command still reconciles the
   push and value-edit probes — it does not exit non-zero solely because
   a probe was unavailable (test simulates by pointing at a no-gh PATH or
   a non-existent owner).
6. `cargo test --release` green; no new clippy `-D warnings`; version
   bumped to the next minor; `CHANGELOG.md` prepended with this TL;DR.
