# PRD: agentns-claude-prctl-wire

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/agentns-claude
Vision: visions/continuity.md

## TL;DR

`agentns-claude` is the launcher that wraps every Claude session so
`/proc/$PID/agent_session` reads a stable 128-bit id from birth. It was
scaffolded through iter-2 and **still synthesizes** that id from
`(uid, btime, monotonic_ns)` — the real namespace-creation call was never
wired. Its own source says so: the `kernel_has_agent_ns()` branch in
`src/main.rs` prints `"PENDING: agent_session detected but iter-2 has not
wired CLONE_NEWAGENT; session_id synthesized"` and returns a fake id. This PRD
is iter-3: replace the synthesis branch with a real
`prctl(PR_SET_AGENT_NS)` create-and-enter, set the intent tag via
`prctl(PR_SET_AGENT_INTENT_TAG)`, **read the kernel's own 128-bit id back**
from `/proc/self/agent_session`, and export *that* as `AGENTNS_SESSION_ID`.
After this, a wrapped child reads a non-zero session id that every downstream
consumer (provq, recall-session-stamp, memlog-witness, session-postmortem)
can finally key on.

## Why this exists

The continuity fleet's entire value chain gates on a non-zero session id at
session birth. Today, measured live this pass (2026-06-12) on
`7.0.10-arch1-5-wintermute`:

- `cat /proc/self/agent_session` → 32 zeros (process is in the init agent NS;
  nothing created a fresh one).
- The kernel fix is **already built and on disk**:
  `pacman -Q linux-wintermute` → `7.0.10.arch1-12`; `/boot/vmlinuz-linux-wintermute`
  installed 2026-06-12 01:43; `apply-agentns.py` carries the
  `PR_SET_AGENT_NS` dispatch (`PR_AGENT_BASE 0x41544E53`). But the **running**
  kernel is pkgrel-5, so a direct `prctl(PR_SET_AGENT_NS)` probe returns
  `EINVAL` until the box reboots into pkgrel-12 (a separate, user-gated step
  flagged by self-review's pacman/reboot docket).
- `agentns-claude` source (`src/main.rs:126-138`, `src/unshare.rs`) confirms
  the create call is unimplemented — the `kernel_has_agent_ns()` arm returns
  `synthesize_session_id()` with a `pending-unshare` mode tag. The
  `acceptance_unshare_boot.rs` test is a placeholder awaiting this wiring.

So even after the reboot lands, `/proc/self/agent_session` stays zero for
Claude sessions until the launcher actually *calls* the create op. This PRD is
that call. It is the smallest load-bearing piece of new code in the activation
arc, and it is unit-testable now and live-testable the moment pkgrel-12 boots.

The old `unshare(CLONE_NEWAGENT)` design is dead — `assay-agentns` proved
`CLONE_NEWAGENT == 0x100 == CLONE_VM`, so unshare can never create this
namespace. Creation goes through prctl, per `PRD-agentns-clone-flag-fix` and
`visions/assay.md`. This PRD consumes that decision; it does not relitigate it.

## What this builds

A new resolution mode in `agentns-claude` and the supporting `unshare.rs`
(misnamed now, but keep the module name for churn-minimisation) functions:

- **`unshare::create_agent_ns() -> io::Result<()>`** — calls
  `prctl(PR_SET_AGENT_NS, 0, 0, 0, 0)`. On `Ok`, the calling process is now in
  a fresh agent namespace. On `EINVAL`/`ENOSYS` (old or stock kernel), returns
  a typed `KernelLacksPrctl` error so the caller can fall back. On `EPERM`,
  returns `NeedsCapSysAdmin` with the setcap remediation string. This is the
  one `unsafe`-adjacent FFI call; gate it behind a `#[cfg(not(test))]`-friendly
  wrapper and keep the crate's `forbid(unsafe_code)` posture by routing through
  `nix::sys::prctl` if it exposes a raw option, else a single audited
  `libc::prctl` call in an `unsafe` block with the documented-safety comment the
  lints require (`undocumented_unsafe_blocks = "deny"`).
- **`unshare::read_agent_session() -> io::Result<String>`** — reads
  `/proc/self/agent_session`, trims, validates it is 32 lowercase hex chars and
  **non-zero**; returns the id. A still-zero read after a successful create is a
  hard error (`CreateSucceededButZero`) — that means the kernel op is a stub,
  and we must not silently ship a fake-looking-real id.
- **`unshare::set_intent(tag: &str)`** — `prctl(PR_SET_AGENT_INTENT_TAG, ...)`,
  truncating to `AGENT_NS_INTENT_MAX` (63). Non-fatal on failure (warn only).
- **Rewired `resolve_session()`** with this precedence:
  1. mock (`/tmp/agentns-mock` / `AGENTNS_SESSION_ID_OVERRIDE`) — unchanged,
     wins for tests.
  2. `--no-unshare` explicitly requested → synthesize (unchanged fallback).
  3. kernel has the prctl op → `create_agent_ns()` + `set_intent()` +
     `read_agent_session()` → **real id**, mode `"prctl"`.
  4. kernel lacks the op (EINVAL/ENOSYS) → synthesize + warn
     `"kernel pre-reboot or stock; session_id synthesized (reboot into
     linux-wintermute >= pkgrel-12 to activate)"`, mode `"synth-fallback"`.
  5. EPERM → fatal with the `setcap cap_sys_admin+ep` remediation.
- The `pending-unshare` mode string and its stderr line are **deleted** — the
  state it described no longer exists once this ships.
- `exec_child` already sets `AGENTNS_SESSION_ID` + `AGENTNS_INTENT`; it now
  carries the real kernel id when mode is `"prctl"`. Add an
  `AGENTNS_MODE=<prctl|synth-fallback|mock|no-unshare>` env so downstream
  consumers and the activation-doctor can see provenance of the id without
  re-probing.

Dependencies: `libc` (for the raw prctl option constants) or extend the
existing `nix` dep; no new heavyweight crates. Keep MSRV 1.85, no let-chains.

## Acceptance criteria

1. **AC1 — create op wired.** `unshare::create_agent_ns()` issues
   `prctl(PR_SET_AGENT_NS, 0,0,0,0)` and maps `EINVAL`/`ENOSYS`→`KernelLacksPrctl`,
   `EPERM`→`NeedsCapSysAdmin`, other→`Io`. Unit test asserts the error mapping
   against injected errnos (mock the prctl behind a trait or `#[cfg(test)]`
   seam).
2. **AC2 — read-back + zero guard.** `read_agent_session()` rejects a 32-zero
   read with `CreateSucceededButZero` and accepts a valid 32-hex non-zero id.
   Unit test drives both via an overridable path
   (`AGENTNS_SESSION_PATH` for tests).
3. **AC3 — precedence.** `resolve_session()` returns mode `"mock"` when a mock
   is present even if the kernel op would work; returns `"no-unshare"` under
   `--no-unshare`; returns `"synth-fallback"` (not an error, not `pending`) when
   the kernel lacks the op. Covered by `acceptance_mock.rs` (extended) and a new
   `acceptance_precedence.rs`.
4. **AC4 — pending mode gone.** No code path emits the string `pending-unshare`
   or the iter-2 "has not wired CLONE_NEWAGENT" line; `grep -r pending-unshare
   src/` is empty. Build is clean under the crate's full lint set
   (`unwrap_used = deny`, `forbid(unsafe_code)` honored or each unsafe block
   documented).
5. **AC5 — intent tag.** With `--intent /build`, a successful prctl path calls
   `PR_SET_AGENT_INTENT_TAG` with `"/build"` truncated to 63 bytes; failure to
   set the tag warns but does not abort the exec. Unit-tested via the trait seam.
6. **AC6 — env provenance.** The exec'd child's environment contains
   `AGENTNS_SESSION_ID`, `AGENTNS_INTENT`, and `AGENTNS_MODE`; `AGENTNS_MODE`
   equals the resolver's mode. Tested via `acceptance_exec.rs` (a wrapped
   `env`/`printenv` child).
