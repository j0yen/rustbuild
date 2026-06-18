# PRD: trim-survey — honest memory & swap pressure enumerator

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/trim
Vision: visions/trim.md

## TL;DR

Self-review reports "swap 5.4G/8G — heavy, monitor; not actionable
today" in every journal, with no attribution of *who* holds the pages.
trim-survey is the honest enumerator: it walks `/proc/meminfo`, every
`/proc/<pid>/status`, the cgroup-v2 memory surfaces, and
`/proc/pressure/memory`, then attributes resident and swapped bytes to
process, systemd unit, and cgroup — emitting JSON for tooling and a
ranked table for humans. It is the foundation of the trim vision (a
NEW repo at `~/wintermute/trim`).

## Why this exists

Live evidence (commands reproducible today on this laptop):

- `free -h` → `Swap: 8.0Gi total, 5.4Gi used`. `/proc/pressure/memory`
  → `full total=337243465` (337 ms-seconds of cumulative full stall) —
  the box *has* thrashed, even when `avg10=0.00` right now.
- Per-process `VmSwap` walk
  (`awk '/VmSwap/{...}' /proc/*/status | sort -rn`) →
  `homeward-embed-svc` **476 MB swapped**, `recalld` **237 MB**,
  two `claude` sessions ~130 MB + ~101 MB, two `python` ~93 MB each.
- Top RSS: `watchman` 663 MB, three `claude` 583/411/335 MB,
  `firefox` 324 MB, three `rustc` ~250 MB each.
- The swap-thrashers are systemd user units (`homeward-embed.service`,
  `recalld.service`) — i.e. they have a stable identity a tool can name.
- Self-review journals 2026-06-12..18 repeat "swap … not actionable" —
  the finding has recurred ≥6 times with no owner, because there is no
  tool that attributes the pressure. (Disk has `ballast`/`careen`/
  `drydock`; memory has nothing — see vision §TL;DR.)

## What this builds

A new Rust workspace at `~/wintermute/trim` (rust-cli + lib crate),
SIGPIPE-reset in `main()` per the local-toolkit convention.

**Library (`trim` crate):**
- `mem::MemInfo` — parse `/proc/meminfo` (MemTotal/MemAvailable/
  SwapTotal/SwapFree/Buffers/Cached).
- `proc::ProcMem { pid, comm, rss_kb, swap_kb, cgroup }` — parse
  `/proc/<pid>/status` (`VmRSS`, `VmSwap`, `Name`) + `/proc/<pid>/cgroup`.
  Skip kernel threads (no `VmRSS`).
- `cgroup::CgroupMem { path, current, swap_current, pressure_some_avg60 }`
  — read cgroup-v2 `memory.current`, `memory.swap.current`,
  `memory.pressure` under `user.slice`.
- `psi::Psi { some_avg10, some_avg60, some_total, full_avg10, ... }` —
  parse `/proc/pressure/memory`.
- `Survey { meminfo, procs: Vec<ProcMem>, cgroups, psi }` with a
  `rank_by_swap()` and `rank_by_rss()`.

**CLI (`trim survey`):**
- `trim survey` → ranked human table (top-N by swap, then by RSS) + a
  PSI summary line + total swap/RSS.
- `trim survey --format json` → the full `Survey` as JSON.
- `--top <N>` (default 15), `--by swap|rss` (default swap).
- Read-only. No writes, no process signals. Pure observation.

Deps: keep minimal — `serde`/`serde_json`, `clap`, `anyhow`. No procfs
crate dependency unless it earns itself; hand-parsing `/proc` is cheap
and avoids version churn (MSRV 1.85, no let-chains).

## Acceptance criteria

1. `trim survey --format json` emits valid JSON containing `meminfo`,
   `psi`, and a non-empty `procs` array; each proc entry has `pid`,
   `comm`, `rss_kb`, `swap_kb`.
2. The summed `swap_kb` across `procs` is within 10% of
   `SwapTotal - SwapFree` from `/proc/meminfo` (sanity: attribution
   roughly accounts for used swap).
3. `trim survey` (human) prints a ranked table where the top swap
   holder on a thrashing box (e.g. `homeward-embed-svc`) appears at or
   near the top, plus a PSI line showing `some`/`full avg60` and the
   cumulative `total`.
4. `--by rss` re-ranks by resident set; `--top N` bounds the row count
   to N.
5. Kernel threads (entries with no `VmRSS`) are excluded; the tool does
   not panic on PIDs that vanish mid-walk (race-safe: skip on ENOENT).
6. The binary resets SIGPIPE at the start of `main()` so
   `trim survey | head` does not panic (per local-toolkit convention).
7. `cargo test` green: unit tests parse fixture `/proc/meminfo`,
   `/proc/<pid>/status`, and `/proc/pressure/memory` blobs and assert
   the parsed fields. (Build via /cloudbuild per the standing rule.)
