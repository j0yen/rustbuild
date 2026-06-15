# PRD: answerable-session-truth — a ledger line you cannot attribute is a ledger line you cannot dispute

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/answerable
build_version_bump: minor
Vision: visions/answerable.md

## TL;DR

When the agent namespace is dark (`/proc/self/agent_session` reads all
zeros — the current state on this box, where the booted
`linux-wintermute` is pkgrel-1, not the ≥12 that arms agentns), every
`answerable record` line gets `session:0000…0` and is unattributable.
Add a fallback session-identity resolver to the `answerable` crate so
that when agentns is unavailable it derives a real, stable id from a
lower tier — the provfs `user.prov.session` xattr form
(`comm:<comm>:pid:<n>:uid:<n>`) or the agorabus peer id of the writing
process — and stamps that instead.

## Why this exists (Phase 1 evidence, 2026-06-14)

- All 49 lines in `~/.local/state/answerable/ledger.jsonl` carry
  `"session":"00000000000000000000000000000000"`. `tail -5` confirms
  every recent line is zeros.
- This is the documented agentns all-zeros condition: the installed
  kernel is `7.0.11.arch1-1` (pkgrel 1); agentns only populates
  `/proc/self/agent_session` at pkgrel ≥12 booted (see gossip
  2026-06-14, repeated across ~50 dream passes). Until the user installs
  pkgrel-12 and reboots, the agentns surface stays dark.
- An accountability ledger whose every entry is attributed to the null
  session defeats half its purpose: you can see *what* was done but not
  *which run* did it, so two concurrent sessions (this box runs
  main/build/dream/self-review headless) are indistinguishable in the
  record.

The fallback tiers exist: provfs stamps `user.prov.session` on
closed-after-write files (`getfattr -n user.prov.session`), and every
live session has an agorabus peer id (`agorabus peers` shows
`claude-<root-pid>-<project>`). Either is a stable, real identifier.

## What this builds

A `session.rs` module in the `answerable` crate with a single resolver
used by `record`:

- `resolve_session_id() -> SessionId` with this precedence:
  1. **agentns**: read `/proc/self/agent_session`; if it parses to a
     non-zero 128-bit id, use it (unchanged current behavior when armed).
  2. **agorabus peer**: if agentns is zero/absent, query the agorabus
     peer id for this process (read the `WM_SESSION`/agorabus env or the
     peer socket); use `agorabus:<peer-id>` if available.
  3. **provfs/proc fallback**: else synthesize the provfs-style
     `comm:<comm>:pid:<pid>:uid:<uid>` string from `/proc/self/comm`,
     `getpid()`, `getuid()`. Always available.
- The resolved id is recorded in a new optional field `session_kind`
  (`agentns` | `agorabus` | `proc`) alongside the existing `session`
  string, so a reader knows which tier produced the id. Back-compat:
  old lines without `session_kind` parse fine (serde default).
- `record` uses `resolve_session_id()` instead of reading
  `/proc/self/agent_session` directly. The zero-session path is now only
  reachable if all three tiers somehow fail (it should not).

## Acceptance criteria

1. With agentns dark (mock `/proc/self/agent_session` source returning
   zeros), `resolve_session_id()` returns a non-zero `proc`-tier id of
   the form `comm:*:pid:*:uid:*` and `session_kind == "proc"` (unit test
   injecting a zero agentns reader and no agorabus env).
2. With a non-zero agentns id available, the resolver returns it
   unchanged and `session_kind == "agentns"` (unit test with a non-zero
   mock reader) — the armed-kernel path is preserved.
3. With agentns dark but an agorabus peer id present in the environment,
   the resolver returns `agorabus:<id>` and `session_kind == "agorabus"`
   (unit test setting the env var).
4. A ledger line written through `record` under the dark-agentns
   condition has a non-zero `session` and a `session_kind` field; an old
   line lacking `session_kind` still deserializes (back-compat test).
5. `answerable log --json` surfaces `session_kind` when present and omits
   it cleanly when absent (no crash on mixed-vintage ledgers).
6. `cargo test --release` green; no new clippy `-D warnings`; version
   bumped to the next minor; `CHANGELOG.md` prepended.
