# PRD-carbon-rename

**Status:** Draft v0.1
**Vision:** visions/carbon.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-carbon-node-identity (WM_NODE concept must exist to set it)

## TL;DR

This laptop is `hostname=wintermute` and is treated as the system's center.
The carbon vision demotes it to a peer named `carbon`. This PRD performs the
rename across every layer where the old name is load-bearing — hostname,
Tailscale node name, `WM_NODE`, and config/journal references — idempotently,
on **this laptop only**.

## Why this exists

**Evidence (2026-06-20):**
- `hostname` → `wintermute`. The name appears in Tailscale (`wintermute
  100.114.123.20`), in journal paths, and informally throughout the daemons.
- constellation renamed ryzen-work→ryzen7 this same day and learned the rename
  spans three independent layers: system hostname (`hostnamectl`), the Tailscale
  admin-panel node name (does *not* auto-follow hostname), and in-repo string
  references. carbon-rename applies that same checklist to the laptop.
- `WM_NODE` is about to become real (PRD-carbon-node-identity); this PRD is where
  the laptop's `node.toml` is written with `name = "carbon"`.

## What this builds

Runs on: **this laptop** (the box currently named wintermute).

1. **`hostnamectl set-hostname carbon`** (requires sudo — prompt the user; do not
   escalate silently).
2. Write `~/.config/wintermute/node.toml` with `name = "carbon"`, `roles =
   ["voice"]`, `fleet = "wintermute"` (the fleet/system name stays "wintermute"
   — only the *node* is renamed).
3. **Tailscale:** restart `tailscaled`; surface the manual admin-panel rename
   step to the user (the node name in the admin console must be changed by hand,
   as it was for ryzen7).
4. Grep-and-update in-repo references that mean "this node" (not the fleet/brand):
   journal config, any `WM_NODE=wintermute` bootstrap env. Leave brand/fleet
   references ("wintermute" the project) untouched — only the *node* identity.
5. Idempotent: re-running is a no-op once `hostname=carbon` and `node.toml` exist.

## Acceptance criteria

1. `hostname` → `carbon` after the PRD runs (post sudo step).
2. `~/.config/wintermute/node.toml` exists with `name = "carbon"`; `wm-node id`
   prints `carbon`.
3. `tailscale status` shows this node as `carbon` (after the user completes the
   admin-panel rename — the PRD documents this as a manual gate, not a failure).
4. No daemon's `EnvironmentFile`/bootstrap env still sets `WM_NODE=wintermute`;
   any such line now reads `WM_NODE=carbon`.
5. Re-running the script is a clean no-op (idempotency asserted by a second run
   producing no changes).
