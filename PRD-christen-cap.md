# PRD: christen-cap — grant the one capability the unshare needs, safely

**Status:** Draft v0.1
**build_target:** mixed
**build_into:** `~/wintermute/christen`
**Vision:** visions/christen.md

## TL;DR

Even once launches are routed through `agentns-claude`, the
`unshare(CLONE_NEWAGENT)` will fail with `EPERM` and silently fall back to an
unwrapped exec unless the launcher binary carries the `CAP_SYS_ADMIN` file
capability. **christen-cap** is the guarded grant: `christen cap` detects
whether `agentns-claude` / `agent-wrap` already carry `cap_sys_admin+ep`,
explains the *precise* scope of the grant (file-scoped, single audited
binary, `+ep`, not setuid-root, not a global change), **prints** the exact
`sudo setcap` line (never runs it), and after a grant verifies the fix by
unsharing inside a sandbox and reading back a nonzero session id. It also
documents the narrower setuid-helper alternative as an open decision.

## Why this exists

- **The capability is a hard prerequisite, from source.** `agent-wrap.c`:
  "Needs CAP_SYS_ADMIN. The intended deployment is file caps … `sudo setcap
  cap_sys_admin+ep /home/jsy/.local/bin/agent-wrap`. … unshare EPERM — needs
  CAP_SYS_ADMIN." `agentns-claude` has the same requirement. Without it,
  routing (christen-route) produces an exec that EPERMs and falls back —
  the session is *still* born unwrapped, and the all-zeros docket item
  (`agentns-session-zeros`) never clears.
- **It's security-sensitive and must stay user-gated.**
  `feedback_classifier_per_command` and the sudo discipline: christen-cap
  must **explain and print**, never auto-`setcap`. CAP_SYS_ADMIN is broad;
  the mitigation is that a *file* cap scopes it to one audited binary with no
  setuid bit, and the PRD must state that trade plainly so the user consents
  with eyes open.
- **The current state is verifiable, measured 2026-06-05.** No live session
  is wrapped (`agent_session = 0…0`), consistent with the launchers lacking
  the cap (or not being routed — christen-route handles the latter; this PRD
  handles the former). `CONFIG_AGENT_NS=y`, so once the cap is granted and
  routing is in place the unshare will succeed.
- Depends on christen-plan only (for the launcher binary paths in the
  `RoutePlan` / config); builds in parallel with christen-detect and
  christen-route.

## What this builds

Extend the `christen` crate (bump minor) with a `cap` module and a `christen
cap` subcommand.

**Capability detection** — a `CapReader` trait (`caps(path) -> CapSet`) with
a real impl (reads the file's effective/permitted capability set via
`getcap`-equivalent — `cap-std`/`caps` crate or shelling `getcap`) and a
`FakeReader` for tests. `CapState` enum per binary: `Present` (has
`cap_sys_admin` in `+ep`), `Absent`, `Unreadable { detail }`,
`Setuid { warn }` (flag if the binary is unexpectedly setuid — a red flag to
surface, not grant over).

**The grant plan** — a pure `cap_plan(binaries: &[(PathBuf, CapState)]) ->
CapPlan` where each entry is `Grant { path, command }` (the exact `sudo
setcap cap_sys_admin+ep <path>` line) | `AlreadyGranted { path }` |
`Blocked { path, reason }` (unreadable / setuid / missing binary).
Declarative — performs no `setcap`.

**The scope explainer** — `christen cap` prints, before any command, a fixed
block stating: (a) what CAP_SYS_ADMIN allows in general, (b) why a *file*
cap on a single non-setuid binary is the scoped mitigation, (c) that the
grant is the user's to run. Required prose; a test asserts the explainer is
emitted before the `setcap` line.

**Post-grant verification** — `christen cap --verify` (read-only, no grant):
spawn `agent-wrap`/`agentns-claude` under `sbx` (the sandbox tool) to run a
trivial command, then read the child's `/proc/$pid/agent_session`; report
`Live` if nonzero, `EPERM-fallback` if still zero (cap not yet granted), or
`Absent` on a non-`-wintermute` kernel. This proves the grant worked without
trusting the absence of a stderr warning.

**UX:** `christen cap [--verify] [--format json]`. Default prints the scope
explainer + the `CapPlan` (per-binary state + the `setcap` line to run).
Never executes `setcap`. JSON carries `{ binaries: [{path, state, action}],
verify? }`.

## Acceptance criteria

1. `cargo build` + `cargo test` pass offline. `cap_plan` is pure (operates
   only on injected `(path, CapState)` pairs — asserted to read nothing).
2. `cap_plan` maps states correctly against fixtures: `Absent` →
   `Grant{command: "sudo setcap cap_sys_admin+ep <path>"}`; `Present` →
   `AlreadyGranted`; `Unreadable`/`Setuid`/missing → `Blocked` with the
   documented reason. The generated `setcap` line is byte-exact.
3. The `CapReader` real impl reads a known binary's cap set (a test grants
   nothing — it reads `agentns-claude`'s *current* set and asserts the
   reader returns a well-formed `CapState`, whatever it is).
4. **Scope explainer** is emitted before any `setcap` line in `christen cap`
   output; a test asserts the explainer text precedes the command and that
   the tool never shells `setcap` itself (fake `setcap` on `PATH` records
   zero invocations).
5. `christen cap --format json` emits the documented schema (per-binary
   state + action) for a mixed fixture (one `Absent`, one `Present`).
6. `christen cap --verify` on this laptop: spawn the launcher under `sbx`,
   read the child's `agent_session`, report `Live`/`EPERM-fallback`/`Absent`
   correctly — **deferred_acs:[6]** (requires the `-wintermute` kernel + the
   real launcher + `sbx`; the cloud box has none).
7. The narrower-privilege alternative (a minimal setuid-root helper that
   `unshare(CLONE_NEWAGENT)`s then immediately drops, vs the file-cap on the
   whole launcher) is documented in the README as an explicit open decision
   with the trade-off stated; no code path auto-installs either.
8. README documents the cap requirement (citing `agent-wrap.c`), the
   print-only grant flow, the scope mitigation, the `--verify` method, and
   that christen-route's wiring is inert until this cap is granted.
