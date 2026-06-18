# PRD: chaff-repair — untrack build artifacts and commit the deletion

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/chaff
Vision: visions/chaff.md

## TL;DR

chaff-survey finds the tracked junk and chaff-policy says which is safe to
remove; chaff-repair is the arm that ACTS. For each policy-approved repo it
`git rm -r --cached`s the eligible junk, ensures the matching ignore line
exists, stages the new/updated `.gitignore`, and commits the deletion with
the Joe Yen identity — dry-run by default, one repo at a time, emitting a
structured verdict per repo. It mirrors `adopt apply` and `consign drain`:
plan-by-default, `--no-dry-run` to mutate, reversible-by-design (the blobs
remain in history; only the index stops tracking them).

## Why this exists

Live evidence (2026-06-18):

- `careen-ledger` carries **1942** tracked `target/` files whose on-disk
  copies were already deleted by the disk fleet — its `git status` is 1942
  ` D target/...` lines. The correct resolution is to commit those
  deletions with `target/` newly ignored, collapsing 1942 phantom dirty
  entries to zero. No tool does this today; it is hand-work the
  self-review explicitly defers each day.
- `coda` proves the footgun chaff-repair must handle: `.gitignore` already
  says `/target/`, yet `target/.rustc_info.json` is tracked. Adding the
  ignore line is NOT enough — repair MUST `git rm --cached` the
  already-tracked paths for the ignore to take effect.
- `mqo-narrative-compose` is diverged and MUST be skipped — which is why
  repair consumes chaff-policy's gate rather than re-deciding safety.

The plan-by-default, `--no-dry-run`-to-apply, structured-verdict shape is
validated on this laptop by `adopt apply`, `consign drain`, and the trim
vision's trim-relief. chaff-repair reuses it so its blast radius is legible.

## What this builds

A `repair` module + `chaff repair` subcommand in `~/wintermute/chaff`,
consuming chaff-policy's `Vec<RepoPlan>`.

- `chaff repair [--no-dry-run] [--repo <name>] [--format json|text]` —
  for each `RepoPlan` with `eligible == true`, in series:
  - compute the eligible path set (policy's `PathDecision`s);
  - if the repo has no `.gitignore`, delegate to `chaff::gitignore::render`
    + write it (reuse chaff-gitignore); if it has one, append any missing
    pattern line(s) needed to cover the untracked dirs (idempotent —
    never duplicate an existing line);
  - run `git rm -r --cached --quiet -- <paths>` (paths may already be
    deleted on disk; `--cached` only touches the index, so this is fine);
  - `git add .gitignore`;
  - commit:
    `git -c user.email=jyen.tech@gmail.com -c user.name="Joe Yen" commit
    -m "chaff: stop tracking build artifacts (<N> files)"`;
  - emit a verdict `{repo, files_untracked, gitignore_action, committed,
    commit_sha, status}`.
- Dry-run (default) prints the exact `git rm` path count and the commit
  message it WOULD make, mutating nothing.
- One repo at a time (no parallel index mutation). On any git error for a
  repo, record `status: failed` with the stderr and continue to the next
  repo (do not abort the batch).
- Does NOT push (that is `consign drain`'s job) and does NOT rewrite
  history (the blobs stay in `.git`; see vision open question — a
  history-purge PRD is deferred).

Deps: reuse chaff-survey/policy/gitignore crates. No network. MSRV 1.85,
no let-chains. `sigpipe::reset()` in `main`. Honor the recall note that
/build branches must route cargo through /cloudbuild — this is a local
git op, no cargo at runtime.

## Acceptance criteria

1. Dry-run (default) over a fixture repo tracking `target/x.o` reports
   `files_untracked: 1` and `committed: false`, and the file is STILL
   tracked afterward (`git ls-files` still lists it).
2. `--no-dry-run` on that repo runs `git rm --cached`, commits, and
   afterward `git ls-files` no longer lists `target/x.o`; the blob still
   exists in `git cat-file` history (forward-only, not purged).
3. On a repo whose `.gitignore` already covers `target/` (the coda case),
   repair untracks the tracked `target/` paths and does NOT duplicate the
   existing ignore line.
4. On a repo with no `.gitignore`, repair creates one (via chaff-gitignore
   render) AND untracks, in a single commit.
5. The commit author is `Joe Yen <jyen.tech@gmail.com>` (verify
   `git log -1 --format='%an <%ae>'`).
6. A repo marked `eligible: false` by chaff-policy (e.g. diverged) is
   skipped with `status: skipped` and reason carried from policy; its
   index is untouched.
7. `--repo <name>` restricts action to a single named repo.
8. A git failure on one repo yields `status: failed` for that repo and
   does NOT prevent the next repo from being processed; the command's exit
   code is non-zero iff at least one eligible repo failed.