7. **AC7 [boot] — live non-zero id.** On a box booted into
   `linux-wintermute >= pkgrel-12`, `agentns-claude --intent test -- cat
   /proc/self/agent_session` prints a non-zero 32-hex id, and a second wrapped
   child prints a *different* id (fresh namespace per launch). This fills in
   `acceptance_unshare_boot.rs`. **User-gated on the reboot.**
8. **AC8 — README + CHANGELOG.** README documents the prctl path, the
   `AGENTNS_MODE` env, the `setcap cap_sys_admin+ep` requirement, and the
   fallback behavior on pre-reboot/stock kernels. CHANGELOG notes iter-3.

## Boot gating

AC1–AC6 and AC8 are buildable and testable **now** against the running
pkgrel-5 kernel (the prctl probe returns EINVAL, exercising the fallback path
deterministically). AC7 is the only boot-gated criterion and lands the moment
the box reboots into pkgrel-12 — the same reboot window self-review's pacman
docket already tracks. Do not block the build on the reboot; ship AC1–AC6/AC8
and leave AC7 as a post-boot live checkpoint, the same pattern
`session-postmortem` AC9 uses.

## Open questions

- **Cap delivery.** File caps (`setcap cap_sys_admin+ep`) on the installed
  binary is the plan (`PRD-agentns-launch-flip` owns the install). Confirm
  there is no objection to a `cap_sys_admin` file-capability on a
  `~/.local/bin` binary; the alternative is a tiny setuid-root shim, which is
  worse. Leaning file-cap.
- **Single audited unsafe block vs. nix.** If `nix 0.30` exposes the agentns
  prctl options through `prctl::set(...)` they may not — they're custom
  `PR_AGENT_BASE` options the upstream crate won't know. Most likely one
  documented `unsafe { libc::prctl(...) }` block is required; that is
  acceptable under the crate lints with the safety comment. Confirm during
  iter-1.
