# PRD-constellation-bus-wintermute

**Status:** Draft v0.1
**Vision:** visions/constellation.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/agorabus-nats-bridge

**Depends on:** PRD-constellation-nats-hub, PRD-constellation-bus-ryzen (NATS hub on ryzen7 + ryzen7 bridge both running)

## TL;DR

wintermute (this laptop) has `wm-busbridge` installed at `~/.local/bin/wm-busbridge`
(v0.5.0) but has no NATS leaf config and no `wm-busbridge.service`. This PRD joins
wintermute to the fleet bus by configuring it as a NATS leaf pointing to ryzen7
(the interim hub at Tailscale IP 100.111.184.102:7422).

## Why this exists

**Evidence (2026-06-20, live probes on wintermute):**
- `~/.local/bin/wm-busbridge` exists (wm-busbridge installed)
- `ls ~/.config/nats/` → no such directory (no NATS config)
- `ls ~/.config/systemd/user/wm-busbridge*` → no matches (no service file)
- `ls ~/.config/systemd/user/nats*` → no matches
- Tailscale: wintermute=100.114.123.20, ryzen7=100.111.184.102 (direct connection, tx/rx active)
- `agorabus peers` on wintermute shows 0 fleet peers

wintermute's agorabus is single-machine only. This PRD makes it fleet-aware.

## What this builds

All changes on **wintermute** (local).

1. **`~/.config/nats/leaf.conf`** — NATS leaf config pointing to ryzen7:
   ```
   server_name: wm-leaf-wintermute
   listen: "127.0.0.1:4222"
   jetstream {
     domain: wintermute
     store_dir: ~/.local/share/nats/jetstream
   }
   leafnodes {
     remotes [
       {
         url: "nats://100.111.184.102:7422"
       }
     ]
   }
   ```
   Note: no credentials needed for initial setup — the hub can add auth later.

2. **Install `nats-server` on wintermute** (for the leaf process):
   - `pacman -S nats-server` OR copy from ryzen7 via scp.
   - Fall back: the wm-busbridge can connect directly to ryzen7:4222 (client
     port) without a local leaf process if nats-server is not available.

3. **`~/.config/systemd/user/nats-leaf.service`**:
   ```ini
   [Unit]
   Description=NATS leaf node — constellation fleet (wintermute)
   After=network.target

   [Service]
   ExecStart=%h/.local/bin/nats-server -c %h/.config/nats/leaf.conf
   Restart=on-failure
   RestartSec=5

   [Install]
   WantedBy=default.target
   ```

4. **`~/.config/systemd/user/wm-busbridge.service`**:
   ```ini
   [Unit]
   Description=wm-busbridge — agorabus ↔ NATS bridge (constellation fleet)
   After=agorabus.service nats-leaf.service
   Requires=agorabus.service

   [Service]
   ExecStart=%h/.local/bin/wm-busbridge run
   Environment=WM_NATS_URL=nats://127.0.0.1:4222
   Restart=on-failure
   RestartSec=5

   [Install]
   WantedBy=wintermute.target
   ```

5. **Enable and start**:
   ```
   systemctl --user enable --now nats-leaf wm-busbridge
   ```

6. **End-to-end validation**: from wintermute, `wm-busbridge selftest` confirms round-trip
   through ryzen7 hub. `agorabus peers --fleet` should show ryzen7 session(s).

## Acceptance criteria

1. `systemctl --user is-active nats-leaf` returns `active` on wintermute.
2. `systemctl --user is-active wm-busbridge` returns `active` on wintermute.
3. `wm-busbridge selftest` exits 0 on wintermute.
4. `agorabus peers --fleet` on wintermute shows at least one fleet peer from ryzen7.
5. `agorabus peers --fleet` on ryzen7 shows at least one fleet peer from wintermute.
   (Both machines see each other — bidirectional fleet presence confirmed.)
