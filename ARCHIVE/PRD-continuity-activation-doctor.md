# PRD: continuity-activation-doctor

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/agentns-doctor
deferred_acs: [8]
Vision: visions/continuity.md

## TL;DR

`agentns-doctor status` classifies the *live process's* namespace state
(absent/init/live/malformed) — but that surface cannot distinguish "nothing
wrapped the launch" from "the kernel physically can't create the namespace
yet" from "the wrapper is wired but the box hasn't rebooted." That ambiguity is
exactly why the `agentns-session-zeros` self-review docket has recurred 12+
runs as "cause unknown." This PRD adds an `agentns-doctor activation`
subcommand that reports the **whole continuity chain** as one go/no-go, names
the *exact* blocking layer, and prints the *exact* remediation command — so the
recurring finding becomes closeable instead of perennial.

## Why this exists

Live evidence this pass (2026-06-12):

- `agentns-doctor status` reports the running process is in the init agent NS
  (all-zeros) — true but uninformative; it can't tell jsy whether to reboot,
  reinstall, re-wire the launcher, or fix the kernel.
- The real blocking layer is measurable and was measured this pass, but only by
  hand-running four separate probes:
  - `uname -r` → `7.0.10-arch1-5-wintermute` (running pkgrel **5**)
  - `pacman -Q linux-wintermute` → `7.0.10.arch1-12` (installed pkgrel **12**)
    → **reboot pending**: the fix is on disk but not booted.
  - a compiled `prctl(PR_SET_AGENT_NS)` probe → `EINVAL` (running kernel lacks
    the op).
  - `~/.zshrc` `claude()` → passes `--no-unshare` (wrapper would synthesize even
    post-boot).
- Self-review's 2026-06-12 journal: "agentns: /proc/self/agent_session
  all-zeros … docket key agentns-session-zeros runs_seen=12." Twelve runs of a
  finding that a single composite probe would have closed with "reboot pending,
  run X."

`assay agentns` and `quicken` already attest the kernel *mechanism* (FlagRejected
vs Live) and probe *primitive surfaces*; `agentns-doctor` already reads the live
process. None of them joins running-vs-installed-pkgrel + prctl-probe +
launch-wiring + downstream-stamp into a single activation verdict. This PRD is
that join — a read-only, no-mutation diagnostic that tells next-Claude in one
line where the chain is broken and how to fix it.

## What this builds

A new subcommand in the existing `agentns-doctor` (extends
`~/wintermute/agentns`, reusing its proc-reading code):

`agentns-doctor activation [--json]` evaluates and reports these layers, in
order, short-circuiting to the first that blocks:

1. **kernel-installed** — parse `pacman -Q linux-wintermute` for installed
   pkgrel; parse `uname -r` for running pkgrel. If installed > running →
   verdict `RebootPending { running, installed, remedy: "reboot into
   linux-wintermute (the fix kernel is on /boot already)" }`. (No reboot is
   issued — read-only.)
2. **kernel-prctl** — fork a throwaway child that calls
   `prctl(PR_SET_AGENT_NS)` and reports the result (never mutate the parent's
   namespace). EINVAL/ENOSYS → `KernelLacksPrctl { errno, remedy: install +
   reboot pkgrel >= 12 }`. Reuses the `assay agentns` mechanism if a library
   surface exists; otherwise a minimal in-tree probe.
3. **launcher-installed** — is `~/.local/bin/agentns-claude` present, executable,
   and `cap_sys_admin=ep` (via `getcap`)? Absent/uncapped →
   `LauncherMissing/LauncherUncapped { remedy: run agentns-claude install.sh }`.
4. **launcher-wired** — does the `claude()` shell function (and the headless
   units) route through the launcher *without* `--no-unshare`? Wired-with-
   no-unshare → `WrapperForcesSynth { remedy: PRD-agentns-launch-flip }`.
5. **live-session** — read `/proc/self/agent_session`; if non-zero and a valid
   32-hex id → that layer is `Live`.
6. **downstream-stamp** — write a throwaway probe file under `$HOME`, read back
   `getfattr -n user.prov.session`; report whether provfs stamps the **real**
   agentns id or the `comm:<comm>:pid:<n>` fallback form. Real-id → continuity
   chain proven end-to-end at the file layer.

