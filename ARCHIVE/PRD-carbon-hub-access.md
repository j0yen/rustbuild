# PRD-carbon-hub-access

**Status:** Shipped v1.0
**Vision:** visions/carbon.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-carbon-node-identity (hub declares role=hub)

## TL;DR

The cloud hub at `100.66.158.49` is live in the Tailscale mesh (direct path,
active tx/rx) but is **not SSH-reachable** from this laptop — `ssh hub` fails
host-key verification with no key installed. Nothing can move to the cloud until
the hub is a working node: SSH access, a `systemd --user` session, agorabus,
NATS in hub mode, and — because the hub is **ARM** while cloudbuild is x86 — an
ARM build path. This PRD is the unblock everything cloud-side cascades from
(mirroring PRD-constellation-nats-hub, which was constellation's unblock).

## Why this exists

**Evidence (2026-06-20):**
- `ssh hub` → `Host key verification failed`; `ssh jsy@100.66.158.49` →
  `Permission denied` (no key, password auth not set up). The box is reachable at
  the network layer (Tailscale direct path confirmed this session) but not at the
  shell layer.
- `docs/cloud-hub.md` specifies **Hetzner CAX21, ARM, 4vCPU/8GB**. cloudbuild
  (Hetzner x86 burst) produces x86_64 binaries that will not run on ARM — the
  same arch-mismatch class that bit constellation on ryzen7 (Ubuntu/AVX-512).
- constellation already ships `cloud/scripts/hub-failover.sh` and a headscale
  role — the hub is *planned* infrastructure, just never brought to a usable
  shell state for relocation work.

## What this builds

Runs on: **this laptop**, targeting the **hub** over Tailscale.

1. **SSH access:** install the laptop's public key to the hub's
   `authorized_keys` (the user provides the one-time password or runs the
   `ssh-copy-id` step themselves — surface it, do not assume credentials).
   Verify `ssh hub hostname` succeeds.
2. **systemd --user session:** enable lingering (`loginctl enable-linger`) so
   user units run without an active login — the hub has no interactive session.
3. **agorabus + NATS hub:** install `agorabus` and `nats-server` on the hub
   (ARM builds — see step 5); write `nats hub.conf` (clients :4222, leaf :7422,
   JetStream domain=hub) and `agorabus.service`; enable both.
4. **Node identity:** write the hub's `node.toml` with `name = "hub"`,
   `roles = ["hub"]` (consumes PRD-carbon-node-identity).
5. **ARM build path:** pick and document one of — (a) native build on the hub
   via rustup (slow on CAX21 but simple), (b) cross-compile `aarch64-unknown-
   linux-gnu` from this laptop, or (c) a burst ARM builder. Default to (a) for
   bootstrap; record the choice in `constellation/docs/cloud-hub.md` and emit a
   `wm-armbuild` helper script under `constellation/cloud/scripts/`.

## Acceptance criteria

1. `ssh hub hostname` succeeds from this laptop (key-based, no password prompt).
2. `ssh hub 'loginctl show-user $USER -p Linger'` → `Linger=yes`.
3. `ssh hub 'systemctl --user is-active nats-hub agorabus'` → both `active`;
   `nats --server nats://127.0.0.1:4222 account info` exits 0 on the hub.
4. `ssh hub 'wm-node role hub'` exits 0 (identity wired).
5. The ARM build path is exercised end-to-end at least once: one wintermute Rust
   binary (e.g. `agorabus`) is built for aarch64 and runs on the hub
   (`agorabus --version` exits 0 on the hub); the chosen method is documented in
   `docs/cloud-hub.md`.
