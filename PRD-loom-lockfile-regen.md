# PRD-loom-lockfile-regen

Status: Draft v0.1

build_target: self-mod
build_priority: high
build_into: /home/jsy/.claude/skills/build
Vision: visions/loom.md

## TL;DR

`Cargo.lock` is the single most common reason parallel same-target branches
fail to integrate, and it is the **dumbest** one: the lockfile is a generated
artifact, but each worktree branch commits its own regenerated copy, so two
branches that each so much as `cargo build` produce divergent `Cargo.lock`
hunks that conflict or leave the primary tree dirty. This PRD stops treating
`Cargo.lock` as a hand-merged file: it gets a deterministic merge driver and is
**regenerated** at integrate time instead of merged.

## Why this exists

- gossip 2026-06-06T06:39:07Z: "`concord-bridge` + `concord-cruxes` → both exit
  3 (Cargo.lock dirty from the other); NEITHER integrated." Two branches blocked
  each other purely on the lockfile — zero real source conflict.
- `worktree-extend.sh:76-78` `die 3 "target tree dirty"` — a half-applied or
  divergent `Cargo.lock` in the primary `main` tree trips the dirty-tree guard
  before any merge is even attempted; both siblings deferred.
- `quicken-attest` (gossip / journal 2026-06-05) conflicted on
  "main.rs/Cargo.toml" — `Cargo.toml`+`Cargo.lock` churn rides along with every
  dep-adding subcommand.
- The lockfile carries no information a fresh `cargo generate-lockfile` can't
  reproduce from the merged `Cargo.toml`s — merging it line-by-line is pure
  accidental conflict.

## What this builds

Two pieces, both inside `~/wintermute/build-skill/scripts/`:

1. **A committed merge convention.** `worktree-extend.sh integrate` ensures the
   target repo has, at integrate time (idempotently, create-if-absent):
   - a `.gitattributes` line `Cargo.lock merge=ours` (the `ours` built-in driver
     keeps `main`'s side instead of conflicting), and
   - the repo-local config to enable it where needed
     (`git config merge.ours.driver true`), set on the repo before the merge.
   This makes `git merge` never *conflict* on `Cargo.lock` — it silently keeps
   one side, which is then corrected by step 2.
2. **Post-merge regeneration.** After a successful source merge but **before**
   the version-bump commit (between lines 84 and 86 of `worktree-extend.sh`),
   run `cargo generate-lockfile --offline` (fallback to `cargo build --offline
   --quiet` if `--offline` can't satisfy a new dep, then a note) in the repo so
   the committed `Cargo.lock` is canonical for the merged `Cargo.toml`. The
   existing `git add -A` (line 91) then stages the regenerated lockfile into the
   bump commit. If regeneration needs network (a genuinely new dependency not in
   the cache), skip-with-note (`last_error=lockfile-regen-needs-net`) rather than
   silently committing a stale lock — never silent-pass.

Constraints / shape:
- Workspaces: a member repo may have a single root `Cargo.lock`; run the regen
  at the repo root. Non-workspace single-crate repos behave the same.
- `--offline` first to keep this laptop's lock-step deterministic and avoid
  surprise network during the serial integrate (which holds the flock).
- Idempotent: re-running integrate on an already-converted repo re-applies the
  `.gitattributes` line only if missing; never duplicates it.
- Back-compat: repos that already integrate cleanly are unaffected — the
  `.gitattributes` + regen are no-ops when there's no lockfile churn.
- No behavioural change to `Cargo.toml` merges (those are real and stay
  conflict-able — only the *lock* is regenerated).

## Acceptance criteria

1. After integrate, the target repo contains `.gitattributes` with
   `Cargo.lock merge=ours` (created if absent, not duplicated if present);
   verified idempotent across two integrate runs.
2. A regression test with two branches that each add a **distinct** dependency
   (so each regenerates `Cargo.lock` differently) integrates both with **zero
   lockfile conflicts** and a final `Cargo.lock` that `cargo verify-project` /
   `cargo build --offline` accepts as consistent with the merged `Cargo.toml`.
3. The post-merge `cargo generate-lockfile --offline` runs after the source
   merge and before the bump commit; the regenerated `Cargo.lock` is included in
   the `$slug (parallel integrate)` commit (single commit, not a trailing one).
4. When regeneration genuinely requires network (new uncached dep), integrate
   does **not** commit a stale lockfile: it records
   `last_error=lockfile-regen-needs-net` to the sidecar and surfaces a note,
   leaving the source merge committed but flagged.
5. A repo with no `Cargo.lock` churn integrates byte-identically to today (regen
   is a no-op; `.gitattributes` add is the only new artifact, and only if the
   repo lacked it).
6. The /build SKILL documents that `Cargo.lock` is regenerated, not merged, and
   that branch agents need not (and should not) hand-resolve lockfile conflicts.
