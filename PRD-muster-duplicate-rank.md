# PRD: muster-duplicate-rank

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/muster
Vision: visions/muster.md

## TL;DR

When several interactive sessions share a project slug, `muster verdict` flags
*all* of them `duplicate` with identical reasons — useless for picking out the
genuine zombie. This PRD ranks same-slug duplicates by activity recency so
exactly the freshest stays `live` and the idle-older ones become
`duplicate (idle <N>s)`, naming the one that should actually go.

## Why this exists

Verified live (2026-06-16): four interactive sessions all run from `/home/jsy`,
so all four resolve to project slug `jsy` and all four are flagged `duplicate`:

```
1054     interactive-tty  duplicate  296277s   duplicate … slug 'jsy'
949440   interactive-tty  duplicate   41771s   duplicate … slug 'jsy'
1147375  interactive-tty  duplicate   34435s   duplicate … slug 'jsy'
1195852  interactive-tty  duplicate   33669s   duplicate … slug 'jsy'
```

The verdict cannot say which is the zombie. It plainly is 1054 — 3.4 days old,
idle, dragging 21 leaked workers — but muster gives it the same verdict as three
live shells. The vision's open question ("is cwd the right key … confirm
against a real duplicate when one recurs") has recurred four-fold, live, so the
disambiguation is now research-motivated, not speculative.

## What this builds

Extends `~/wintermute/muster/src/verdict.rs` (and a small activity probe):

1. **Activity recency per session**, most-recent-wins of:
   - transcript mtime under `~/.claude/projects/<slug>/*.jsonl` for the session,
   - ctrace last-event time if a session ndjson exists,
   - falling back to process start (uptime) when neither is available.
2. **Rank within each duplicate group** (same resolved slug, same origin class
   `interactive-tty`): the single most-recently-active entry keeps verdict
   `live`; the rest stay `duplicate` with reason `duplicate (idle <N>s)` where
   `<N>` is seconds since that session's last activity.
3. Singletons (only one session for a slug) are unaffected — they were never
   `duplicate`.
4. JSON gains `last_activity_unix` and, for duplicates, `idle_s`; text reason
   column carries the idle figure.

Deps: none new; reuses census roster + filesystem mtime reads.

## Acceptance criteria

1. Given N>1 same-slug `interactive-tty` sessions, exactly one is verdict
   `live` (the most-recently-active) and the rest are `duplicate` with an
   `idle_s` value (unit test with 4 sessions, distinct activity times → only
   the freshest is `live`).
2. `idle_s` for a duplicate equals (now − its last_activity_unix), where
   last_activity is the max of transcript mtime / ctrace last-event / start
   time; tested across each source being the most recent.
3. A slug with a single session is never marked `duplicate` (regression).
4. Sessions in *different* slugs are never ranked against each other (two
   single-session slugs both stay `live`).
5. Missing transcript/ctrace data degrades to uptime without error (test).
6. Existing verdict tests for non-duplicate kinds (orphan/stale) still pass;
   `cargo test` green; `cargo build` clean.
