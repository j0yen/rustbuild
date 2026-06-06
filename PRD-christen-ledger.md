# PRD: christen-ledger — record each session's true identity and footprint

**Status:** Draft v0.1
**build_target:** mixed
**build_into:** `~/wintermute/christen`
**Vision:** visions/christen.md

## TL;DR

Once sessions are born wrapped (christen-route + christen-cap), each one
carries a true session id, an intent tag, a budget, and live syscall
counters — and today all of that evaporates on exit. **christen-ledger**
captures the payoff: at session birth it records `(session_id, intent,
budget, start_counters)`; at session-end it snapshots the final
`agent_counters` and writes a per-session JSON entry under
`~/.claude/christen/ledger/`. The result is a real, queryable "what did
session X actually do" record — syscalls, bytes written, forks, connects,
unlinks, wall time — keyed by the **true** session id that agorabus,
memlog, and provfs have all been attributing to `0…0` instead.

## Why this exists

- **The identity exists but nothing keeps it.** Once routed+capped, a
  session's `/proc/$PID/agent_session` is a stable nonzero id and
  `/proc/$PID/agent_counters` ticks real numbers (measured-zero today only
  because unwrapped). No artifact persists either across the session's
  death — and headless ticks die by SIGKILL (the coda finding:
  cgroup-teardown kills the session before graceful hooks), so an in-process
  record would be lost anyway.
- **The downstream consumers are already keyed to this id.**
  `agorabus-session-start.sh:55` derives the agorabus session-id from
  `/proc/self/agent_session`; memlog/provfs stamp `user.prov.session` with
  the agentns id when `CONFIG_AGENT_NS=y` (it is). Today they all collapse to
  `0…0`. A ledger keyed by the true id is what makes cross-session
  attribution and post-mortem (the continuity vision) finally legible.
- **The receipt primitive exists.** `agentns-doctor receipt` already
  "snapshots agent-namespace counters to a JSON ledger (session receipt)."
  christen-ledger is the *wiring*: call it at the right moments, key by
  session id + intent, accumulate under one directory, and make it queryable
  — it does not re-implement counter reading.
- Depends on christen-route being live to produce non-trivial entries, but
  builds and tests fully against a `FakeReader` (injected counters), so it
  is cloud-build-safe.

## What this builds

Extend the `christen` crate (bump minor) with a `ledger` module, a `christen
ledger` subcommand, and a print-only SessionEnd hook installer.

**The ledger model:**

- `LedgerEntry { session_id: String, intent: Option<String>, budget:
  Option<String>, opened_at: u64, closed_at: Option<u64>, start: Counters,
  end: Option<Counters>, kernel: String }` — `serde`-serializable; one JSON
  file per session at `~/.claude/christen/ledger/<session_id>.json`.
- `Counters` mirrors `/proc/$PID/agent_counters` (`total_syscalls`,
  `openat_count`, `write_bytes`, `connect_count`, `unlink_count`,
  `fork_count`, `elapsed_ns`).
- A pure `delta(start, end) -> Counters` and a pure
  `summarize(&LedgerEntry) -> EntrySummary` (human one-liner: intent, id
  prefix, wall time, top counter movers).

**The `LedgerStore` trait** (`open(entry)`, `close(session_id, end)`,
`list()`, `get(session_id)`) with a real `FsStore`
(`~/.claude/christen/ledger/`) and a `FakeStore` for tests. Writes are
idempotent; closing an already-closed or unknown session is a logged no-op,
not an error (a SIGKILLed session that never `close`d leaves an open-only
entry — that itself is a signal, not a bug).

**The subcommand:**

- `christen ledger open` — read the current process's
  `agent_session`/`intent`/`counters`, write an open `LedgerEntry`. Intended
  for a SessionStart hook (after routing makes the id real).
- `christen ledger close` — read final counters, patch the matching entry's
  `closed_at`/`end`. Intended for a SessionEnd hook. Reads the session id
  from `/proc/self` so it self-identifies.
- `christen ledger list [--format json]` — table/JSON of entries with the
  `EntrySummary` line; `--open-only` shows sessions that never closed
  (SIGKILL casualties).
- `christen ledger show <id>` — full entry + computed `delta`.

**The hook installer** — `christen ledger install` PRINTS (never writes) the
`settings.json` SessionStart (`christen ledger open`) and SessionEnd
(`christen ledger close`) hook entries, plus a note that they are inert until
christen-route/christen-cap make the session id real. No auto-edit of
`~/.claude` (`feedback_classifier_per_command`).

## Acceptance criteria

1. `cargo build` + `cargo test` pass offline. `delta` and `summarize` are
   pure; `delta` of equal counters is all-zero; a wall-time `summarize`
   covers a fixture with known movers.
2. `LedgerEntry`/`Counters` round-trip through `serde_json`; a fixture entry
   serializes to the documented one-file-per-session shape.
3. `LedgerStore` against `FakeStore`: `open` then `close` yields a complete
   entry with `end` set; `close` on an unknown/already-closed id is a no-op
   (logged), not an error; `list --open-only` returns only never-closed
   entries.
4. `christen ledger open`/`close` drive the store from injected counters in a
   `FakeReader` integration test (no real `/proc` dependency); the entry's
   `delta` matches the injected movement.
5. `christen ledger list --format json` emits the documented schema with one
   entry per session plus the `EntrySummary` line; `christen ledger show
   <id>` prints the full entry + delta. `christen ledger list | head` does
   not panic (SIGPIPE).
6. `christen ledger install` PRINTS the SessionStart + SessionEnd hook JSON
   and the "inert until routed" note; a test asserts it writes nothing under
   `~/.claude` and shells nothing.
7. **Live ledger on the laptop** — with routing+cap in place, a real wrapped
   session writes an `open` entry at start and a `close` entry with nonzero
   counters at end — **deferred_acs:[7]** (needs a real `-wintermute`
   wrapped session; the cloud box has neither kernel nor session history).
8. README documents the entry schema, the open-only-as-SIGKILL-signal
   semantics, the `agentns-doctor receipt` relationship, and how the ledger
   id lines up with agorabus/memlog/provfs session attribution.
