# PRD: chaff-policy — default-deny gate for safe untracking

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/chaff
Vision: visions/chaff.md

## TL;DR

chaff-repair will run `git rm -r --cached` against repos — a destructive
index operation. Before any of that, chaff needs a gate that decides, per
candidate path and per repo, whether untracking is *safe*. chaff-policy is
that default-deny gate, modeled on `consign-policy` and the trim vision's
policy layer: only recognized regenerable artifacts in recognized
regenerable directories are eligible, and several conditions HARD-exclude a
whole repo from any action regardless of config.

## Why this exists

Live evidence (2026-06-18) shows the failure modes a gate must prevent:

- `coda` tracks `target/.rustc_info.json` AND a real `src/` tree. A naive
  "untrack everything matching a glob" would be fine here, but
  `hold-guard`'s survey shows **6 `src/` files** mixed in with 203
  `target/` files — proving a repo can have legitimately-tracked files
  whose paths *brush* a loose pattern. The gate must untrack only paths
  inside a recognized regenerable *directory* (`target/`, `node_modules/`,
  `.venv/`, `dist/`, `__pycache__/`, `.pytest_cache/`) or with a
  regenerable *extension* (`.o`, `.rlib`, `.rmeta`) — never a `src/` path.
- `mqo-narrative-compose` is **diverged** (ahead 2 / behind 2 per the
  consign survey). Committing an untrack on a diverged/mid-rebase repo
  risks compounding a state that needs human triage. Such repos are
  HARD-excluded.
- The autobuilder-receipt-order note warns that `target/autobuilder/
  receipts/` is wiped by determinism runs — i.e. regenerable, so chaff
  *should* untrack it; the gate must NOT special-case it as precious.

The pattern of a default-deny policy crate that gates a destructive
sibling is already validated on this laptop: see consign-policy
(PRD-consign-policy.md) and the trim vision's trim-policy. chaff reuses the
shape so behavior is predictable across the fleet.

## What this builds

A `policy` module + `chaff policy` subcommand in `~/wintermute/chaff`,
consuming chaff-survey's `Vec<RepoChaff>`.

- `chaff policy [--format json|text]` — runs survey, then annotates each
  repo and each candidate path with an `eligible: bool` + `reason`.
- Eligibility rules (a path is eligible only if ALL hold):
  - it matches the regenerable pattern set (`chaff::patterns::REGENERABLE`
    from chaff-survey) **by directory or extension**, and
  - its top path component is NOT `src`, `tests`, `benches`, `examples`,
    `contrib`, `scripts`, or `.github` (defense in depth against an
    extension match on a real source file).
- Repo-level HARD exclusions (non-overridable by config) — if any holds,
  the whole repo is `eligible: false` with the reason:
  - detached HEAD, mid-rebase/merge/cherry-pick (presence of
    `.git/rebase-*`, `MERGE_HEAD`, `CHERRY_PICK_HEAD`), or diverged from
    upstream (`git rev-list --count --left-right @{u}...HEAD` both > 0);
  - no upstream AND no remote is fine to untrack locally, but the commit
    is created locally only (note in reason, still eligible);
  - a repo currently being written to by an active `/build` worktree
    (path under `.build-worktrees/`) — skip.
- Config overlay at `~/.config/chaff/policy.toml` (merged over built-in
  defaults) may ADD excluded path prefixes or excluded repos, but may NOT
  remove a HARD exclusion (mirrors consign-policy / trim-policy).
- Library API: `policy::evaluate(&[RepoChaff]) -> Vec<RepoPlan>` where
  `RepoPlan { repo, eligible, reason, paths: Vec<PathDecision> }`.

Deps: reuse chaff-survey's crate; `toml` for the overlay. No network.
MSRV 1.85, no let-chains. `sigpipe::reset()` already in `main`.

## Acceptance criteria

1. A `target/x.o` path in a repo on a clean branch is `eligible == true`.
2. A `src/main.rs` path (even if it somehow matched) is `eligible ==
   false` with reason naming the `src` exclusion.
3. A repo with `.git/MERGE_HEAD` present is wholly `eligible == false`
   with reason `mid-merge` (or equivalent), and none of its paths are
   eligible.
4. A repo diverged from upstream (left>0 AND right>0) is wholly
   `eligible == false` with reason naming divergence.
5. A repo under a `.build-worktrees/` path is excluded with reason naming
   the active-build exclusion.
6. `~/.config/chaff/policy.toml` adding an extra excluded repo marks that
   repo `eligible == false`; a config attempting to whitelist a `src/`
   path does NOT make it eligible (HARD exclusion wins).
7. `policy::evaluate` returns one `RepoPlan` per surveyed repo, and
   `chaff policy --format json` round-trips it as valid JSON.
8. `chaff policy` over a tree with zero eligible repos exits 0 and reports
   an empty plan (rest, not error).
