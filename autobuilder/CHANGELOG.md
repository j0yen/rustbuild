# Changelog

## v0.6.1 — 2026-09-07

Pays down repo-wide gate debt that has blocked every rustbuild ship for
~20 hours: `autobuilder/Cargo.toml`'s 4 in-workspace path deps
(`session-trace-receipt`, `autobuilder-receipt`, `autobuilder-gate`,
`autobuilder-evolve-safety`) now carry explicit `version = "0.1.0"` so
`cargo deny check bans` stops flagging them as wildcard; `anyhow` is
bumped to 1.0.104 (lockfile-only) clearing the RUSTSEC UB advisory;
`acceptance_ac_x1_helps.rs` (AC-X1) now builds its own debug binaries via
`cargo build --bins` instead of hard-requiring pre-built `--release`
binaries, so `cargo test --workspace` passes on a fresh checkout with no
out-of-band build step; and `scripts/run-metrics.sh` now resolves
`audit-checks.sh` at its real path (`rustbuild`, not the stale
`autobuilder` skill name) with a schema-valid empty-findings fallback, so
`target/autobuilder/receipts/risk-gate.json` is never emitted as a 0-byte
file again. CI-checks (GitHub Actions never running any workflow for
this repo) is confirmed to be an account-level spending-limit/minutes-quota
setting, not a code defect — flagged for a human to check in `j0yen`'s
GitHub billing settings.

## v0.6.0 — 2026-09-06

`rollback-plan` now correctly infers the `redeploy-tag` model from an
`agent/deploy-manifest.toml` marker (AC1/AC3) and stays on the default
`revert-commits` model when no marker or explicit `rollback_model` is
present (AC3 regression proof, unweakened fleet default). Adds
`scripts/set-rollback-model.sh`, a standalone helper that idempotently
stamps `rollback_model` onto an existing `agent/intent-card.json` (AC5).
Evaluated `wm-node` and `adopt` against the deploy-tag-or-not question and
recorded both staying on `RevertCommits` (AC6) — neither ships as a
tag-redeployed service. Documents the schema and both opt-in paths in
README.md (AC4). mcphost itself has not yet been stamped with the marker
file or the `rollback_model` key — that onboarding is a tracked follow-on,
not part of this ship (AC1/AC2 prove the mechanism against fixtures, not
the live repo).

## v0.5.0 — 2026-09-06

ac-traceability: find this PRD and its nested-crate test coverage

Fixed the ac-traceability gate blocker (was 0/6 ACs traced, mis-picking a
stale legacy PRD file at repo root): added extended-gates.toml
(prd_path = PRD-rustbuild-hermetic-scope.md), committed the PRD text at
repo root, and widened collect_test_text() to walk the whole autobuilder/
tree (this repo has no root Cargo.toml; crates live nested under
autobuilder/crates/*). Verified twice (receipted): ac-traceability verdict
flipped from block to pass (8 AC ids, 0 untraced).

hermetic-build/cold-build-time/flake-audit's cargo_exit_code:101 is a
distinct, still-open extend-gate.sh/extended-receipts.sh --project
path-resolution defect (repo root has no Cargo.toml at any commit; cargo
needs --manifest-path autobuilder/Cargo.toml) — confirmed again this tick,
out of scope for this PRD's own code.

## v0.4.0 — 2026-09-05

The rollback-plan producer gains a `redeploy-tag` mode alongside the
existing `revert-commits` check: for a deploy-gated crate whose real
rollback is redeploying the previous tagged version (not reverting `--no-ff`
merge commits), the receipt now verifies base-tag existence, tag-lineage
contiguity, and a tagged/taggable HEAD instead of per-commit
git-revert-cleanliness — fixing the mcphost gate re-blocking on every
version-bump merge. Selected via `rollback_model` in
`agent/intent-card.json` or `agent/AUTOBUILDER_PROGRAM.md`, else inferred
from a deploy manifest, else defaults to `revert-commits` (unchanged).
Receipt schema bumps to v2 (additive); the gate accepts v1 and v2. Adds
`--explain` and `--migrate-note`.

## v0.3.0 — 2026-09-05

The hermetic-build producer now attributes outbound sockets to the build's
own process tree (spawned cargo + its descendants, sampled via /proc every
150ms) instead of diffing the whole machine's /proc/net/tcp before/after —
so a cohabiting process's connection (agorabus, tailscale, a Claude
session, snap refreshes) never false-blocks the gate, and a build-tree
connection that opens and closes mid-build is still caught. Receipt schema
bumps to v2 (structured pid/comm/remote per socket; a v1 receipt on disk
is still accepted during the transition). Loopback and unix-domain traffic
stay ignored by default; `--strict` also attributes loopback.

## v0.2.0 — 2026-05-30

Add `autobuilder publish` subcommand that codifies the manual Stage-6 publish pipeline
(README/LICENSE generation, branch normalize to `main`, repo create via `wm-publish`,
push via `wm-push`, `REPOS.md` update) into a deterministic, idempotent, dry-run-capable
command. Shells out to safety wrappers; never calls `gh repo create` or `git push` directly.
Writes a `publish-receipt/v1` receipt to `target/autobuilder/receipts/publish-receipt.json`.
ACs 1–9 hermetic-green; AC10 (live network) deferred per PRD.
