# PRD: wintermute-mail — voice-driven email read + compose

**Author:** /dream (Claude Opus 4.7), with jsy
**Status:** Draft v0.1
**Date:** 2026-05-27
**Vision:** `visions/wintermute.md` (Fleet 2 — action layer)
**Builds on:** `PRD-wintermute-dialog.md`, `PRD-wintermute-brain.md`,
  `PRD-wintermute-bootstrap.md` (account credentials enter via the
  caregiver web UI)
build_target: rust-cli
build_priority: medium
deferred_acs: [1, 2, 3, 4, 5, 6, 7, 8, 10]
deferred_ac_reasons:
  "1": "wm-bootstrap /mail HTTP page integration and live SecretService write require a running wm-bootstrap daemon and freedesktop SecretService — not available offline; set-account mock path is tested in acceptance_ac3.rs and acceptance_ac11.rs."
  "2": "inbox against a real Gmail app-password account requires live IMAP credentials and an active Gmail session; stub mode returns empty messages array (tested offline in acceptance_ac5.rs)."
  "3": "read {id} returning a real message body with HTML-converted text and live attachment metadata requires a live IMAP session; JSON shape and HTML stripping are tested offline in acceptance_ac6.rs and acceptance_ac8.rs."
  "4": "send with destructive verbal-confirm flowing through wm-dialog and actual SMTP delivery to a recipient inbox requires a live SMTP session; destructive guard and JSON response shape are tested offline in acceptance_ac7.rs."
  "5": "search {query} returning ≥1 hit in a primed test account requires a live IMAP server with known mail; no meaningful offline mock for server-side IMAP SEARCH result count."
  "6": "mark_read flipping the IMAP \\Seen flag on a real server and verifying the subsequent inbox excludes the message requires a live IMAP session; no meaningful offline mock for server-side flag persistence."
  "7": "delete with verbal confirmation and server-side Trash move requires a live IMAP session; no meaningful offline mock for IMAP MOVE-to-Trash round-trip."
  "8": "IMAP IDLE producing wm.mail.new within 30s requires a live IMAP server delivering real mail; compile-time and stub coverage in acceptance_ac10.rs."
  "10": "real voice round-trip (brain → search → dialog read → brain compose → send confirm) requires the full Fleet 1+2 runtime stack (agorabus, wm-brain, wm-dialog) plus live IMAP/SMTP; no meaningful mock for a multi-daemon voice interaction."
mock_unjustified_for: [5, 6, 7, 10]
mock_justifications:
  "5": "IMAP SEARCH result count from a real server cannot be meaningfully mocked — a stub that returns a hardcoded hit would prove nothing about the server-side query path."
  "6": "IMAP \\Seen flag persistence across a server round-trip cannot be mocked without embedding a real IMAP server; the test would be self-referential."
  "7": "IMAP MOVE-to-Trash is a server-side operation; a mock that fakes the move proves only that the code calls the right function, not that mail is preserved safely in Trash."
  "10": "A full voice round-trip spans wm-brain, wm-dialog, agorabus, IMAP, and SMTP — mocking all five subsystems would be tautological and would validate the mock harness rather than the integration."

---

## TL;DR

A daemon `wm-mail` that exposes a small mail surface over agorabus:
inbox listing, message reads, send, search. IMAP (`async-imap`) for
read; SMTP (`lettre`) for send. Credentials live in the freedesktop
keyring set up by `wm-bootstrap` (extended in this PRD with a
`/mail` form). Destructive actions (delete, mass mark) go through
`wm-dialog`'s verbal-confirmation protocol.

---

## 1. Why this exists

Vision §End-state #9: *"Mail / calendar / music through MPRIS,
IMAP/SMTP, CalDAV."* Mail is the highest-utility piece for the
voice-first user — most everyday business with caregivers, family,
doctors flows through email.

Concrete evidence from Phase 1:

