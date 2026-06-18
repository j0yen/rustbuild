# PRD: trim-attribute — classify who holds the memory

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/trim
Vision: visions/trim.md

## TL;DR

trim-survey enumerates and ranks memory holders, but a flat ranking
can't tell relief-eligible idle daemons apart from sacrosanct
user-facing apps. trim-attribute classifies every holder into a named
class — `wintermute-daemon`, `claude-session`, `local-llm`, `build`,
`user-app`, `system` — using systemd unit identity (and provfs /
agentns session id where available), and tags each with a relief
*candidacy* that policy will later turn into a decision. Read-only;
extends the `trim` crate.

## Why this exists

Live evidence (this laptop, 2026-06-18):

- The top swap/RSS holders span every class at once:
  `homeward-embed-svc` (daemon, 476 MB swap), `recalld` (daemon,
  237 MB), `claude` ×3 (session), `firefox`/`slack` (user-app),
  `rustc` ×3 (build), `watchman` (system). A relief tool that treats
  these uniformly would either do nothing useful or do something
  dangerous (restart Firefox to "save memory").
- The dangerous-to-touch and safe-to-touch holders are
  distinguishable by *systemd unit identity*: `systemctl --user
  list-units` shows `homeward-embed.service`, `recalld.service`,
  `agorabus.service`, `wmd.service`, `wm-dialog.service` as managed
  units; Firefox/Slack/rustc are not user-managed daemons.
- The kernel now stamps provenance: provfs `user.prov.session`
  (comm-chain / agentns-id form) and `/proc/<pid>/agent_session`
  exist (see vision-colophon, vision-assay) — usable to attribute a
  `claude` PID to a session when present, with graceful fallback when
  agentns is inert (CLONE_NEWAGENT EINVAL, run 17+, per self-review).

## What this builds

Extends the `trim` library + CLI.

**Library:**
- `class::HolderClass { WintermuteDaemon, ClaudeSession, LocalLlm,
  Build, UserApp, System }`.
- `attribute::classify(proc: &ProcMem, units: &UnitMap) -> HolderClass`:
  - resolve `/proc/<pid>/cgroup` → systemd unit (parse the
    `…/<unit>.service` leaf under `user.slice`); cache a
    `UnitMap` from `systemctl --user show -p Id,Names` once per run.
  - `wintermute-daemon` if the unit matches the fleet allowlist
    (`agorabus|recalld|wmd|wm-*|homeward-*` …) — sourced from the unit
    list, not hardcoded comm strings.
  - `local-llm` if comm/unit is `ollama*`.
  - `build` if comm ∈ {rustc, cargo, cc, ld, sccache}.
  - `claude-session` if comm is `claude`/`node` under a claude session
    cgroup; attach session id from `/proc/<pid>/agent_session` when
    non-zero, else from provfs / cmdline heuristic.
  - `user-app` for known desktop apps (firefox, slack, electron, …);
    `system` otherwise.
- `attribute::candidacy(class, proc) -> Candidacy { eligible_maybe,
  reason }` — a *hint* only (e.g. "wintermute-daemon + 476MB swap →
  candidate"); the binding decision is trim-policy's.

**CLI:**
- `trim survey` rows gain a `class` column.
- `trim attribute --format json` → per-holder `{pid, comm, class,
  unit, session_id?, candidacy}`.
- `--class <name>` filters to one class.

## Acceptance criteria

1. `trim attribute --format json` tags every surveyed holder with one
   of the six `HolderClass` values; no holder is left unclassified
   (`system` is the catch-all).
2. On this box, `homeward-embed.service` and `recalld.service`
   classify as `wintermute-daemon` with a non-empty `unit`; `firefox`
   and `slack` classify as `user-app`; `rustc` classifies as `build`.
3. Classification uses systemd unit identity (cgroup → unit), not a
   hardcoded comm allowlist alone; a daemon renamed in its unit file
   is still attributed by unit. (Test with a fixture cgroup path.)
4. When `/proc/<pid>/agent_session` is present and non-zero, the
   session id is attached to `claude-session` holders; when agentns is
   inert (all-zero / ENOENT), the field is absent and classification
   still succeeds (no panic, graceful fallback).
5. `candidacy.eligible_maybe` is true for idle wintermute daemons above
   the swap floor and false for every `user-app` / `build` /
   `local-llm` holder — but it is explicitly labeled a hint, and
   carries no authority to act (no relief code in this PRD).
6. Read-only: the change issues no signals, no restarts, no writes to
   `/sys` or `/proc`. `trim attribute` against the live system mutates
   nothing.
7. `cargo test` green: fixture-based tests for cgroup→unit parsing and
   class assignment across all six classes. (Build via /cloudbuild.)
