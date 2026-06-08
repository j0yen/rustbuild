# PRD: muster-census

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/muster
Vision: visions/muster.md

## TL;DR

The laptop runs several `claude` processes at once — one interactive, plus
headless timer runs (`/build`, `/dream`, `/self-review`) — and nothing renders a
roster of who is actually present. self-review falls back to `pgrep -af claude`
and cannot say where a PID came from. `muster census` is the missing roster: it
enumerates every live `claude` process, attributes each to an **origin class**,
reconciles it against its agorabus peer / worker subtree / ctrace tracer /
`agentns` session-id, and emits the result as human text and `--format json`.

## Why this exists

Captured live during the dream that drafted this PRD (2026-06-08), the process
table held three concurrent `claude` processes:

- pid 33958 — interactive, parent 4279 (a login tty), argv
  `claude --dangerously-skip-permissions`; its bus subtree is
  `agorabus subscribe --session-id claude-33958-jsy` (34174) + the
  `agorabus-worker.sh claude-33958-jsy` workers (34292/34339/34340).
- pid 402723 — `claude -p /self-review`, parent 402717
  (`claude-self-review-headless.sh`).
- pid 402724 — `claude -p /dream`, parent 402716
  (`claude-dream-headless.sh`) — the dreaming session itself.

Every fact needed to tell these apart is already in the process tree: the ppid
chain names the launcher, the argv names the role, the cgroup/scope names the
systemd unit, and the `claude-<pid>-jsy` peer ties each to the bus. But:

- `self-review/SKILL.md:70` is `pgrep -af claude` with *"note duplicates, do not
  kill"* — no origin attribution.
- The 2026-06-06 and -07 journals both list *"duplicate Claude sessions … may
  both be real"* under "Pending your call" — an unresolved guess, twice.
- `session-index` / `session-postmortem` / `session-trace-receipt` key on a
  session-id read from `~/.claude/projects/*.jsonl` (confirmed via their
  READMEs) — transcript forensics on sessions that already ran, not a live
  census.
- `agorabus peers` reconciles the bus side; `pevent list` reconciles supervised
  jobs; `ctrace-orphan-reap` reaps bpftrace tracers. None enumerate-and-attribute
  the live `claude` population.

So the roster is genuinely missing, and the live evidence for it is unambiguous.

## What this builds

A new Rust CLI `muster` at `~/wintermute/muster`, `cargo install`-able into
`~/.cargo/bin`, following the local-toolkit conventions
([[feedback_local_tools.md]]): `sigpipe::reset()` as the first line of `main()`
(it pipes to `head`/`jq` — see [[self_sigpipe_panic_toolkit]]); rustc 1.85, no
let-chains.

- **Enumeration.** Walk `/proc` for processes whose `comm`/exe resolves to the
  `claude` launcher (`~/.local/bin/claude`), excluding the agorabus
  subscriber/worker children (those are *part of* a session, not sessions). Read
  `/proc/<pid>/{stat,cmdline,cwd,cgroup,status}` directly — no `ps` shelling.
- **Origin attribution.** Classify each root `claude` PID:
  - `interactive-tty` — has a controlling tty / a shell-or-terminal ancestor and
    no `-p` argv.
  - `headless:<script>` — an ancestor is one of the `claude-*-headless.sh`
    wrappers; `<script>` is the wrapper basename (dream/self-review/build).
  - `timer:<unit>` — the cgroup/scope resolves to a `claude-*.service`/`.timer`
    systemd unit; `<unit>` is that unit.
  - `unknown` — none of the above (surfaced honestly, never silently bucketed).
  - When argv contains `-p /<skill>`, record the role (`dream`, `self-review`,
    `build`, …) regardless of class.
- **Reconciliation.** For each session, attach:
  - agorabus peer: match `claude-<pid>-jsy` against `agorabus peers` output;
    record present/absent.
  - worker subtree: the `agorabus subscribe` + `agorabus-worker.sh` child PIDs.
  - ctrace tracer: a live tracer session keyed to this session, if any
    (`ctrace status`).
  - `agentns` session-id: `/proc/<pid>/agent_session` when **non-zero**; while
    `agentns-session-zeros` is open ([[self_agentns_einval_flag_collision]]) this
    is expected empty, so census records `agent_session: null` and leans on the
    heuristic — written so the kernel id becomes preferred, additively, once the
    fix lands.
- **Output.** Default: one human-readable line per session (role, origin, pid,
  uptime, RSS, bus-peer ✓/✗). `--format json`: an array of roster objects with
  every reconciled field, stable key order for downstream `jq`.

Out of scope (later PRDs): any verdict/classification beyond origin
(muster-verdict), and any termination (muster-reap).

## Acceptance criteria

1. `muster census` run while ≥1 `claude` process is live prints one row per root
   `claude` session and does **not** list agorabus subscriber/worker children as
   their own sessions.
2. For a `claude -p /<skill>` process whose ancestor is a `claude-*-headless.sh`
   wrapper, the row's origin is `headless:<wrapper>` and the role is `<skill>`.
3. For an interactive `claude` with a tty ancestor and no `-p` argv, the row's
   origin is `interactive-tty`.
4. `muster census --format json` emits a JSON array; each element has at least
   `pid, origin, role, cwd, uptime_s, rss_bytes, agorabus_peer (bool),
   worker_pids (array), agent_session (string|null)`; output parses under `jq`.
5. A session with a live `claude-<pid>-jsy` agorabus peer shows
   `agorabus_peer: true`; one without shows `false` (verified by cross-checking
   `agorabus peers`).
6. `agent_session` is `null` when `/proc/<pid>/agent_session` is absent or
   all-zero, and the non-zero kernel value otherwise (the swap is additive — no
   code path assumes zero forever).
7. `muster census --format json | head -1` does not panic (SIGPIPE reset
   verified).
8. `cargo test` green; builds under rustc 1.85 with no let-chains. README
   documents the origin taxonomy and the `agentns`-degradation note.
