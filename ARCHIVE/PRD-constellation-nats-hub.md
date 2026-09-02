# PRD-constellation-nats-hub

**Status:** Shipped v1.0
**Vision:** visions/constellation.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

## TL;DR

The Tailscale mesh is live (wintermute + ryzen7 + hub all connected, confirmed
2026-06-20) but no NATS server is running anywhere — hub:7422 and hub:4222 are
both CLOSED, hub SSH is not accessible. This PRD starts NATS in hub mode on
ryzen7 (which already has `~/.local/bin/nats-server` installed) and creates the
necessary service + config, unblocking both `constellation-bus-ryzen` and
`constellation-bus-wintermute`.

## Why this exists

**Evidence (2026-06-20, live SSH probes):**
- `nc -z -w3 100.66.158.49 7422` → CLOSED; `nc -z -w3 100.66.158.49 4222` → CLOSED
- `ssh 100.66.158.49` → not reachable (hub VPS not bootstrapped)
- ryzen7: `systemctl --user is-active nats` → inactive; `systemctl --user is-active nats-leaf` → inactive
- ryzen7: `which nats-server` → `/home/jsy/.local/bin/nats-server` (already installed)
- ryzen7's `~/.config/nats/leaf.conf` points to `nats://fleet:…@hub:7422` — which resolves to 100.66.158.49 via Tailscale MagicDNS, currently unreachable

Without a running NATS server, `nats-leaf.service` and `wm-busbridge.service` can't start,
so the agorabus fleet fabric is completely dead despite all other infrastructure being ready.

ryzen7 running NATS in hub mode is the fastest unblock. The hub VPS can take over as
the true hub once it's provisioned (ryzen7's config can be changed to leaf then).

## What this builds

Runs on: **ryzen7** (via SSH from wintermute).

1. **`~/.config/nats/hub.conf`** (on ryzen7) — NATS server config:
   ```
   server_name: wm-hub-ryzen7
   listen: "127.0.0.1:4222"   # local client connections
   jetstream {
     domain: hub
     store_dir: ~/.local/share/nats/jetstream-hub
   }
   leafnodes {
     listen: "100.111.184.102:7422"   # accept leaf connections from Tailscale IP
   }
   ```
   Note: Tailscale IP for ryzen7 is 100.111.184.102 (confirmed live).

2. **`~/.config/systemd/user/nats-hub.service`** (on ryzen7):
   ```ini
   [Unit]
   Description=NATS hub — constellation fleet (ryzen7)
   After=network.target

   [Service]
   ExecStart=%h/.local/bin/nats-server -c %h/.config/nats/hub.conf
   Restart=on-failure
   RestartSec=5

   [Install]
   WantedBy=default.target
   ```

3. **Update `~/.config/nats/leaf.conf`** (on ryzen7) — change hub URL from
   `nats://fleet:…@hub:7422` to `nats://127.0.0.1:7422` (same server, leaf
   connection from localhost). This lets the ryzen7 agorabus bridge connect
   via the local leaf port rather than requiring the cloud hub.

4. **Enable and start** `nats-hub.service` on ryzen7 via `systemctl --user enable --now`.

5. **Smoke test**: `nats --server nats://100.111.184.102:4222 account info` from wintermute
   (using Tailscale IP) confirms the hub is reachable cross-machine.

## Acceptance criteria

1. `systemctl --user is-active nats-hub` returns `active` on ryzen7.
2. `nats --server nats://127.0.0.1:4222 account info` exits 0 on ryzen7.
3. `nats --server nats://100.111.184.102:4222 account info` exits 0 from wintermute
   (cross-machine via Tailscale), confirming port 4222 is reachable on the Tailscale IP.
4. JetStream is enabled with domain `hub`: `nats --server 127.0.0.1:4222 str ls` exits 0 on ryzen7.
5. `~/.config/nats/hub.conf` exists and the leaf-listen is bound to the Tailscale IP (not 0.0.0.0, to avoid exposing to non-Tailscale interfaces).
