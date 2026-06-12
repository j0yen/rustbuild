# PRD: agentns-launch-flip

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/agentns-claude
Vision: visions/continuity.md

## TL;DR

The interactive `claude` shell function in `~/.zshrc` already routes the launch
through `agentns-claude` — but with `--no-unshare`, which forces the synthesis
fallback and never attempts namespace creation. And `agentns-claude` isn't even
installed in `~/.local/bin` yet, so the function's guard (`[[ -x
"$HOME/.local/bin/agentns-claude" ]]`) silently fails closed: every session
runs unwrapped. This PRD is the last-mile wiring: build + install
`agentns-claude` with the `cap_sys_admin` file capability, **drop
`--no-unshare`** from the launch path so the real `prctl(PR_SET_AGENT_NS)` is
attempted, and keep a safe fallback so a pre-reboot or stock kernel degrades to
a warning + synthesized id rather than failing the launch. It is the successor
to the archived, futile `PRD-claude-agentns-wrap` (which called the dead
`unshare(CLONE_NEWAGENT)` path).

## Why this exists

Live evidence this pass (2026-06-12):

- `~/.zshrc:32-33` — the `claude()` function:
  `AGENTNS_WRAPPED=1 "$HOME/.local/bin/agentns-claude" --intent interactive
  --no-unshare -- /home/jsy/.local/bin/claude "$@"`. The `--no-unshare` flag
  guarantees synthesis even on a kernel that supports the prctl op.
- `ls ~/.local/bin/agentns-claude` → absent. `cargo build` was never installed.
  So the function's `-x` guard is false and `command claude` runs bare; the
  wrapper is dead wiring.
- The archived `PRD-claude-agentns-wrap` proposed exactly this flip but against
  `unshare(CLONE_NEWAGENT)`, which `assay-agentns` later proved can never work
  (EINVAL, CLONE_VM collision). That PRD is futile by construction; this one
  targets the prctl path that replaced it.
- Self-review's `agentns-session-zeros` docket has recurred **12+ runs** as
  "cause unknown / fix arc pending." The cause is now known and the fix is
  three concrete steps; this PRD performs them.

Without this PRD, `PRD-agentns-claude-prctl-wire` ships a correct launcher that
nothing invokes — the namespace is never entered for a real Claude session, and
`/proc/self/agent_session` stays zero forever regardless of the kernel reboot.

## What this builds

1. **Install + cap.** An idempotent `scripts/install.sh` in the
   `agentns-claude` repo that: `cargo build --release` (via /cloudbuild for the
   heavy compile), `install -Dm755 target/release/agentns-claude
   ~/.local/bin/agentns-claude`, then `sudo setcap cap_sys_admin+ep
   ~/.local/bin/agentns-claude`. Verifies with `getcap` and prints the result.
   Re-runnable; no-op when already current (compare mtime/version).
2. **Wrapper flip.** Edit the `claude()` function in `~/.zshrc` to drop
   `--no-unshare`. The function keeps its `AGENTNS_WRAPPED` recursion guard and
   its `-x` fallback to `command claude`. Because
   `PRD-agentns-claude-prctl-wire` makes the launcher degrade gracefully
   (EINVAL → synthesize + warn, never fail), dropping `--no-unshare` is safe on
   the current pre-reboot kernel: the launch still succeeds, just synthesized,
   until the box reboots into pkgrel-12 — at which point the *same* wrapper
   starts producing real ids with no further change.
3. **Headless parity.** The `/build`, `/dream`, and `/self-review` entry points
   (systemd-user services + any `claude -p` launches) get the same wrapper with
   `--intent <command-name>` instead of `interactive`, so headless sessions are
   namespaced and intent-tagged too. Identify these via
   `systemctl --user cat claude-build.service claude-dream.service` and the
   skill launch scripts; wire the wrapper into each `ExecStart`.
4. **Guard doc.** A short `docs/launch-wiring.md` recording every place the
   wrapper is installed (zshrc, each systemd unit), so a future audit /
   self-review can verify coverage in one read rather than grepping.

## Acceptance criteria

1. **AC1 — installed + capped.** After `scripts/install.sh`,
   `~/.local/bin/agentns-claude` exists, is executable, and
   `getcap ~/.local/bin/agentns-claude` shows `cap_sys_admin=ep`. Script is
   idempotent (second run is a no-op and says so).
2. **AC2 — interactive flip.** `~/.zshrc`'s `claude()` no longer contains
   `--no-unshare`; it still contains the `AGENTNS_WRAPPED` guard and the
   `command claude` fallback. `grep -- --no-unshare ~/.zshrc` returns nothing in
   the `claude()` function body.
3. **AC3 — graceful pre-reboot.** On the current pkgrel-5 kernel, launching
   `claude` through the flipped wrapper succeeds and the child sees
   `AGENTNS_MODE=synth-fallback` with a non-empty `AGENTNS_SESSION_ID` (no hard
   failure, no hang). Verified by a wrapped `printenv AGENTNS_MODE`.
4. **AC4 — headless parity.** `claude-build.service` and `claude-dream.service`
   `ExecStart` route through `agentns-claude --intent /build` (resp. `/dream`);
   `systemctl --user cat` shows the wrapper. A dry-run launch of each shows the
   intent tag in the child env.
5. **AC5 — coverage doc.** `docs/launch-wiring.md` lists every install site and
   matches reality (each listed unit/file actually invokes the wrapper).
6. **AC6 [boot] — live flip.** On a box booted into `pkgrel >= 12`, a freshly
   launched interactive `claude` session reads a non-zero
   `/proc/self/agent_session` and `AGENTNS_MODE=prctl`, with **no edit** to the
   wrapper between pre- and post-reboot (the kernel boot alone flips synth →
   real). **User-gated on the reboot.**

## Boot gating + ordering

Depends on `PRD-agentns-claude-prctl-wire` (needs the graceful-fallback
launcher to make the flip safe pre-reboot). AC1–AC5 land now; AC6 is the
post-boot checkpoint. Ordering: prctl-wire → launch-flip → (reboot) →
continuity-e2e-attest.

## Open questions

- **zshrc edit vs. drop-in.** Editing `~/.zshrc` directly is simplest but
  couples this tool to a dotfile. Alternative: ship a sourced
  `~/.config/agentns/claude-wrapper.zsh` and have `.zshrc` source it, so future
  changes don't re-touch `.zshrc`. Leaning the sourced-file approach for
  durability; confirm with jsy.
- **setcap on every rebuild.** File caps are stripped on `install`/copy, so
  `scripts/install.sh` must re-apply setcap each build. Acceptable; documented
  in AC1. A pacman/AUR-style package would persist it, but that is out of scope.
