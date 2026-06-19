# PRD: homeward-relay-send — make owner notification actually send

- Status: Draft v0.1
- build_target: rust-extend
- build_into: /home/jsy/wintermute/homeward
- Vision: visions/homeward.md
- build_version_bump: minor

## TL;DR

The homeward delivery layer is fully built end-to-end — a lost-pet owner can
submit a report, photos get matched against shelter intake, and a `MatchAlert`
is produced and logged. But the **last inch is a stub**: when a relay endpoint
*is* configured (`HOMEWARD_RELAY_ENDPOINT` set), `RelayEmailDeliverer::deliver`
does **no actual network I/O** — it fabricates a `relay-stub:<id>@<endpoint>`
message-id and records `Sent` in the ledger. So in production an owner is told
(in the ledger) that they were notified when no email/SMS ever left the box.
This PRD wires that one method to a real HTTP relay POST so the notification
actually goes out — the difference between "homeward says it reunited a pet" and
"homeward actually emailed the owner."

## Why this exists

Phase-1 research (2026-06-19) against `~/wintermute/homeward/homeward-report`:

- `src/delivery.rs:160` — `RelayEmailDeliverer` exists with `is_configured()`
  reading `HOMEWARD_RELAY_ENDPOINT`, a real `deliver_as_dry_run` path, and a
  `deliver()` that degrades to dry-run when unconfigured.
- `src/delivery.rs:70-73` (in the configured branch) — explicit comment:
  *"is a stub that records Sent without actual network I/O"*; it builds
  `relay-stub:{alert_id}@{endpoint}` and returns `Sent` without sending.
- The rest of the chain is real: `src/webhook.rs` already uses `reqwest` with a
  validated URL (`validate_notify_url`), `src/alerts.rs` produces `MatchAlert`,
  `src/match_watch.rs` calls `process_candidate`, and `homeward-reportd deliver`
  is a working CLI subcommand. Only the email/SMS relay leg is faked.
- `reqwest` is already a workspace dependency (used by `webhook.rs`), so no new
  dependency is required.

This is the only genuinely-missing piece of the user-facing delivery layer; the
serve/search/matches/upload/submit/web-ui/db-reader pieces all already ship in
`homeward-report` (verified 2026-06-19, see gossip note same date).

## What this builds

Extend `homeward-report/src/delivery.rs`:

- Replace the stub body of the configured branch of `RelayEmailDeliverer::deliver`
  with a real blocking `reqwest` POST to `HOMEWARD_RELAY_ENDPOINT`, sending a
  JSON body `{alert_id, report_id, to, subject, body}` where `body` is the
  existing `render_alert_message(alert)` output and `to` is the report's contact
  token. Reuse `validate_notify_url` from `webhook.rs` to guard the endpoint
  (deny non-https / localhost-by-default, same policy as the webhook sink).
- Map the HTTP result to the existing `DeliveryOutcome`/`DeliveryRecord`:
  - 2xx → `Sent`, with the **real** relay message-id from the response body
    (`message_id` field) or, if absent, the response's `X-Message-Id` header;
    deliverer label `RelayEmail(sent)`.
  - non-2xx or transport error → `Failed { reason }` (a NEW `DeliveryOutcome`
    variant if one doesn't exist; otherwise reuse the existing failure variant),
    deliverer label `RelayEmail(failed)`. **Never record `Sent` on a failed
    send** — that is the exact bug being fixed.
- Honor a `HOMEWARD_RELAY_TIMEOUT_SECS` env (default 10s) so a hung relay can't
  stall the deliver path; a timeout maps to `Failed`.
- Keep `is_configured()` and the unconfigured degrade-to-dry-run path exactly as
  they are. The behavioral change is ONLY in the configured branch.
- Idempotency is unchanged: the `ledger.has_delivered(&alert_id)` suppression
  check still runs first, so a retried alert that already sent is `Suppressed`.

## Acceptance criteria

1. With `HOMEWARD_RELAY_ENDPOINT` set to a mock HTTP server returning 200 with
   `{"message_id":"real-123"}`, `RelayEmailDeliverer::deliver` performs a real
   POST and the resulting `DeliveryRecord` has `outcome == "Sent"`,
   `deliverer == "RelayEmail(sent)"`, and the stored message-id is `real-123`
   (NOT a `relay-stub:` value).
2. The POST body is valid JSON containing `report_id`, the rendered alert text
   from `render_alert_message`, and the report's contact token as `to`.
3. With the mock server returning 500, the outcome is a failure variant
   (`Failed`/equivalent), `deliverer == "RelayEmail(failed)"`, and the ledger
   does NOT record `Sent`.
4. With `HOMEWARD_RELAY_ENDPOINT` unset, behavior is byte-identical to today's
   degrade-to-dry-run path (outcome `DryRun`, label `RelayEmail(degraded-dry-run)`).
5. A relay endpoint that fails `validate_notify_url` (e.g. `http://localhost`)
   is rejected before any send, mapping to a failure variant with a clear reason;
   no network call is attempted.
6. A relay that does not respond within `HOMEWARD_RELAY_TIMEOUT_SECS` maps to a
   failure variant (not a hang, not `Sent`).
7. A previously-delivered `alert_id` is `Suppressed` (no second POST) regardless
   of endpoint configuration — the idempotency guard runs first.
8. `cargo build --release --workspace` and `cargo test -p homeward-report` are
   green; no `unwrap`/`expect`/`panic` in the new code per the repo's deny lints.

## Notes

- Tests use a local mock HTTP server (the pattern `homeward-deliver-enroll`
  already established with its `/enroll` mock) — no real SMTP/relay account
  needed at gate time. Real relay creds are an ops concern, set via
  `HOMEWARD_RELAY_ENDPOINT` at runtime, never committed.
- This is `rust-extend` into the existing `homeward` workspace; build MUST use
  `cloudbuild build homeward -- --release --workspace` (the workspace flag is
  required to compile the `homeward-report` binaries, not just the root lib —
  learned 2026-06-19).
