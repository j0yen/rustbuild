# PRD: chaff-gitignore — synthesize a proper .gitignore for repos that lack one

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/chaff
Vision: visions/chaff.md

## TL;DR

Five `~/wintermute/*` repos track build junk because they have **no
`.gitignore` at all** — nothing ever stopped `git add` from swallowing
`target/`. chaff-gitignore is the additive, low-risk arm of chaff: it
synthesizes a complete, language-appropriate `.gitignore` from the repo's
type (Cargo.toml → Rust, package.json → node, pyproject.toml/setup.py →
python), so the next `git add` cannot reintroduce the junk. It writes the
file but does NOT untrack anything (that is chaff-repair's job), so it
needs no destructive gate and runs independent of chaff-policy.

## Why this exists

Live evidence (2026-06-18), strain 1 of the chaff disease:

- Repos with tracked junk AND no `.gitignore`: `careen-ledger` (1942
  tracked, `ls .gitignore` → No such file), `rosetta-prov` (464),
  `corpus-converge` (4), `headway` (2), `tether-gossip` (8). All are Rust
  crates (each has a `Cargo.toml`), so the correct ignore is well-known.
- Contrast strain 2 (`coda`, `hold-guard`): these DO have a `.gitignore`;
  their problem is pre-existing tracked files, which chaff-repair fixes.
  chaff-gitignore is specifically for the repos that have *nothing*.
- The autobuilder scaffold already SHOULD emit a `.gitignore`, but these
  repos predate or bypassed that — the survey is the ground truth of which
  ones slipped through. A one-time synthesis pass closes the gap and a
  later /build scaffold fix prevents new ones (out of scope here).

This is deliberately separated from chaff-repair because writing a
`.gitignore` is purely additive and safe, while `git rm --cached` is
destructive and gated. Keeping them apart means the safe fix can land and
run on a schedule even if the destructive arm is still under review.

## What this builds

A `gitignore` module + `chaff gitignore` subcommand in
`~/wintermute/chaff`, consuming chaff-survey results.

- `chaff gitignore [--write] [--format json|text]` — for each surveyed
  repo with `has_gitignore == false` AND `tracked_junk > 0` (or `--all`
  to cover clean repos too):
  - detect repo type by marker file: `Cargo.toml` → rust,
    `package.json` → node, `pyproject.toml`/`setup.py`/`requirements.txt`
    → python, else → `generic`;
  - render a `.gitignore` from a bundled template for that type (Rust:
    `/target`, `**/*.rs.bk`, `Cargo.lock` only for libs? — keep
    `Cargo.lock` UNignored, it's a binary crate by default; node:
    `node_modules/`, `dist/`, `.env`; python: `__pycache__/`, `.venv/`,
    `*.pyc`, `.pytest_cache/`);
  - default is dry-run (print the would-be file); `--write` creates
    `.gitignore` (refuses to overwrite an existing one — that's
    strain 2, not this PRD's job).
- The templates are embedded (`include_str!`) so the tool is offline and
  deterministic.
- Library API: `gitignore::plan(&[RepoChaff]) -> Vec<GitignorePlan>` and
  `gitignore::render(repo_type) -> String`.
- It does NOT stage or commit — leaving the new `.gitignore` as an
  untracked file for chaff-repair (or the user) to commit alongside the
  untracking. (Rationale: committing the ignore without untracking the
  already-tracked blobs would be a misleading no-op commit.)

Deps: reuse chaff-survey's crate; templates via `include_str!`. No
network. MSRV 1.85, no let-chains. `sigpipe::reset()` in `main`.

## Acceptance criteria

1. A Rust fixture repo (has `Cargo.toml`) with no `.gitignore` is
   detected as `repo_type == "rust"` and its rendered ignore contains a
   `/target` line.
2. A node fixture repo (has `package.json`) renders an ignore containing
   `node_modules/`.
3. A python fixture repo (has `pyproject.toml`) renders an ignore
   containing `__pycache__/` and `.venv/`.
4. `chaff gitignore` (no `--write`) does not create any file; the repo's
   `.gitignore` still does not exist afterward.
5. `chaff gitignore --write` on a repo with no `.gitignore` creates the
   file with the rendered content; running it again is a no-op that does
   NOT overwrite (reports `skipped: exists`).
6. A repo that already has a `.gitignore` is never written to by this
   command (it is strain 2, out of scope) — reported `skipped: exists`.
7. `--format json` emits one `GitignorePlan` per candidate repo with
   `{repo, repo_type, action: write|skip, reason}`.
8. The new `.gitignore` is left untracked (not `git add`ed) after
   `--write`.
