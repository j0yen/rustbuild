# PRD: answerable-wire-build — make /build actually report to the ledger

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/build-skill
Vision: visions/answerable.md

## TL;DR

Wire `answerable record` into `/build`'s high-consequence steps —
publish, push, bump-version-commit, and self-mod — so the self-report
path the Fleet-1 `answerable` vision *assumed already existed* actually
fires. Today `/build` records nothing; the accountability ledger is empty
of real actions because no skill ever calls it. This PRD closes the
self-report half of "self-report first, reconcile later."

## Why this exists (Phase 1 evidence, 2026-06-14)

- `grep -c answerable ~/.claude/skills/build/SKILL.md` = **0**. The
  Fleet-1 vision said "`/build` + `/dream` call `record` inline" — they
  never did.
- On 2026-06-14 `/build` shipped `homeward` v0.21.0 and v0.22.0, made
  several commits, and pushed 22 times to `j0yen/homeward`. The ledger at
  `~/.local/state/answerable/ledger.jsonl` recorded **zero** of these.
- The `answerable` binary is installed (`~/.local/bin/answerable`, v0.5.0)
  and its `record` subcommand works:
  `answerable record --kind publish --target <repo> --why <reason>
  --reversible false`.

The capability exists; the wiring is missing. This is a self-mod to the
build skill's own files (`build_into` is the build-skill repo).

## What this builds

1. A small idempotent helper at
   `~/wintermute/build-skill/scripts/answerable-emit.sh`:
   - Usage: `answerable-emit.sh <kind> <target> <why> [reversible]`.
   - Runs `answerable record` if `answerable` is on `$PATH`; **no-op
     exit 0** with a one-line stderr note if it is absent (never blocks a
     build tick).
   - `set -uo pipefail`; safe to call many times per tick.
2. Edits to `~/wintermute/build-skill/SKILL.md` documenting that the
   following Phase-4 / Phase-6 actions MUST call `answerable-emit.sh` as
   their final sub-step (after the action succeeds):
   - **publish** (new-repo): `answerable-emit.sh publish
     joeyen-atscale/<slug> "PRD-<slug>: <one-line>" false`
   - **push** (rust-extend): `answerable-emit.sh push <repo> "v<ver> —
     <one-line>" false`
   - **bump-version & commit**: folded into the push emit (one line per
     shipped version is enough; avoid double-recording).
   - **self-mod distribution** (`self-push.sh`): `answerable-emit.sh
     self-edit <file> "<why>" false`
   - **Phase 6 reflect** (drafted follow-on PRD commit+push):
     `answerable-emit.sh draft <prd-path> "Phase 6 reflect" true`
3. After committing the self-mod with the Joe Yen identity, run
   `scripts/self-push.sh` (the build-skill self-distribution path).

## Acceptance criteria

1. `scripts/answerable-emit.sh publish j0yen/test "smoke" false` appends
   exactly one line to the ledger (verify with `answerable log --json |
   tail -1` showing kind=publish, target=j0yen/test) when `answerable` is
   on `$PATH`.
2. With `answerable` removed from `$PATH`, `answerable-emit.sh` exits `0`
   and writes nothing, emitting a single stderr note — a build tick that
   calls it is unaffected (test by running with a stripped PATH).
3. `SKILL.md` contains explicit "call `answerable-emit.sh`" instructions
   at the publish, push, self-mod, and Phase-6 steps (grep the doc for
   `answerable-emit` at ≥4 distinct step locations).
4. The helper is idempotent in the sense that calling it twice records
   two honest lines (it is append-only by design) and never corrupts the
   ledger JSONL (each line parses as JSON).
5. The self-mod is committed with the Joe Yen identity and
   `scripts/self-push.sh` runs clean (fast-forward or no-op).
