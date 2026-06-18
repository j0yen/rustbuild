# PRD: consign-publish — mint a remote for never-published repos

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/consign
Vision: visions/consign.md

## TL;DR

Two fleet repos — `colophon` and `headway`, both built this morning — have **no
git remote at all**. /build's publish step never completed, so they exist only
on this disk. `consign drain` can push a repo that has a remote; it cannot
create one. consign-publish handles the `no-remote` class: mint
`j0yen/<name>` via `gh`, set `origin`, push the default branch, set upstream —
with an honest abort (never a half-created remote) on auth failure or name
collision.

## Why this exists

- Live evidence (2026-06-18): `git -C ~/wintermute/colophon remote` and
  `… headway remote` both return empty. Both were `cargo`-built and committed
  this morning but never got a GitHub repo. They are the canonical
  single-point-of-failure case the consign vision exists to close.
- `gh auth status` confirms an active `j0yen` login in the keyring, so minting
  a remote is actually possible from this box (no token plumbing needed).
- The fleet convention (memory `user_identity`, `project_autobuilder_repo_private`)
  is: shipped crates are public under `j0yen`; private work is private. Publish
  must honor consign-policy's verdict for visibility, not hardcode `--public`.

## What this builds

Extend `~/wintermute/consign` with a `publish` module + subcommand.

- `consign publish [--root <dir>]... [--dry-run] [--only <name>]...` — for each
  repo consign-policy marks `auto-ok` **and** `no-remote`:
  1. Derive the repo name (dir basename) and target `j0yen/<name>`.
  2. Determine visibility from consign-policy (`private-hold` would never reach
     here; `auto-ok` shipped crate ⇒ public unless a `.consign-private` marker
     or policy says private).
  3. `gh repo create j0yen/<name> --source <path> --remote origin
     --<public|private> --push` (or the equivalent create-then-push sequence).
  4. Set upstream for the pushed branch.
- `--dry-run` is the **default** (print the plan, touch nothing); a real run
  requires `--no-dry-run`, mirroring `adopt apply`'s dry-run-by-default safety.
- Honest failure: if `gh` is unauthenticated, if the repo name already exists
  remotely, or if the push fails, abort that repo with a structured error and a
  non-zero per-repo status — never leave a created-but-unpushed remote. Other
  repos in the batch continue (one bad repo doesn't sink the run).
- A receipt per repo: name, target, visibility, created bool, pushed bool,
  error (if any). `--format json` emits the receipt array.

## Acceptance criteria

1. `consign publish` (default) is a dry-run: it prints, for each `auto-ok`
   `no-remote` repo, the `j0yen/<name>` target and intended visibility, and
   creates/pushes nothing. (Assert no `gh repo create` is invoked — test via an
   injected/faked command runner.)
2. `consign publish --no-dry-run` on a fixture `no-remote` repo invokes the
   create+push sequence and sets `origin` + upstream; the receipt records
   `created=true, pushed=true`. (Test with a mocked `gh`/`git` runner — do NOT
   create real GitHub repos in CI; AC may be deferred/mocked.)
3. Visibility is taken from policy, not hardcoded: a repo policy marks private
   yields a `--private` create in the plan; a shipped crate yields `--public`.
4. `gh`-unauthenticated ⇒ the run aborts that repo with a structured error and
   non-zero per-repo status; no partial remote is left behind.
5. A name collision (target already exists remotely) ⇒ structured error, repo
   skipped, batch continues; the receipt records `created=false` with the
   collision reason.
6. Only `no-remote` + `auto-ok` repos are touched; `private-hold`,
   `manual-only`, and repos that already have a remote are ignored.
7. `cargo test` green (mocked runner); build via /cloudbuild;
   `consign publish --help` documents the dry-run default and `--no-dry-run`.
