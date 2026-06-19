---
Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/chaff
build_version_bump: minor
---

# PRD-chaff-repair-push: push cleanup commits after fleet repair

**TL;DR:** `chaff repair` currently commits the git rm --cached cleanup but stops there — the fix lives only on this disk until someone manually pushes it. Add `--push` to `chaff repair` (and wire it into the cron's auto-repair path) so every cleanup commit automatically propagates to origin. Close the loop from "chaff finds junk" → "junk is untracked AND durably off the laptop."

## Background

chaff v0.5.0 ships survey + repair + policy + cron. The cron runs repair (when `~/.config/chaff/auto-repair` exists) and commits the cleanup with the Joe Yen identity. But it does not push. So:

- Live count as of 2026-06-18: 11 repos, 2,753 tracked artifacts (careen-ledger 1942, rosetta-prov 464, hold-guard 203, …).
- Each cron run would commit the cleanup but leave the pushed-bytes counter unchanged, and `consign drain` would still see these repos as "dirty → push needed" in the next scan.
- The repair is half-done: git index cleaned, but origin still shows the artifact blobs.

The fix is one flag and one post-repair step.

## Acceptance tests

### AC1 — repair + push in a normal repo
Run `chaff repair --no-dry-run --push` in a repo that has tracked artifacts, an upstream tracking branch, and no divergence. Verify:
- `RepairVerdict.committed == true`
- `RepairVerdict.push_verdict == "pushed"`
- `git log origin/HEAD..HEAD` shows 0 extra commits after the call (repair commit landed on remote).

### AC2 — repair + push in a no-upstream repo
Run `chaff repair --no-dry-run --push` in a repo with tracked artifacts but no `@{u}` tracking branch (e.g. a freshly-built j0yen repo with no `git push -u` yet). Verify:
- `RepairVerdict.committed == true` (cleanup commit applied locally)
- `RepairVerdict.push_verdict == "skipped-no-upstream"`
- No `git push` invocation attempted (no error in output).

### AC3 — repair + push in a diverged repo
Run `chaff repair --no-dry-run --push` in a repo that is both ahead and behind origin (diverged). Verify:
- `RepairVerdict.push_verdict == "skipped-diverged"`
- `RepairVerdict.committed` may be true (commit still applied locally — diverged repos still benefit from a clean index)
- No `git push` invocation attempted.

### AC4 — dry-run does not push
Run `chaff repair` (default, no `--no-dry-run`) with `--push`. Verify no commit is made, `push_verdict == "skipped-dry-run"`, and no `git push` call occurs.

### AC5 — backward compat: `--no-dry-run` without `--push` does not push
Run `chaff repair --no-dry-run` (no `--push` flag). Verify `push_verdict` is absent or `"none"`, and no `git push` is attempted. No regressions in existing tests.

### AC6 — cron auto-repair with push
With `~/.config/chaff/auto-repair` present, run `chaff-cron.sh`. Verify:
- cron invokes `chaff repair --no-dry-run --push`
- Journal records `repaired=N pushed=M errors=0`
- agorabus event includes `"pushed": M` field alongside existing `repaired`/`errors` fields.

## Implementation notes

- Add `push: bool` flag to the `Repair` subcommand in `main.rs` (alongside existing `no_dry_run` and `repo`).
- Add `push_verdict: Option<String>` field to `RepairVerdict` in `repair.rs`.
- After a successful commit in `repair_all` / the per-repo repair loop, if `--push` is set:
  - Check upstream: `git rev-parse --abbrev-ref --symbolic-full-name @{u}` — if empty, set `push_verdict = "skipped-no-upstream"`, continue.
  - Check diverged: `git rev-list --left-right --count @{u}...HEAD` — if behind > 0 AND ahead > 0, set `push_verdict = "skipped-diverged"`, continue.
  - Run `git push origin HEAD` — on exit 0 set `push_verdict = "pushed"`, on non-zero set `push_verdict = "push-failed:<stderr>"`.
- Update `chaff-cron.sh` (in `src/templates/` or wherever it's generated/installed):
  - Pass `--push` to the `chaff repair --no-dry-run` call when `AUTO_REPAIR_SENTINEL` is present.
  - Parse `pushed` count from JSON output; include in agorabus event and journal line.
- Do NOT update `~/.local/bin/chaff-cron.sh` directly — go through the normal extend path (update template or source, bump version, install via extend-handler.sh).

## Version bump
0.5.0 → 0.6.0 (minor: new `--push` flag + cron integration).
