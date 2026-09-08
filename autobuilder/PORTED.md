# PORTED — this crate is frozen; do not add new autobuilder features here

**Status:** frozen / ported-from, as of 2026-09-07 (updated 2026-09-08).

**Canonical source of truth:** `~/wintermute/autobuilder` (nested crate root
`autobuilder/`, invoke with `--project-root autobuilder`) —
`https://github.com/j0yen/autobuilder`, `autobuilder` package, now at
**v0.7.1** (deliberately higher than this crate's v0.6.2, so "the canonical
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
- `54ab8f7` (merged as `00ce607`) — v0.7.1: ported this crate's
  hermetic-build v2 schema/transition (`crates/gate`,
  `crates/gate/tests/hermetic_scope_ac6_v1_transition.rs`, PRD
  PRD-rustbuild-hermetic-scope) and the `autobuilder-extended-gates`
  0.1.0→0.1.1 producer deltas (determinism/cold-build-time isolated
  `CARGO_TARGET_DIR`, hermetic-build per-socket attribution + `--strict`,
  ac-traceability nested-crate + numbered-heading parsing, secrets-scan
  allowlist, mutation-kill comment-aware scanner) into the split repo —
  operator-notes on PRD-autobuilder-source-unify, 2026-09-08.

**This crate's HEAD at freeze time:** `171b747` (v0.7.0 freeze);
`~/wintermute/rustbuild/autobuilder`'s extended-gates lineage at the
0.1.1 port's source was this crate's own HEAD as of 2026-09-08, ported
byte-for-byte into the canonical repo (see the port commit above).

**What stays true here:** rustbuild's own harness and receipts continue to
reference this crate directly — its own gate is unaffected by the freeze,
and nothing here is deleted. What changes: no NEW autobuilder feature PRD
should target `~/wintermute/rustbuild/autobuilder` as `build_into` — route
those to the canonical crate instead (see `build-contract.md`'s autobuilder
routing note).
