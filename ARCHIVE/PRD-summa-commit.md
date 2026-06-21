# PRD-summa-commit

**Status:** Draft v0.1
**Vision:** visions/summa.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-summa-schema (the vault must be a git repo first)

## TL;DR

Karpathy's pattern only works if "the cost of maintenance is near zero" — and that
includes version history accruing without the human thinking about it. summa-schema
makes `~/Notes/` a git repo; this PRD makes commits *happen on their own*: a
`summa-commit.timer` that commits the vault on a cadence and a hook point for
post-ingest commits, classified **node-local** in `placement.toml` because the
vault is laptop/`carbon`-held canonical state.

## Why this exists

**Evidence (2026-06-21):**
- Karpathy: "treat the wiki as a git repository for version history and
  collaboration." A repo with no commit discipline accrues no history — the same
  decay that left `~/Notes/` un-versioned for 498 files in the first place. The
  human will not `git commit` a notes vault by hand; that is precisely the chore
  the system should absorb.
- The carbon vision just relocated **fleet** timers to the hub and kept
  **node-local** ones (`cloudbuild-watchdog`, `ctrace-reap`) on the laptop, encoding
  the split in `placement.toml`. The vault is single-node canonical state (like
  recall's store — see [[carbon]] open question on canonical-memory placement), so
  summa-commit is **node-local**: it commits the vault on the machine that holds it.
- `/summa` flows (ingest, ask) mutate the vault; without an automatic commit, a
  day's ingests sit uncommitted and a bad lint `--fix` is unrecoverable. A timer +
  an explicit `summa-commit` entrypoint the skill can call closes that gap.

## What this builds

Runs on: **this laptop** (`carbon`), the node that holds `~/Notes/`.

1. **`summa-commit.sh`** (in constellation's `cloud/scripts/` or a vault-local
   `bin/`) — `git -C ~/Notes add -A && git commit` with the Joe Yen identity and a
   message summarizing the diff (`summa: <N> files changed (<ingests> ingests,
   <answers> answers)` derived from `log.md` tail since last commit). No-op with
   exit 0 when the tree is clean. Never force-pushes; the vault may be local-only
   (no remote required — guard the push behind "remote exists").
2. **`summa-commit.timer` + `.service`** (systemd --user) — fire on a cadence
   (e.g. every 30 min during waking hours, or daily) to commit accrued vault
   changes. `ExecCondition=wm-node should-run summa-commit` as a belt-and-suspenders
   node guard (consistent with carbon's relocated units).
3. **placement.toml entry** — classify `summa-commit` as **node-local** under the
   `[timers]` (or a new `[vault]`) section, documenting that it runs only on the
   vault-holding node and must NOT be relocated to the hub.
4. **Idempotent install** — re-running enables nothing twice; a clean tree commit is
   a no-op; the timer is enabled only if not already.

## Acceptance criteria

1. `summa-commit.sh` on a dirty `~/Notes/` produces exactly one commit with a
   message summarizing the change counts; on a clean tree it exits 0 with no commit.
2. `systemctl --user is-enabled summa-commit.timer` → `enabled` on the laptop after
   install; `systemctl --user list-timers` shows it scheduled.
3. `placement.toml` classifies `summa-commit` as `node-local`; a comment documents
   that the vault is single-node canonical and the unit must not move to the hub.
4. The commit script guards its (optional) push behind a remote-exists check and
   never uses `--force`; with no remote configured it commits locally and exits 0.
5. Re-running the installer is a clean no-op (timer already enabled, no duplicate
   units), exit 0.
