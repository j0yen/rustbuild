# PRD: adopt-apply — close the loop, don't just track it

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/docket.md (Fleet 2 — the adoption forcing function)
deferred_acs: [2, 9]
mock_justifications:
  2: "AC2 requires cargo install in tests — slow integration test deferred; apply logic is covered by AC1/AC3/AC6/AC7/AC8"
  9: "AC9 is a live real-world validation (rollout on PATH after apply); not autonomously testable"

## TL;DR

`adopt scan` detects and `adopt report` tracks unadopted artifacts — but
tracking a 9-day-unadopted `rollout` for a 10th day is not the goal.
`adopt apply` is the mutating half: it installs the safe,
**non-daemon** CLIs that the scan found `not-installed`, one at a time,
idempotently, with `--dry-run` as the default posture. Daemons are out
of scope by default and delegated to `rollout`/vigil — adopt never
bounces a live voice daemon. This is the component that turns the whole
Fleet-2 ledger from a watcher into a closer.

## Why this exists (Phase 1 evidence, 2026-06-12)

- **Tracking alone does not close.** The same staleness/never-installed
  items have been *noticed* every self-review for a week; what's missing
  is the safe, autonomous *action*. The standing user instruction is
  "always auto-publish, always commit, no caps" (memory
  `feedback_always_commit_push`), which leans toward letting a
  reversible, low-blast-radius install run autonomously.
- **The action is genuinely safe for the CLI case.** Installing a plain
  CLI is `cargo install --path <repo> --root ~/.local` — reversible
  (delete the binary), no live-conversation risk, no systemd unit to
  bounce. This is categorically different from the daemon case vigil
  guards so carefully (`rollout` serializes + polls peers precisely
  because dropping a voice daemon mid-turn is the real cost).
- **No overlap with vigil.** `vigil-install-restart` (`rollout install
  <binary> --dest`) installs a freshly-built binary and restarts its
  systemd unit — it is for *daemons running stale bytes*. `adopt apply`
  is for *plain CLIs that never entered PATH and have no unit*. They are
  complementary halves of the same "close the loop at the install site"
  goal; adopt explicitly delegates the daemon case to rollout.

## What this builds

A new subcommand `adopt apply` in the existing `adopt` crate.

### Behavior

- `adopt apply` (default = `--dry-run`) prints the exact install
  commands it *would* run for each `not-installed` **non-daemon**
  artifact, and touches nothing.
- `adopt apply --execute` performs the installs, **strictly one at a
  time**, each via the artifact's `fix_cmd`
  (`cargo install --path <repo> --root ~/.local`). After each install it
  re-checks that the binary is now on `$PATH` and invokable
  (`<bin> --version` or `--help` exits 0); emits a per-artifact verdict
  `{bin, installed, invokable, elapsed}`. Stops the run on the first
  install that fails to land (does not cascade) and exits non-zero.
- **Daemons are excluded by default.** An artifact tagged `is_daemon`
  is skipped with a note pointing at `rollout install` / vigil. The
  `--with-daemons` flag opts in and, for each daemon, **shells to
  `rollout install`** rather than re-implementing the guarded restart —
  if `rollout` is not on PATH, the daemon is skipped with the note "run
  `adopt apply --execute` for rollout first." (The bootstrap ordering:
  installing `rollout` is itself a non-daemon CLI install adopt can do
  unaided, which then unlocks the daemon path.)
- **Idempotent.** Re-running after a successful install reports
  `installed-current` and does nothing.
- `--only <bin>` restricts apply to a single named artifact (e.g.
  `adopt apply --execute --only rollout`).

### Safety / shape

- `--execute` is required for any mutation; the bare command is a
  dry-run. No `--execute`, no install.
- One install in flight at a time — never parallel `cargo install`
  (cargo target-dir / registry-lock contention, and it keeps the
  verdict ordering legible).
- Subprocess args passed as a discrete vector, never a shell string
  (same injection guard as adopt-report).
- SIGPIPE-safe. MSRV 1.85, no let-chains.

## Acceptance criteria

1. `adopt apply` with no flags is a **dry-run**: it prints the install
   commands for `not-installed` non-daemon artifacts and executes
   nothing (no binary appears on PATH as a result).
2. `adopt apply --execute --only <bin>` for a `not-installed` non-daemon
   artifact runs exactly that artifact's `fix_cmd`, then verifies the
   binary is on `$PATH` and `<bin> --version`/`--help` exits 0, emitting
   an `installed: true, invokable: true` verdict. (Test with a temp
   crate fixture installed to a temp `--root`.)
3. Installs run **strictly one at a time** — the implementation never
   spawns two `cargo install` processes concurrently (asserted by the
   serialization structure / a test that observes ordering).
4. A daemon-tagged artifact is **skipped** by default with a note
   referencing `rollout install`; it is only attempted under
   `--with-daemons`, and then via a shell-out to `rollout install`, not
   a re-implemented restart.
5. Under `--with-daemons` when `rollout` is absent from `$PATH`, the
   daemon artifact is skipped with the note to install `rollout` first;
   the run does not error solely because of the missing `rollout`.
6. Re-running `adopt apply --execute` after a successful install is a
   no-op for that artifact (verdict `installed-current`); idempotent.
7. On the first failed install, the run stops (no cascade) and exits
   non-zero with the failing artifact named.
8. No artifact-derived string reaches a shell (injection guard
   verified with a metacharacter-laden fixture name).
9. `adopt apply --execute` against the live laptop would install
   `rollout` (the 9-day-unadopted non-daemon CLI) and leave
   `command -v rollout` resolving — documented as the real-world
   close-the-loop outcome (the criterion the whole fleet exists for).
