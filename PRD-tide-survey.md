# PRD: tide-survey

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/tide
Vision: visions/tide.md

## TL;DR

The laptop's pacman update queue has grown to 101 packages and self-review
reports it as "BLOCKED — needs a reboot window" every single run, hand-counting
the queue and hand-classifying which packages force a reboot. There is no tool
that renders this honestly. `tide survey` is the foundation of the tide vision:
a read-only CLI that enumerates the pending update queue, partitions it into
*reboot-requiring* vs *safe-to-apply-live*, surfaces the booted-vs-installed-vs-
available kernel skew, and emits a single overall verdict —
`current | updates-pending | reboot-pending | reboot-required` — as human text
and `--json`.

## Why this exists

Captured live during the dream that drafted this PRD (2026-06-08):

- The 2026-06-06, -07, and -08 self-reviews each list, verbatim under "Pending
  your call," **"pacman 101 updates BLOCKED"** with the same protected set
  (`linux 7.0.9→7.0.11`, `linux-firmware*` ×12, `glibc`/`systemd`/`mesa`). The
  queue grew **29 → 101** in two days (journal 2026-06-08: *"pacman queue grew
  from 29 (2026-06-06) to 101 (today) — 72 new packages"*).
- `mend.md:51` tags `pacman-kernel-update-blocked` as *"user-gated reboot, not
  buildable"* — recognized but deliberately left uncovered by any PRD.
- `command -v checkupdates needrestart reflector` returned **nothing** this pass
  — the standard Arch update-introspection tools are all absent. self-review's
  count is therefore hand-rolled each tick with no reusable primitive behind it.
- Live version skew, verified: booted kernel `uname -r` =
  `7.0.10-arch1-5-wintermute` (a custom `linux-wintermute` build), `pacman -Q
  linux` = `7.0.9.arch1-1`, queue wants `7.0.11`. Three different versions,
  rendered nowhere — "what would a reboot actually change" is guesswork.

So a structural, honest survey of the update/reboot state is genuinely missing,
and the evidence motivating it is unambiguous and recurring.

## What this builds

A new Rust CLI `tide` at `~/wintermute/tide`, `cargo install`-able into
`~/.cargo/bin`, following the local-toolkit conventions
([[feedback_local_tools.md]]): `sigpipe::reset()` as the **first line of
`main()`** (it pipes to `head`/`jq` — see [[self_sigpipe_panic_toolkit]]); rustc
1.85, **no let-chains**; cloud-build-safe (no network during `cargo test`).

This PRD creates the workspace + binary and the survey subcommand only; later
PRDs (`tide-restart`, `tide-window`, `tide-landfall`) rust-extend into this repo.

### Modules

- `model.rs` — core types reused by the whole vision:
  - `RebootClass` = `RebootRequiring | SafeToLive` and the curated
    reboot-requiring name/prefix set (`linux`, `linux-firmware`,
    `linux-api-headers`, `glibc`, `systemd`, `mesa`, plus their `lib*`/
    `*-headers` siblings).
  - `PendingUpdate { name, installed_ver, available_ver, class }`.
  - `KernelSkew { booted: String, installed_pkg: String, available_pkg:
    Option<String> }` from `uname -r`, `pacman -Q linux`, and the queue.
  - `UpdateState { pending: Vec<PendingUpdate>, kernel: KernelSkew, verdict }`.
  - `Verdict` = `Current | UpdatesPending | RebootPending | RebootRequired`
    with a documented decision rule (see AC4).
- `pacman.rs` — the read-only queue reader. Implements the `checkupdates`
  pattern: sync the pacman sync-db into a **private** `--dbpath` under
  `$XDG_CACHE_HOME/tide/checkup-db` (seeded from `/var/lib/pacman/sync` via a
  copy of the `local` db link), run `pacman -Sy --dbpath <cache>` then
  `pacman -Qu --dbpath <cache>`, parse the `name old -> new` lines. **Never**
  touches `/var/lib/pacman`. If the private-db path can't be built, fall back to
  parsing `pacman -Qu` against the existing synced db (best-effort; flag
  `stale: true` in output). All parsing is fixture-driven and unit-tested
  offline; the live `pacman -Sy` is only exercised by the `survey` runtime, not
  by `cargo test`.
- `render.rs` — human table + `--json` (serde). Human form groups by
  `RebootClass` and prints the kernel skew line and the overall verdict last.
- `main.rs` — clap; `tide survey [--json]`.

### UX

```
$ tide survey
verdict: reboot-required
kernel:  booted 7.0.10-arch1-5-wintermute · installed linux 7.0.9 · available 7.0.11
reboot-requiring (4): linux 7.0.9→7.0.11 · glibc … · systemd … · mesa …
safe-to-apply-live (97): foo 1.2→1.3 · bar …
$ tide survey --json | jq .verdict
"reboot-required"
```

### Dependencies

`clap`, `serde`/`serde_json`, `sigpipe`. No network crates. pacman invoked via
`std::process::Command`.

## Acceptance criteria

1. `cargo build --release` produces a `tide` binary; `cargo install --path .`
   places it in `~/.cargo/bin`. `tide --help` lists the `survey` subcommand.
2. `sigpipe::reset()` is the first statement in `main()`; `tide survey --json |
   head -1` does not panic or print a `BrokenPipe` backtrace.
3. The pacman queue parser is unit-tested against **fixture** `pacman -Qu`
   output (a captured multi-line sample incl. `linux`, `glibc`, and ordinary
   packages) and correctly partitions each into `RebootRequiring` /
   `SafeToLive`. Tests run fully offline (no `pacman -Sy`).
4. The `Verdict` decision rule is implemented and unit-tested per these cases:
   empty queue → `Current`; queue with only `SafeToLive` and no kernel skew →
   `UpdatesPending`; booted-kernel ≠ installed-`linux` (a pending reboot already
   earned) → `RebootPending`; queue contains ≥1 `RebootRequiring` package →
   `RebootRequired`.
5. `KernelSkew` is populated from `uname -r`, `pacman -Q linux`, and the queue's
   `linux` entry (available = `None` when `linux` isn't in the queue), and the
   skew line renders all three values.
6. `tide survey --json` emits a stable schema (`verdict`, `kernel{booted,
   installed_pkg,available_pkg}`, `pending[]{name,installed_ver,available_ver,
   class}`, `stale`) validated by a serde round-trip test.
7. The private-dbpath reader never writes under `/var/lib/pacman`; on failure to
   build the private db it falls back to `pacman -Qu` and sets `stale: true`
   (asserted via an injected reader error in tests).
8. `README.md` documents the verdict rule, the reboot-requiring set, and the
   `checkupdates`-pattern private-db read; notes rustc 1.85 / no let-chains and
   that the system db is never mutated.
