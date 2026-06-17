# PRD: homeward-alert-delivery — the match alert actually reaches the owner

Status: Shipped
build_target: rust-extend
build_into: /home/jsy/wintermute/homeward
Vision: visions/homeward.md

## TL;DR

`homeward-report` *generates* a `MatchAlert` — deduplicated, framed as "a
possible match appeared," carrying a brokered `contact_token` — but nothing
**delivers** it. The vision end-state #5 ("a match alert fires when a new intake
scores above threshold … within minutes") is unrealized: the alert is an object
in a store, not a notification a searching owner receives. This PRD builds the
last mile honestly — a `Deliverer` trait whose default is dry-run, an email-relay
adapter keyed by the brokered token (never a raw address), an append-only
delivery ledger, and the wiring that fires delivery when an alert is generated.

## Why this exists

Phase-1 live inspection (2026-06-13, `homeward-report/src/alerts.rs`):

- `alerts.rs:38-60` defines `MatchAlert` with a `contact_token` that is
  explicitly "the brokered relay handle — NOT a raw phone/email" (`alerts.rs:43`).
  The module *generates and dedups* alerts (AC3 of homeward-report) but contains
  **no transport** — no function sends, emails, webhooks, or otherwise delivers.
- The owner-facing service (`reportd serve`) holds reports and can surface alerts
  on read, but the vision's promise is **push within minutes**, not poll-and-hope.
  An owner who has to keep refreshing has not been alerted.
- The honesty pattern is already established in this fleet: `homeward-federation-
  export` ships a `Syndicator` whose only machine target is **dry-run-by-default**
  and whose gated channels are `ManualOnly`, never fictional transports. Delivery
  must follow the same contract — a real adapter that is off until a real relay
  credential exists, with the dry-run path fully exercised and audited.

This is a `rust-extend` of the existing `homeward-report` crate.

## What this builds

- **A `Deliverer` trait** in `homeward-report` (`src/delivery.rs`): `fn
  deliver(&self, alert: &MatchAlert) -> DeliveryOutcome`. Default registered
  deliverer is `DryRunDeliverer` — it renders the exact message that *would* be
  sent and records it, sending nothing. Parallels `Syndicator`'s honesty.
- **An email-relay adapter** (`RelayEmailDeliverer`): resolves the brokered
  `contact_token` → a relay address via the existing contact-broker indirection,
  renders a plain-language alert (candidate-not-confirmation framing preserved
  from `alerts.rs`), and is **disabled unless `HOMEWARD_RELAY_ENDPOINT` +
  credential are configured** — absent config, it degrades to dry-run, never
  errors, never leaks a raw address.
- **An append-only delivery ledger** (`delivery_log`): every attempt records
  `{alert_id, report_id, deliverer, outcome (DryRun|Sent|Suppressed|Failed),
  ts}`; O_APPEND, SIGPIPE-safe, queryable via a `reportd alerts-log` subcommand.
  An alert is delivered **at most once** (dedup keyed on `alert_id`).
- **Wiring**: when `reportd` generates a `MatchAlert`, it invokes the registered
  `Deliverer` and writes the ledger. A `reportd deliver --report <id> --dry-run`
  subcommand makes the path manually exercisable and testable.

### Non-goals

- No SMS/push/Nextdoor transports — email-relay is the one honest machine channel;
  others are partnership-gated (same finding as the federation export table).
- No change to alert *generation* or threshold logic — this delivers what
  `alerts.rs` already produces.
- No raw contact storage — the brokered-token indirection is preserved end to end.

## Acceptance criteria

1. A `Deliverer` trait exists with a `DryRunDeliverer` default that renders the
   would-send message and records a `DryRun` outcome **without sending anything**;
   a unit test asserts no transport is attempted in the default configuration.
2. `RelayEmailDeliverer` degrades to dry-run (not error) when
   `HOMEWARD_RELAY_ENDPOINT`/credential are unset; a test asserts an unconfigured
   relay never attempts a send and never surfaces a raw address.
3. The delivery ledger is append-only and records one entry per attempt with
   `{alert_id, report_id, deliverer, outcome, ts}`; `reportd alerts-log` prints it.
4. An alert is delivered at most once: generating the same `MatchAlert` twice
   yields exactly one non-suppressed ledger entry (dedup on `alert_id`), asserted
   by test.
5. `reportd deliver --report <id> --dry-run` runs the full generate→deliver→ledger
   path and prints the rendered message; exits 0 with a `DryRun` ledger entry.
6. The rendered message preserves the candidate-not-confirmation framing
   (no "we found your pet" language) and never contains a raw phone/email — only
   the brokered token or its relay-resolved handle; asserted by test.
7. `cargo test` for `homeward-report` passes and `cargo clippy` introduces no new
   warnings over the crate's baseline.
