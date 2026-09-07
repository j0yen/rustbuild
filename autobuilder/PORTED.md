# PORTED — this crate is frozen; do not add new autobuilder features here

**Status:** frozen / ported-from, as of 2026-09-07.

**Canonical source of truth:** `~/wintermute/autobuilder` (nested crate root
`autobuilder/`, invoke with `--project-root autobuilder`) —
`https://github.com/j0yen/autobuilder`, `autobuilder` package, now at
**v0.7.0** (deliberately higher than this crate's v0.6.1, so "the canonical
install" is checkable by version number alone).

**Why:** two divergent copies of the `autobuilder` companion binary existed
— this crate (v0.6.1, shipped the `RedeployTag` rollback model,
deploy-manifest/intent-card/`AUTOBUILDER_PROGRAM.md` model resolution, and
the tag-lineage verdict) and the split repo (v0.4.0, shipped
mechanical-commit classification, no redeploy-tag) — and the installed
`~/.local/bin/autobuilder` matched neither, so gate runs silently used
feature-incomplete rollback checks for hours. PRD-autobuilder-source-unify
ported this crate's v0.4.0→v0.6.1 rollback deltas (`RedeployTag` model,
model resolution, tag-lineage verdict, and their tests) into the split
repo, which already held the newer feature work and its own gate
scaffolding. See that PRD for the full ship note.

**Port commits (in the canonical repo, `~/wintermute/autobuilder`):**
- `a9e567c` — port v0.4.0→v0.6.1 rollback deltas into `src/rollback.rs`,
  `crates/gate` schema transition, ported tests.
- `3f775f2` — version bump to v0.7.0, ported `scripts/set-rollback-model.sh`.

**This crate's HEAD at freeze time:** `171b747`.

**What stays true here:** rustbuild's own harness and receipts continue to
reference this crate directly — its own gate is unaffected by the freeze,
and nothing here is deleted. What changes: no NEW autobuilder feature PRD
should target `~/wintermute/rustbuild/autobuilder` as `build_into` — route
those to the canonical crate instead (see `build-contract.md`'s autobuilder
routing note).