- `~/wintermute/wintermute-bootstrap` is shipped — already runs a
  caregiver-facing HTTP form for one-time setup. Adding a `/mail`
  step is a small extension (a separate PRD's iter scope), not a
  re-architecture.
- `async-imap` (MPL-2.0, active) and `lettre` (MIT-OR-Apache, broad
  use) are the proven Rust IMAP+SMTP stack. Both async-friendly,
  rustls TLS, no native OpenSSL needed.
- `secret-service` crate over freedesktop SecretService gives a
  durable keyring for IMAP password / SMTP password / OAuth token
  (when supported).

---

## 2. What this builds

### 2.1 Binary: `wm-mail`

Long-running daemon. Connects to IMAP idle on the primary inbox;
publishes `wm.mail.new` envelopes on arrival (brain decides whether
to interrupt).

### 2.2 Tools (topic `wm.mail.cmd`)

| Tool | Args | Returns |
|---|---|---|
| `inbox` | `{limit?=10, unread_only?=true}` | `{messages:[{id, from, subject, date, snippet}]}` |
| `read` | `{id}` | `{from, to, subject, body_text, attachments}` |
| `send` | `{to, subject, body, in_reply_to?}` | `{ok, message_id}` — destructive: confirm |
| `search` | `{query, limit?=20}` | `{messages}` — IMAP SEARCH |
| `mark_read` | `{id}` | `{ok}` |
| `delete` | `{id}` | `{ok}` — destructive: confirm |
| `folders` | `{}` | `{folders}` |

### 2.3 Verbal-confirm protocol

`send` and `delete` emit `wm.brain.reply.destructive` (already
handled by `wm-dialog` Fleet 1). Confirmation text constructed by
brain ("you want me to send 'Yes I'll be there at 3' to John — say
'yes send'").

### 2.4 Credentials flow

`wm-bootstrap` (already shipped) gets a new `/mail` page that posts
to `wm-mail set-account` over agorabus. The daemon writes to
SecretService and reloads.

Fields: IMAP host/port/user/pass, SMTP host/port/user/pass, From
address, friendly name. No OAuth v1 — Gmail-app-passwords + iCloud
app-specific are the documented paths.

### 2.5 New-mail signal

IMAP IDLE on INBOX. On a new message, publish `wm.mail.new` with
`{id, from, subject}`. Brain's policy decides whether to interrupt
("you have a new email from your sister, want me to read it?") or
let it wait. Quiet-hours respected via Fleet 3 once that ships;
v1 default: never interrupt, only on explicit "any new mail?".

---

## 3. Risks

- **Provider quirks** — Gmail requires app passwords or OAuth;
  iCloud requires app-specific passwords; some EU providers have
  rate-limits. Document Gmail-app-passwords + iCloud setup in the
  bootstrap `/mail` page.
- **Attachment handling** — v1: list names + types only; brain says
  "this message has an attached PDF, I can't read attachments yet".
  Future Fleet 2 extension can chain through `wm-screen-narrate` for
  inline images.
- **Big inboxes** — IMAP SEARCH over 10y of mail is slow on some
  servers. Default `inbox` limit 10; `search` server-side with
  IMAP SEARCH keywords (not body grep).
- **HTML body** — convert to text via `html2text` crate; preserve
  links as bracketed `[text](url)` for brain to optionally read.

---

## 4. Sequencing

Independent of `wm-browser` / `wm-desktop` / `wm-screen-narrate`.
Depends on `wm-bootstrap` shipped (it is, per archive). Composes
with `wm-calendar` (some invitations land as mail; future cross-PRD
bullet).

---

## 5. Acceptance criteria

1. `wm-bootstrap` `/mail` page accepts an account, posts to
   `wm-mail set-account`, daemon writes to SecretService and reports
   `{ok:true, host:<host>, user:<user-masked>}`.
2. `wm-mail inbox` against a Gmail app-password account returns
   the latest 10 messages with `from`, `subject`, `date`, `snippet`
   populated; HTML stripped from snippet.
3. `wm-mail read {id}` returns the full body in `body_text`,
   HTML converted, attachments listed by name+MIME but not content.
4. `wm-mail send` issues a destructive confirmation through
   `wm-dialog`; on "yes send", the message lands in the recipient's
   inbox and the From-account's Sent folder.
5. `wm-mail search {query:"from:sister"}` returns at least 1 hit
   in a primed test account.
6. `wm-mail mark_read {id}` flips IMAP `\Seen`; subsequent
   `inbox {unread_only:true}` excludes it.
7. `wm-mail delete` requires verbal confirmation; on "yes delete",
   moves to Trash (not expunge) for safety.
8. IMAP IDLE: a new arriving message produces a `wm.mail.new`
   publish within 30 s of server delivery (verified by manual send
   from a phone).
9. Credentials never logged or sent over agorabus in plaintext —
   only the masked-user form is published; ctrace summary confirms
   no password substrings in network or stdout traffic.
10. **[live]** Real round-trip: jsy says "any new mail from my
    sister?", brain calls `search`, dialog reads the latest. If
    she says "reply yes I'll be there", brain calls `send` with
    confirm flow. End-to-end <20 s.
