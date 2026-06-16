# PRD: roundtable-digest — so the wit is read, not just filed

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/roundtable
Vision: visions/roundtable.md
Depends-on: roundtable-session (produces the crowned bon mot + column)

## TL;DR

The whole point of the Round Table was that the wit was *read* — FPA printed the
best line in his syndicated column the next morning. Without that, roundtable
just relocates the creative wing's write-only problem: the bon mot gets crowned
and filed where no one looks. This PRD ships a SessionStart hook that surfaces
yesterday's crowned **bon mot** and today's column headline in one line at the
top of a session — reading only local state, offline-safe — so creativity on
this laptop finally becomes a conversation the user sees.

## Why this exists

Verified 2026-06-15: `~/wintermute/the-lunch/hooks/the-lunch-sessionstart.sh`
already establishes the exact pattern — a SessionStart hook that reads only local
state under `$XDG_STATE_HOME/the-lunch/<date>/`, makes no network/API calls, and
emits a single status line ("the table convened at noon — N dishes, M seated").
But it stops at "the table is set"; it never shows the *output* — the crowned
line, the column. roundtable-session produces both (the crowned bon mot in the
vicious-circle ledger; the column in the columns slot via conning-tower). This
PRD reads those and surfaces them, reusing the-lunch's hook conventions verbatim
so the two lines sit naturally together at session start.

## What this builds

Extend `~/wintermute/roundtable`:

- **`hooks/roundtable-sessionstart.sh`** — a SessionStart hook that:
  - Reads the most recent crowned line from the vicious-circle ledger (default
    `$XDG_DATA_HOME/vicious-circle/ledger.jsonl`) and the latest column headline
    from the columns slot — local files only, no network, no API.
  - Emits **one line**, e.g.:
    `roundtable · yesterday's bon mot: "<line>" — <persona>; column: <headline>`
  - Degrades gracefully: if no session has run yet, emits
    `roundtable · no lunch yet today — run: roundtable session --with-games`
    (mirrors the-lunch's "no lunch yet" fallback). Never errors the session
    start; `set -euo pipefail` but every read is guarded.
- **`roundtable digest [--date] [--format text|json]`** — the same surface as a
  CLI (the hook is a thin wrapper over it), so the user can ask for it on demand
  and other tools (daily-receipt) can consume the JSON.
- **install.sh wiring** — symlink the hook into `~/.claude/scripts/` and add the
  SessionStart entry to `~/.claude/settings.json` (jq + atomic rename, snapshot
  to `settings.json.bak.<ts>` first), exactly as the-lunch's installer does.
  Idempotent: re-running does not duplicate the settings entry.

MSRV 1.85 for the `digest` subcommand. The hook is shell; the digest logic is
Rust reusing roundtable's existing ledger/columns readers.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `roundtable digest --date <d>` with a fixture ledger + columns slot prints the
   crowned line, its persona, and the column headline (unit-tested against
   fixtures); `--format json` emits a documented shape consumable by
   daily-receipt.
3. `roundtable digest` with no session run for the date prints the "no lunch yet"
   fallback and exits 0 (a test asserts the fallback path).
4. `hooks/roundtable-sessionstart.sh` runs under `set -euo pipefail`, makes no
   network calls, emits exactly one line, and never exits non-zero even when the
   ledger/columns are missing (a test runs it against an empty `XDG` and asserts
   exit 0 + one line).
5. install.sh symlinks the hook into `~/.claude/scripts/` and adds the
   SessionStart settings entry via jq + atomic rename, snapshotting settings.json
   first; re-running does not duplicate the entry (idempotency test, or a
   `--dry-run` asserting the planned jq mutation is a no-op on the second pass).
6. The hook line and the-lunch's existing hook line coexist without conflict
   (both are single-line SessionStart emitters; a test or doc note confirms no
   ordering/overwrite issue).
