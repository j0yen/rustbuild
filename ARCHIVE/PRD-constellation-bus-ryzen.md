# PRD-constellation-bus-ryzen

**Status:** Shipped v1.0
**Vision:** visions/constellation.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/agorabus-nats-bridge

**Depends on:** PRD-constellation-nats-hub (NATS hub must be running on ryzen7)

## TL;DR

ryzen7 has agorabus-nats-bridge v0.5.0 built at
`~/wintermute/agorabus-nats-bridge/target/release/wm-busbridge` but the binary
is not installed and neither `nats-leaf.service` nor `wm-busbridge.service` is
enabled. This PRD installs the bridge and starts the fleet bus on ryzen7.

## Why this exists

**Evidence (2026-06-20, live SSH probes):**
- `ls ~/wintermute/agorabus-nats-bridge/target/release/wm-busbridge` → exists (v0.5.0)
- `which wm-busbridge` on ryzen7 → not installed
- `systemctl --user is-active wm-busbridge` → not-active (service file exists but binary missing)
- `agorabus peers` on ryzen7 shows 0 fleet peers — the bus is single-machine only
- `~/.config/systemd/user/wm-busbridge.service` exists with `After=nats-leaf.service`

With the NATS hub running (PRD-constellation-nats-hub), this PRD completes the ryzen7 side.

## What this builds

Runs on: **ryzen7** (via SSH from wintermute).

1. **Install wm-busbridge** (on ryzen7):
   ```
   install -m755 ~/wintermute/agorabus-nats-bridge/target/release/wm-busbridge ~/.local/bin/wm-busbridge
   ```

2. **Update `~/.config/nats/leaf.conf`** if not already updated by constellation-nats-hub
   (the hub URL should point to localhost:7422 where the hub is now running).

3. **Enable + start `nats-leaf.service`** (leaf connection to local hub on 127.0.0.1:7422):
   ```
   systemctl --user enable --now nats-leaf
   ```

4. **Enable + start `wm-busbridge.service`**:
   ```
   systemctl --user enable --now wm-busbridge
   ```

5. **Selftest**: `wm-busbridge selftest` on ryzen7 — publishes a `wm.fleet.*` event
   locally and confirms it appears on NATS.

6. **Verify fleet peers visible** from wintermute after wintermute's bridge is also up:
   `agorabus peers --fleet` should show ryzen7 session(s) on wintermute.

## Acceptance criteria

1. `which wm-busbridge` returns a path on ryzen7 (binary installed).
2. `systemctl --user is-active nats-leaf` returns `active` on ryzen7.
3. `systemctl --user is-active wm-busbridge` returns `active` on ryzen7.
4. `wm-busbridge selftest` exits 0 on ryzen7 (event round-trips through NATS).
5. No errors in `journalctl --user -u wm-busbridge -n 20` on ryzen7.
