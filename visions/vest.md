# Vision: vest — built things must actually become live, and prove it

> A binary that compiles is not a binary that runs. The laptop builds
> dozens of CLIs, installs them with `cargo install --force`, and
> assumes they landed. `vest` is the discipline of *vesting* —
> putting a built artifact into actual, verified possession of the
> running system, idempotently, with a receipt that says "yes, this
> one is live."

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-13
**Status:** active
**Seed:** bare `/dream` (interactive). fallow = fresh (streak=0).
Phase-1 live inspection of `adopt`, `adopt-cron.service` journal, and
the `/home/jsy/~/` junk tree.

---

## TL;DR

`adopt` is the laptop's tool for "shipped artifacts that never got
installed." It runs every 6h via `adopt-cron.service`. Yet self-review
has reported **84/84 tracked artifacts not-current** for run after run
(2026-06-11/12/13). The cure never converges. Phase-1 inspection found
*why*, and the why is not one bug but a cluster of adoption-integrity
gaps:

1. The cron's 02:42 run installed `ac-judge` to
   **`/home/jsy/~/.local/bin/`** — a literal `~` directory, off PATH —
   because a `--root ~/.local` reached `cargo` without tilde expansion.
   `cargo` does not expand `~`; it created a junk prefix. Nothing
   guards against this, and nothing cleans the 4.7M of debris it left.
2. `adopt apply` runs `cargo install --force` on every stale artifact
   **every 6h regardless of whether the source changed** — 3min 33s CPU
   and 1.1G memory peak per run (measured from the unit's own
   accounting), reinstalling binaries that are byte-identical to last
   time.
3. When an install lands in the wrong place (or anywhere off the
   convention dirs), adopt's only signal is a binary pass/fail smoke
   check that collapses every distinct failure mode — wrong-prefix,
   not-on-PATH, build-fail, smoke-fail — into one opaque "FAILED". So
   "84/84 stale" carries zero diagnostic information; the self-review
   re-parks it run after run with no way to act.
4. The systemd **user-manager PATH is `/usr/local/bin:/usr/bin`** — it
   excludes `~/.local/bin` and `~/.cargo/bin`. adopt happens to work
   around this (it probes the convention dirs directly), but any other
   user service that execs an adopted binary by name silently can't
   find it.

`vest` makes adoption *correct* (never writes outside the real prefix,
cleans what the bug left), *idempotent* (skip what's already current),
*legible* (a real failure taxonomy reported to the docket), and
*reachable* (the env actually resolves what was installed).

## Why this is real (Phase 1 evidence, 2026-06-13)

Measured live this session:

- **The junk prefix exists.** `find /home/jsy/~` →
  `/home/jsy/~/.local/bin/ac-judge` (+ `.crates.toml`,
  `.crates2.json`), `du -sh` = **4.7M**, all timestamped `Jun 13 02:43`
  — i.e. created by the 02:42 `adopt-cron` run.
- **The cron log proves the tilde path.** `journalctl --user -u
  adopt-cron.service`: `Installing /home/jsy/~/.local/bin/ac-judge` …
  `warning: be sure to add /home/jsy/~/.local/bin to your PATH`. Then
  `ac-judge FAILED: install exited 0 but --version / --help failed`
  (because `is_invokable` probes the *real* `~/.local/bin`, where the
  binary never landed).
- **The waste is metered.** Same unit: `Consumed 3min 33.285s CPU time
  over 42.369s wall clock time, 1.1G memory peak` — for a run that
  adopted *one* binary into the void and then aborted.
- **The backlog never converges.** self-review reflective memories
  (`01KTZYJZQY…` 2026-06-13, `01KTZS50DF…` 2026-06-12) both carry
  `adopt-scan-stale-binaries: 84/84 tracked artifacts not-current`.
- **The env gap.** `systemctl --user show-environment` →
  `PATH=/usr/local/bin:/usr/bin`. No `~/.local/bin`, no `~/.cargo/bin`.
- **The smoke check is the only signal.** `adopt/src/apply.rs:248-265`
  — on `cargo install` success it calls `is_invokable(&artifact.bin)`
  and, on miss, emits a single string `"install exited 0 but --version
  / --help failed"`. No bucketing of *why*.

## End-state

- `adopt apply` can never write outside `$HOME` — a `--root` carrying a
  literal `~`, or resolving outside `$HOME`, is rejected before exec.
- A single `adopt doctor` (or `scan` extension) detects mis-installed
  debris (the `/home/jsy/~/` tree) and reports it; an opt-in `--clean`
  removes adopt-created junk that duplicates a correctly-installed twin.
- `adopt apply` skips artifacts whose source is unchanged since the last
  successful install — the 6-hourly run becomes near-instant when
  nothing moved.
- Every not-current artifact carries a *reason bucket* (wrong-prefix /
  off-PATH / build-fail / smoke-fail / source-newer), reported to the
  docket so self-review acts on a category instead of re-parking "84/84".
- The systemd user PATH resolves `~/.local/bin` and `~/.cargo/bin`.

## Components (one bullet per future PRD)

- **vest-root-guard** — `adopt` validates/canonicalizes its install
  prefix, refuses literal-`~` or out-of-`$HOME` roots, and grows a
  `doctor` that detects + optionally cleans the `/home/jsy/~/` debris.
- **vest-verify** — a failure taxonomy: classify each not-current
  artifact into actionable buckets and report counts to the docket.
- **vest-incremental** — content/mtime marker per artifact; skip
  `cargo install --force` when the source is unchanged since the last
  successful vest.
- **vest-path** — add `~/.local/bin` and `~/.cargo/bin` to the systemd
  user-manager PATH via `environment.d`, so user services resolve
  adopted binaries.

## Order

```
vest-root-guard ──► vest-verify ──► vest-incremental
vest-path  (independent — config only, ship anytime)
```

`vest-root-guard` first: stop the bleeding and clean the debris before
anything else reasons about install state. `vest-verify` next: it needs
the guard's notion of "wrong-prefix" to bucket correctly.
`vest-incremental` builds on a clean, verified baseline. `vest-path` is
a standalone config change with no code dependency.

## Open questions

1. **Cleanup safety.** Should `adopt doctor --clean` ever `rm` the
   `/home/jsy/~/` tree autonomously, or only when each junk binary has a
   verified correctly-installed twin? (PRD ships the conservative
   twin-checked form; full autonomous rm stays user-gated.)
2. **Incremental marker location.** Per-artifact hash marker under
   `~/.local/state/adopt/` vs. reading the installed binary's build
   metadata. (PRD-vest-incremental proposes the state-dir marker;
   revisit if it proves fragile across rebuilds.)
3. Was the tilde-root a since-fixed binary bug or environmental? The
   03:33 `adopt` rebuild dry-runs the *expanded* path, so the immediate
   cause may already be gone — but nothing *prevents recurrence* or
   *cleans the debris*, which is what vest-root-guard exists to do
   regardless.