Output: a human table by default (one row per layer: `LAYER  STATE  DETAIL`)
ending in a single bold verdict line — either `ACTIVATION: LIVE` or
`ACTIVATION: BLOCKED at <layer> — <remedy>`. `--json` emits a stable object
(schema committed) for the SessionStart hook and self-review to consume.

A thin **SessionStart wire** (shell, in `~/.claude/scripts/`) runs
`agentns-doctor activation --json` and, on `BLOCKED`, prints a one-line banner
with the remedy; on `LIVE`, stays silent. This converts the recurring
"all-zeros, cause unknown" docket entry into a self-resolving banner that names
the cause every session until it's fixed.

## Acceptance criteria

1. **AC1 — pkgrel skew detected.** With installed pkgrel > running pkgrel,
   `agentns-doctor activation` reports `RebootPending` naming both pkgrels and
   the reboot remedy. Unit-tested by injecting fixture `uname`/`pacman` output
   via env override (`ACTIVATION_UNAME`, `ACTIVATION_PACMAN_BIN`).
2. **AC2 — prctl probe isolated.** The prctl probe runs in a forked child and
   the parent's `/proc/self/agent_session` is unchanged before and after the
   call (no mutation of the running process). Verified by reading the parent id
   pre/post.
3. **AC3 — launcher layer.** With the launcher absent, reports
   `LauncherMissing`; present-but-uncapped → `LauncherUncapped`; present+capped
   → passes the layer. Driven by a temp `PATH`/`HOME` fixture.
4. **AC4 — wiring layer.** Given a `claude()` function containing
   `--no-unshare`, reports `WrapperForcesSynth`; without it, passes. Driven by a
   fixture rc file via `ACTIVATION_RC_FILE`.
5. **AC5 — single verdict + JSON.** Output ends in exactly one
   `ACTIVATION: LIVE|BLOCKED ...` line; `--json` validates against the committed
   `schemas/activation.schema.json` and includes `{blocked_layer, remedy,
   running_pkgrel, installed_pkgrel, agent_session, stamp_form}`.
6. **AC6 — read-only.** A full run mutates no persistent state: the only writes
   are a throwaway probe file it creates and deletes under `$TMPDIR` or `$HOME`,
   and it issues no `pacman`, `setcap`, `reboot`, or namespace-create on the
   live process. Asserted by running under a `wchg`/`ctrace` capture in the test
   and checking the write set is empty modulo the probe file.
7. **AC7 — SessionStart banner.** The hook script prints a one-line remedy
   banner on `BLOCKED` and is silent on `LIVE`; exits 0 always (never blocks
   startup). Tested by stubbing the doctor's JSON output both ways.
8. **AC8 [boot] — LIVE verdict.** On a box booted into pkgrel >= 12 with the
   launcher installed+capped+wired, `agentns-doctor activation` reports
   `ACTIVATION: LIVE` and the downstream-stamp layer shows the real agentns id.
   **User-gated on reboot + the two launcher PRDs.**
9. **AC9 — README + CHANGELOG.** Documents the subcommand, the layer model, the
   JSON schema, and the SessionStart wire.

## Boot gating + ordering

AC1–AC7 and AC9 are buildable now (every layer below `live-session` is
measurable on the running pkgrel-5 kernel and reports `BLOCKED at
kernel-installed` truthfully today). AC8 is the post-boot LIVE checkpoint.
Independent of the two launcher PRDs for *building* (it only reads their
artifacts), so it can develop in parallel; its `LIVE` verdict naturally gates
on them plus the reboot. This is the tool that makes the activation arc
*observable* — build it early so the rest of the fleet has a scoreboard.

## Open questions

- **Extend agentns-doctor vs. new binary.** Extending keeps the agent-namespace
  diagnostics in one place and reuses its proc readers; a new `continuity-doctor`
  would be a cleaner name but duplicate plumbing. Leaning extend; the subcommand
  name `activation` keeps it discoverable. Confirm with jsy.
- **Overlap with `mend`/`muster` self-review doctors.** Several `mend-*` and
  `muster-*` PRDs add self-review doctor bridges. This subcommand should *emit*
  a docket-compatible JSON line those can consume rather than re-implement their
  reporting. Coordinate the JSON shape with the docket key `agentns-session-zeros`
  so the existing finding resolves rather than forks.
