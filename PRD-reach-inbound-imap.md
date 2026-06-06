# PRD: reach-inbound-imap — a real inbound channel so jsy can actually reach her back

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/wintermute-reach
Vision: visions/kin.md
Depends: none (extends shipped wintermute-reach v0.2.0; the dialog reply→TTS
  path it feeds already ships in wintermute-dialog/src/family.rs:363).

## TL;DR

The kin vision's second promise — *"You can reach her back"* — is a stub. When
jsy replies to one of Mom's messages, nothing carries that reply from off-device
back to the device on her desk. `wm-reach reply "..."` is a one-shot CLI publish
that only works if a human runs it on the same box (`src/main.rs:7` literally
labels it `"v1 inbound stub"`, `src/dispatch.rs:54 publish_reply`). This PRD
builds the real inbound channel: an IMAP/maildir poll loop that watches jsy's
mailbox for a reply, and publishes it as a `wm.family.reply` the dialog already
knows how to speak. The send half of the loop shipped; this closes the return half.

## Why this exists

Phase-1 evidence (2026-06-06, direct grep of shipped repos):

- **The reply path is wired everywhere except the wire.** `wintermute-dialog/src/family.rs:363`
  ships `FamilyFsm::on_reply`, which formats `"{from} says: {body}"` and routes it
  to `wm.tts.say` (test `family_fsm_ack_delivered_emits_tts_with_joe` proves it).
  The topic constant `TOPIC_FAMILY_REPLY = "wm.family.reply"` is defined
  (`family.rs:54`). The *only* missing piece is something that produces that event
  from jsy's actual off-device reply.
- **What exists today is a stub, by its own admission.** `wintermute-reach/src/main.rs:7`
  comments the `reply` subcommand as `"publish a wm.family.reply (v1 inbound stub)"`;
  `dispatch.rs:49` repeats `"v1 inbound stub"`. It is a manual CLI publish — useful
  for end-to-end testing, useless as a real return channel.
- **The vision already chose the answer.** `visions/kin.md` OQ#4: *"Email-poll is
  simplest; a webhook needs the device reachable from outside… Deferred to wm-reach
  v0.2 — v0.1 can be send-only with a stubbed reply path."* v0.2 shipped send-only;
  this is the deferred inbound work.
- **The precedent crate already does IMAP cleanly.** `wintermute-mail/Cargo.toml:72`
  uses `async-imap 0.9` with `default-features=false, features=["runtime-tokio"]`
  and `lettre 0.11` rustls-only — no native OpenSSL. reach already sends via
  sendmail/SMTP; reading the reply mailbox is the symmetric move with a proven dep.
- **Spoken replies are an injection surface.** Whatever this publishes is spoken
  aloud to an elderly parent. If the poll accepts any message in the mailbox, a
  spam email becomes a voice in Mom's room. The enrolled-caregiver address
  (`wintermute-family-enroll` writes `/etc/wintermute/conf.d/50-family.env`) is the
  allowlist; only a reply whose `From` matches it may become a `wm.family.reply`.

## What this builds

A new `inbound` module in `wintermute-reach` plus a daemon poll task and config,
behind a default-off `inbound` capability so existing deployments are unchanged
until enrolled.

- **`src/inbound.rs`** — `InboundConfig { enabled, transport, mailbox, poll_secs,
  allow_from: Vec<String> }`; an `InboundTransport` trait with two impls:
  - `MaildirInbound` (default, no network) — scans a maildir `new/` dir, parses
    `From`/`Subject`/body, returns `Vec<InboundReply>`, moves consumed messages to
    `cur/` so they aren't re-emitted. Fully fixture-testable.
  - `ImapInbound` (behind existing-style feature `imap`) — `async-imap 0.9`,
    rustls, fetches UNSEEN from a configured folder, marks `\Seen` after publish.
- **`InboundReply { from, body, ts }` → `wm.family.reply`** — reuse the serde
  envelope shape the dialog expects (`from`, `body`, `ts`). The `from` field is
  set to the enrolled caregiver display name (e.g. `"Joe"`), never the raw email.
- **From-allowlist gate** — a candidate reply whose parsed `From` address is not in
  `allow_from` (seeded from the enroll config's caregiver address) is dropped and
  counted, never published. Logged at info without the body.
- **Daemon integration** — a poll task in the existing daemon select loop, spawned
  only when `InboundConfig::enabled`; applies the existing self-emitted-topic filter
  (it publishes `wm.family.reply`, must not re-consume its own) consistent with
  `daemon.rs:135`.
- **De-dup** — a small seen-set / maildir-move so a slow consumer never double-speaks
  the same reply across two poll ticks.
- **Config** — read from `/etc/wintermute/conf.d/` like the rest of reach; default
  OFF. `WM_REACH_INBOUND_*` env fallbacks mirror the existing `WM_REACH_*` pattern.
- Version bump to **v0.3.0**. sigpipe guard already present in reach; no new binary.

## Acceptance criteria

1. `wm-reach --help` documents an `inbound` capability (a `daemon --inbound` flag
   or `[inbound] enabled` config key); with inbound disabled, daemon behaviour is
   byte-for-byte the prior v0.2.0 (regression test: existing tests still green).
2. A `MaildirInbound` pointed at a fixture maildir containing one reply from the
   enrolled address publishes exactly one `wm.family.reply { from, body }` whose
   `body` equals the fixture body and whose `from` is the enrolled display name.
3. A fixture reply whose `From` is NOT in `allow_from` produces zero
   `wm.family.reply` publishes and increments a `dropped_unauthorized` counter.
4. A consumed maildir message is moved out of `new/` (or marked seen) so a second
   poll tick over the same mailbox publishes nothing — no double-speak.
5. The daemon does not re-consume its own `wm.family.reply` publish (self-emitted
   filter holds; assert no echo loop in a two-tick test).
6. The `imap` feature compiles (`cargo build --features imap`) and is excluded from
   the default build; the default build has no `async-imap` in its dependency tree.
7. No reply body is logged at or above info level; only counts and the authorized
   `From` are logged. (grep the log-emitting sites in test.)
8. `ImapInbound` against a fake-IMAP capture (or a `#[ignore]`/`deferred_acs`
   live-IMAP smoke) fetches UNSEEN and marks `\Seen` — the live-server leg is a
   `deferred_acs` inline int; the maildir leg above is the autonomous proof.
9. `cargo test` green; `cargo clippy` clean under the crate's existing
   `-D`-grade lints (no `unwrap_used`/`expect_used`/`panic`); release-gate receipts
   produced per autobuilder.

## Out of scope / deferred

- A push/webhook inbound transport (needs the device reachable from outside — the
  bootstrap NAT/mDNS story). Email/maildir poll is the headless-safe minimum;
  webhook can be a later feature flag like the outbound `ntfy`/`webhook` ones.
- Multi-turn / threaded replies ("re: which message") — depends on the continuity
  vision; v1 reply is single-shot and spoken as-is.
