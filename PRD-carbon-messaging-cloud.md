# PRD-carbon-messaging-cloud

**Status:** Draft v0.1
**Vision:** visions/carbon.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-carbon-subscribers-cloud (the consumers that trigger sends now run on the hub)

## TL;DR

Outbound messaging — the homeward owner-notify path and the relay email send —
runs today as embedded logic inside laptop-bound daemons. A lost-pet match found
at 2am while the laptop is asleep produces no notification until the laptop wakes.
This PRD makes outbound messaging execute on the always-on hub, triggered by bus
events, so a message queued anywhere in the fleet is *sent* regardless of which
laptop is awake.

## Why this exists

**Evidence (2026-06-20):**
- No dedicated messaging/notify systemd unit exists — `grep -lE
  'notify|email|relay' ~/.config/systemd/user/*.service` is empty. Sends are
  embedded: `homeward-reportd` carries owner-notify; the relay email path
  (PRD-homeward-relay-send, shipped) does a reqwest POST. Both run only where
  their host daemon runs — the laptop.
- Once carbon-subscribers-cloud moves `homeward-ingest`/`homeward-report` to the
  hub, the *triggers* for messages (a new match, a new report) already fire on
  the hub. The send should happen there too, not bounce back to the laptop.
- The user named "messaging" as one of the three things to move. Messaging is the
  most latency- and availability-sensitive of the three: a delayed reminder is
  annoying; a delayed "your lost dog was found" notification is a real harm.

## What this builds

Runs on: **the hub** (deployed from this laptop).

1. **Confirm the send paths run on the hub:** owner-notify (inside
   `homeward-reportd`) and relay email send move with their daemons in
   carbon-subscribers-cloud; this PRD verifies the *send* leg works from the hub
   (credentials present, outbound network allowed, DNS/TLS ok).
2. **Credentials:** the messaging secrets (SMTP/relay creds, any API keys for
   the notify channel) are provisioned on the hub via the constellation secrets
   role (`secrets/shared/…` sops files already exist in the repo). Do NOT copy
   plaintext creds ad hoc — wire through the existing secrets path.
3. **Bus-triggered send:** ensure the message-send is driven by a `wm.*` bus
   event (e.g. `wm.homeward.match`) so any node can queue a message and the hub
   sends it. If the current path is in-process only, add a thin bus subscriber
   that invokes the existing send function on the event.
4. **Idempotency / no double-send:** a message must send exactly once even though
   multiple nodes may see the same bus event — gate on `wm-node role hub` so only
   the hub sends, and dedup on a message id.
5. **Smoke test** an end-to-end send from the hub (to a test address/sink).

## Acceptance criteria

1. Messaging credentials are present on the hub via the constellation secrets
   role (not ad-hoc plaintext); `ssh hub` shows the rendered secret file with
   correct perms (0600).
2. An outbound send executes successfully **from the hub** end-to-end (verified
   against a test recipient/sink; log shows 2xx / accepted).
3. The send is triggered by a bus event: publishing a test `wm.homeward.match`
   (or equivalent) to the fleet bus causes exactly one send from the hub.
4. Only the hub sends — a node without `role hub` seeing the same event does NOT
   send (asserted by the `ExecCondition`/role gate + a message-id dedup).
5. With the laptop asleep/offline, a fleet-published message event still results
   in a send from the hub (the core availability win, verified by stopping the
   laptop's daemons and publishing from ryzen7).
