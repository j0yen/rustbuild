# PRD: trim-relief — apply the gentlest reversible lever

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/trim
Vision: visions/trim.md

## TL;DR

For each policy-eligible daemon, trim-relief picks the gentlest
effective lever — drop-cache IPC if supported, else `try-restart` for
self-healing units, else a `MemoryHigh=` transient cap — and applies it
**dry-run by default** (`--no-dry-run` to act), writing a reversible
receipt per action. It mirrors `adopt apply` / `consign drain`: it only
ever touches what trim-policy authorized. Extends the `trim` crate.

## Why this exists

Live evidence (this laptop, 2026-06-18):

- `homeward-embed.service` holds **476 MB in swap** while the box sits
  at `avg10=0.00` — i.e. it is idle and its pages are cold. A
  `try-restart` would return those 476 MB to the pool; the unit is
  systemd-managed and (per fleet convention) self-healing.
- `recalld.service` holds **237 MB swapped**; recall is known to
  cold-load its index (~500 ms per query, per
  `project_recall_in_repo_versions` / vision notes) — a candidate for
  a "shed cache" IPC rather than a full restart, if one exists.
- These are exactly the holders self-review calls "not actionable":
  the action is safe and reversible, it just needs a tool to take it.
- Precedent for the dry-run-by-default + receipt shape exists in
  `adopt apply` and `consign drain` (gossip 2026-06-18); reuse it so
  the UX is consistent across the reclaim fleet.

## What this builds

Extends the `trim` library + CLI.

**Library:**
- `relief::Lever { DropCache { how }, TryRestart, MemoryHighCap { bytes } }`.
- `relief::choose(holder, unit_meta) -> Lever`: prefer `DropCache` if
  the daemon advertises a shed mechanism (a known signal or
  agorabus/socket command — table of supported units; empty for now is
  fine, recalld's IPC is a vision open-question), else `TryRestart`
  for self-healing units, else `MemoryHighCap` sized to current RSS ×
  a factor.
- `relief::apply(lever, dry_run) -> Receipt`:
  - `TryRestart` → `systemctl --user try-restart <unit>` (try-restart,
    not restart: a no-op if the unit is not running — never *starts*
    a stopped unit).
  - `MemoryHighCap` → `systemctl --user set-property --runtime <unit>
    MemoryHigh=<bytes>` (runtime: not persisted to disk, so it
    evaporates on reboot — reversible by construction).
  - `DropCache` → send the unit's advertised shed command.
  - In `dry_run`, build and log the exact command(s) but execute
    nothing.
- `relief::Receipt { ts, unit, lever, before_swap_kb, before_rss_kb,
  dry_run, applied, reverted_by }` written under
  `~/.local/state/trim/receipts/`.

**CLI:**
- `trim relief` → dry-run: prints, per eligible unit, the lever it
  WOULD apply and the bytes it would reclaim. Default.
- `trim relief --no-dry-run` → applies, emits receipts.
- `trim relief --unit <name>` → restrict to one unit (still
  policy-gated).
- `trim relief --revert <receipt-id>` → undo a `MemoryHighCap` (clears
  the runtime property); restarts are noted as non-revertable (the
  process already cycled) but harmless.

## Acceptance criteria

1. `trim relief` (no flag) is dry-run: it executes zero
   `systemctl … start|stop|restart|set-property` calls (assert via a
   command-recorder shim in tests) and prints the intended lever +
   estimated reclaim per eligible unit.
2. `trim relief` only ever considers units that `trim policy` marks
   `Eligible`; a `Denied` unit (user-app, build, non-self-healing,
   mid-turn) is never acted on even with `--unit <that-unit>`
   (policy re-checked at act time, not just list time).
3. `relief::choose` prefers `TryRestart` for a self-healing daemon with
   no shed-IPC entry, and `MemoryHighCap` only when restart is
   unavailable; `DropCache` is selected only for units in the
   shed-capable table. Table-driven test.
4. `--no-dry-run` against an eligible idle daemon issues exactly the
   chosen lever's command and writes a `Receipt` capturing
   `before_swap_kb`/`before_rss_kb`, `applied=true`, and the exact
   command run.
5. `TryRestart` uses `try-restart` (never `restart`/`start`), so a
   stopped unit is never started by trim; `MemoryHighCap` uses
   `--runtime` so the cap does not persist across reboot. Asserted by
   inspecting the generated command in tests.
6. `trim relief --revert <id>` clears a runtime `MemoryHigh` property
   for that receipt's unit; reverting a `TryRestart` receipt is a
   logged no-op (not an error).
7. Every lever is reversible or self-healing and never targets a
   user-facing process; a test feeds a synthetic eligible set that
   includes a poisoned `user-app` entry and asserts it is skipped at
   apply time. `cargo test` green; build via /cloudbuild.
