# PRD: plumb-core — pair every detection probe with an independent ground-truth oracle

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/plumb
Vision: visions/plumb.md

## TL;DR

Self-review's Phase B.5 runs deterministic shell probes whose verdicts
park findings on the docket. At least three of those probes have
returned readings the system's real state contradicts. Nothing checks a
probe before trusting it. `plumb check <probe-id>` is the read-only
engine that, per probe, runs the probe's **verdict** and an **independent
oracle** for the same condition by a different mechanism, normalizes both,
and reports `agree | disagree | error` — the missing second measurement
that catches a lying instrument.

## Why this exists (Phase 1 evidence, 2026-06-13)

Measured live this session against probe source and today's journal:

- **The memlog probe is wrong right now.**
  `~/.claude/skills/self-review/SKILL.md:186`:
  `MEMLOG_GROUP=$(getent group memlog 2>/dev/null && echo yes || echo no)`.
  `getent group memlog` emits the group line (`memlog:x:NNN:jsy`) *then*
  `echo yes`, so the variable holds a multiline string; the gate at
  `SKILL.md:200` `[ "$MEMLOG_GROUP" = yes ]` is false → probe reports
  memlog **inactive while it is active**. Journal 2026-06-13 verbatim:
  *"memlog detection bug — getent+echo multiline capture breaks ACTIVE
  state check; real state IS active."*
- **The ctrace-wiring probe has read false-absent before.**
  `SKILL.md:644` `grep -q 'scribe backfill' "$HOOK"`. Journal 2026-06-12:
  *"Phase A ctrace-sessionend-flake probe was wrong (said wiring absent
  when it exists)."*
- **The adopt-report probe assumed a missing subcommand.** Journal
  2026-06-13: *"adopt `report` subcommand not yet implemented; manually
  reported to docket."* An independent `adopt report --help` exit-code
  check is ground truth.
- **No existing tool fills this gap.** `assay` re-runs the primitive a
  *fix* targeted; `warrant` proves a *close note*; `vigil`/`binstale`
  prove a *running binary*. All assume the *detector* is correct. This is
  the detector check none of them provide.

## What this builds

A new Rust CLI crate at `~/wintermute/plumb/`, installed to
`~/.local/bin/plumb`.

### Command surface

```
plumb check <probe-id>        # run one probe's verdict + oracle, compare
plumb check --all             # run every registered probe
plumb check <probe-id> --format json
plumb list                    # list registered probe-ids + condition names
```

### Probe registry — `~/.config/plumb/probes.toml`

Declarative; one `[[probe]]` per condition:

```toml
[[probe]]
id        = "memlog-active"
condition = "memlog group active for current user + /dev/memlog gid"
# The probe's OWN verdict logic, reproduced as run by self-review:
verdict   = '''MEMLOG_GROUP=$(getent group memlog 2>/dev/null && echo yes || echo no)
               [ "$MEMLOG_GROUP" = yes ] && echo active || echo inactive'''
# An INDEPENDENT second measurement by a different mechanism:
oracle    = '''( id -nG 2>/dev/null | tr " " "\n" | grep -qx memlog ) \
               && [ "$(stat -c %G /dev/memlog 2>/dev/null)" = memlog ] \
               && echo active || echo inactive'''
normalize = "trim_lower_firstword"   # collapse to the first whitespace token, lowercased
```

Seed `probes.toml` ships with the three known-bad probes:
`memlog-active`, `ctrace-backfill-wired`
(verdict = grep the hook; oracle = a second independent grep variant /
read of the hook), and `adopt-report-exists`
(verdict = assume present; oracle = `adopt report --help >/dev/null 2>&1`
exit code).

### Comparison semantics

- Run `verdict` and `oracle` each via `sh -c`, capturing stdout + exit.
- Apply `normalize` to each output (a small fixed set of named rules —
  `trim_lower_firstword`, `exit_code`, `nonempty` — not arbitrary code).
- Emit one of: `agree` (normalized outputs equal), `disagree` (both ran,
  outputs differ), `error` (either command exited nonzero in a way the
  normalize rule treats as failure).
- JSON: `{"probe":"memlog-active","verdict":"inactive","oracle":"active","result":"disagree"}`.

### Constraints

- Read-only: plumb runs commands the registry author declared; it does
  **not** mutate system state. (Registry authoring is trusted, same trust
  level as self-review's own SKILL.md.)
- SIGPIPE-safe: `sigpipe::reset()` as the first line of `main()` (per the
  toolkit-wide SIGPIPE panic lesson — `recall where | head` precedent).
- MSRV 1.85, no let-chains (matches the lib-crate toolchain bar).
- `--format json` is stable and is the contract `plumb-ledger` and
  `plumb-selfreview-bind` consume.

## Acceptance criteria

1. `plumb --version` prints a semver; `plumb --help` lists `check` and
   `list`.
2. With the seed `probes.toml` present, `plumb list` prints the three
   probe-ids and their condition strings.
3. `plumb check memlog-active --format json` runs both commands and emits
   a JSON object with `probe`, `verdict`, `oracle`, `result` keys.
4. Given a constructed fixture where verdict and oracle outputs differ,
   `result` is `"disagree"`; where they match, `result` is `"agree"`;
   where a command fails per its normalize rule, `result` is `"error"`.
5. `plumb check --all` runs every registered probe and exits non-zero if
   any probe's result is `disagree` (so a caller can gate on it); `0` when
   all agree.
6. A malformed `probes.toml` entry (missing `verdict` or `oracle`) is
   reported as a clear config error naming the probe-id, not a panic.
7. Piping `plumb check --all` into `head` does not panic (SIGPIPE reset
   verified).
8. The seed `memlog-active` probe, run on this laptop, returns
   `result: disagree` — reproducing the live bug as a regression anchor.
