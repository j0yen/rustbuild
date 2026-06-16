# PRD: agorabus-worker-idempotency-fix

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/autobuilder/proposals/agorabus-worker.draft.sh
Vision: visions/muster.md

## TL;DR

`agorabus-worker.sh` guards against double-spawning a worker for the same
session, but the guard's regex anchors the session id to end-of-line while the
real argv has a trailing cwd argument — so the guard never matches and workers
accumulate without bound. One session on this box has 21 of them. This PRD fixes
the one-line anchor so the idempotency check actually works.

## Why this exists

Verified live (2026-06-16). The live worker script
(`~/.claude/scripts/agorabus-worker.sh` → symlink to
`~/wintermute/dotfiles/.claude/scripts/agorabus-worker.sh`) line 51:

```sh
if pgrep -f "agorabus-worker.sh $sid\$" | grep -v "^${self_pid}\$" >/dev/null 2>&1; then
    exit 0
fi
```

The pattern `agorabus-worker.sh claude-1054-jsy$` requires the session id at the
**end** of the matched argv. But the script is launched (and re-launched on
every agorabus reconnect under the `Restart=always` drop-ins) as:

```
bash …/agorabus-worker.sh claude-1054-jsy /home/jsy
```

The trailing ` /home/jsy` (the worker's own cwd arg, set a few lines below as
`worker_cwd="${2:-$HOME}"`) means the `$`-anchored pattern never matches.
The guard is a silent no-op, so:

- `pgrep -fc "agorabus-worker.sh claude-1054-jsy"` = **21** live workers for one
  session;
- those stale workers keep old `agorabus subscribe` binaries alive across
  reloads → 3 deleted-exe subscribers (pids 1906/2201/579203), the exact
  "fleet-binary-staleness" finding self-review has carried open 3+ runs.

This is the leak *source*. `muster-subtree-*` (this vision) makes the rot
*visible* and *reapable*; this PRD stops new sessions from producing it.

## What this builds

A corrected `agorabus-worker.draft.sh` (copy of the current live script with
only the guard fixed), shipped to
`~/wintermute/autobuilder/proposals/` per the script's own header discipline
("Held out of live infra until jsy smoke-tests in one session"). jsy installs it
via the documented `install -m755 …proposals/agorabus-worker.draft.sh
…dotfiles/.claude/scripts/agorabus-worker.sh` path.

The fix: match the session id as a whole argv token tolerating a trailing arg,
e.g.

```sh
if pgrep -f "agorabus-worker\.sh $sid( |\$)" | grep -v "^${self_pid}\$" >/dev/null 2>&1; then
```

(or equivalently drop the `\$` and match `agorabus-worker.sh $sid`), so an
existing worker for the same sid — launched with the trailing cwd — is detected
and the duplicate exits 0. The change must NOT alter any other behavior:
recursion guard, methods list, RPC dispatch, and the `<sid>-worker` connection
all stay byte-identical.

## Acceptance criteria

1. A shell test (bats or a self-contained `bash` harness) proves: with a live
   process whose argv is `agorabus-worker.sh <sid> /home/jsy`, the guard pattern
   matches that argv (the pre-fix `…$sid\$` pattern does not — the test asserts
   both, demonstrating the regression and the fix).
2. The guard still excludes the worker's own PID (`grep -v "^${self_pid}\$"`) so
   a worker never treats itself as the pre-existing duplicate.
3. The fix touches only the guard line; a `diff` against the current live script
   shows exactly one changed hunk (verified in the PRD receipt).
4. `bash -n proposals/agorabus-worker.draft.sh` parses clean; `shellcheck`
   reports no new warnings vs the live script.
5. The draft is written to `~/wintermute/autobuilder/proposals/` and is NOT
   installed to the live path by the build (install is jsy's gated step).
6. A short note in the draft header records the install command and the verified
   bug (21 workers for pid 1054 on 2026-06-16).
