# PRD: mend-ctrace-render — ctrace summaries should render on exit, not get backfilled every morning

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/ctrace-scribe
Vision: visions/mend.md

## TL;DR

The `SessionEnd` hook is supposed to stop the session's ctrace and render its
summary as the session closes. It doesn't — reliably enough that every
self-review run finds N session logs with no summary and renders them via a
backfill pass (`ctrace scribe backfill`). The backfill is a band-aid that has
persisted across 5+ runs and is tracked as docket finding
`ctrace-sessionend-flake` (runs_seen: 5). This PRD fixes the render-on-exit
path so summaries write when the session ends, demotes backfill to a true
fallback, and reports to docket only when *both* miss.

## Why this exists

- **It's a counted, owner-less chronic finding.** Live docket this pass:
  `[open] ctrace-sessionend-flake (warn) — "ctrace SessionEnd hook not writing
  summaries (7 backfilled)" runs_seen: 5 report_count: 5`. No vision claims it.
- **Every recent self-review re-applies the band-aid.** journal 2026-06-07:
  "ctrace scribe backfill: rendered 7 … residual 0"; 2026-06-06: "scribe
  backfill rendered 626 (residual 0, 1251 skipped)". The reflection note is
  explicit: *"backfill is a band-aid; fix the hook."*
- **The hook is structurally fragile.** `~/.claude/scripts/ctrace-session-end.sh`
  renders only if a marker file `~/.cache/ctrace/claude-owns.json` exists and
  its `.log` field points at an extant file:
  ```
  [ -f "$marker" ] || exit 0
  log=$(jq -r '.log // empty' "$marker" …)
  "$ctrace" stop …
  rm -f "$marker"
  [ -n "$log" ] && [ -f "$log" ] && [ -x "$summarize" ] && "$summarize" "$log" …
  ```
  If the marker was never written (session didn't claim ownership), or the log
  path drifted, or `ctrace stop` raced another session's stop, the render is
  silently skipped — `|| true` swallows every failure and the hook exits 0. No
  signal, no docket report; the gap is only discovered the next morning.

## What this builds

A `mixed` change spanning `~/wintermute/ctrace-scribe` (the render engine) and
the hook script:

1. **Robust ownership resolution** — when the `claude-owns.json` marker is
   missing or stale, fall back to resolving the session's active log from
   `ctrace status` / the session id the hook already has, instead of giving up.
   The render should succeed whenever a renderable log exists, not only when the
   marker is pristine.
2. **A `ctrace-scribe render-session` entrypoint** in the rust crate that takes
   a session id (or log path) and renders idempotently — safe to call from the
   hook on exit *and* from backfill, same code path, so the two can't diverge.
3. **Demote backfill to fallback + signal on real miss.** The SessionEnd hook
   calls `render-session`; only if that genuinely fails (no log, render error)
   does it `docket report --key ctrace-sessionend-flake --severity warn
   --evidence session:<id>`. Backfill in the self-review then renders only true
   leftovers. When render-on-exit works, the self-review's backfill count for
   the day is 0 — the success signal.
4. **Never block shutdown.** Preserve the existing contract: the hook always
   exits 0, never delays session close (bounded timeout on render; if it can't
   finish fast, defer to backfill and report).

Constraints: SIGPIPE reset per [[self_sigpipe_panic_toolkit]]. rustc 1.85, no
let-chains. The fix must be idempotent — rendering the same session twice is a
no-op, never a double-write.

## Acceptance criteria

1. `cargo build --release` + `cargo test` green in `~/wintermute/ctrace-scribe`.
2. `ctrace-scribe render-session <id>` renders a summary for a fixture session
   log; calling it twice produces a byte-identical result (idempotent, no
   double-write).
3. With the marker file **deleted** but a valid session log present, the
   SessionEnd path still renders the summary (ownership-fallback verified) — the
   exact failure mode the current hook silently skips.
4. With **no** renderable log, the hook emits exactly one
   `docket report --key ctrace-sessionend-flake` line and exits 0 (does not
   block, does not error).
5. A successful render-on-exit emits **no** docket report (signal is the absence
   — the band-aid is no longer the steady state).
6. The hook exits 0 in every branch (marker-present, marker-missing,
   log-missing, render-error) — verified by exercising all four.
7. Backfill (`ctrace scribe backfill`) and `render-session` share one render
   code path (no divergent formatting between exit-render and backfill output).
