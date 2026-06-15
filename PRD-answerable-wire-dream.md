# PRD: answerable-wire-dream — make /dream report its generative autonomy

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/dream-skill
Vision: visions/answerable.md

## TL;DR

Wire `answerable record` into `/dream`'s autonomous output steps — the
Phase 5 commit + push of PRDs and vision docs, and the Phase 3 PRD-draft
itself — so the accountability ledger captures the *generative* half of
this agent's autonomy, not just the building half. `/dream` commits and
pushes to `~/wintermute/autobuilder` every productive pass and records
none of it.

## Why this exists (Phase 1 evidence, 2026-06-14)

- `grep -c answerable ~/.claude/skills/dream/SKILL.md` = **0** (same gap
  as `/build`; see PRD-answerable-wire-build).
- `/dream`'s Phase 5 is explicitly "Commit and push … `git push origin
  main`" with the Joe Yen identity, every pass that drafts. That is an
  autonomous push to a public-by-design repo — exactly the
  high-consequence action class the `answerable` ledger exists to record
  — and it is currently invisible to the ledger.
- The companion PRD-answerable-wire-build adds
  `scripts/answerable-emit.sh` to the build skill; this PRD reuses the
  same helper contract for the dream skill so both spines report through
  one mechanism.

This is a self-mod to the dream skill's own files (`build_into` is the
dream-skill repo). It depends on the `answerable-emit.sh` helper from
PRD-answerable-wire-build; if that helper is not yet present at a shared
path, this PRD ships its own copy under the dream skill's `scripts/`.

## What this builds

1. Ensure `scripts/answerable-emit.sh` is available to the dream skill
   (symlink to or copy of the build-skill helper; same
   `<kind> <target> <why> [reversible]` contract, same no-op-if-absent
   behavior).
2. Edits to `~/wintermute/dream-skill/SKILL.md`:
   - **Phase 3 (draft)**: after each `PRD-<slug>.md` is written,
     `answerable-emit.sh draft <prd-path> "vision-<slug>" true`.
   - **Phase 5 (commit + push)**: after the push succeeds,
     `answerable-emit.sh draft-push <repo> "dream: <N> PRDs + <M>
     visions from <seed>" true`.
   - One emit per pass for the push (not one per file) to keep the ledger
     legible; per-file emits only for the draft step.
3. Commit the self-mod with the Joe Yen identity and push via the dream
   skill's own distribution path (mirror whatever `/build`'s
   `self-push.sh` does; if the dream skill has no self-push helper, a
   plain guarded `git push origin main` from the skill repo is the
   fallback, logged as `dream-self-push-fallback`).

## Acceptance criteria

1. `scripts/answerable-emit.sh draft /tmp/PRD-x.md "vision-x" true`
   appends one line to the ledger with kind=draft, target=/tmp/PRD-x.md,
   reversible=true (verify via `answerable log --json | tail -1`).
2. With `answerable` absent from `$PATH`, the emit is a no-op exit 0 and
   a dream pass that calls it is unaffected (stripped-PATH test).
3. `dream/SKILL.md` references `answerable-emit` at both the Phase-3 draft
   step and the Phase-5 push step (grep ≥2 distinct locations).
4. The helper available to the dream skill produces byte-identical
   behavior to the build-skill helper for the same arguments (diff the
   two scripts, or confirm the symlink target).
5. The self-mod is committed with the Joe Yen identity and pushed clean.
