# PRD: roundtable-bind — bind the columns into an issue

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/roundtable
Vision: visions/roundtable.md
Depends-on: roundtable-session (produces the columns this binds)

## TL;DR

`roundtable session` produces one syndicated column per day. The *New Yorker*
Harold Ross founded from the Algonquin's very room was a periodical — columns
bound into issues with a masthead and a cover. `new-yorker` shipped the
issue-assembler, house-style enforcer, masthead, and generative cover, but
nothing gathers the accumulated daily columns and binds them. This PRD extends
`roundtable` with `roundtable bind [--since]`, which collects the columns
syndicated since the last issue and runs `new-yorker issue` + `cover` to bind a
periodical, idempotently per period.

## Why this exists

Verified 2026-06-15: the `new-yorker` repo is a workspace with
`newyorker-masthead`, `newyorker-house-style`, `newyorker-issue`, and
`newyorker-cover` all implemented (cover at "all 15 ACs green"). And
`conning-tower syndicate --to columns --columns-dir <dir>` routes each day's
column to a canonical columns slot. So the daily columns accumulate in a known
directory and the binder exists — but, exactly like the critique and games
stages before their roundtable wires, nothing connects "the pile of daily
columns" to "the issue assembler." roundtable-session established the columns
destination; this PRD consumes it.

## What this builds

Extend `~/wintermute/roundtable`:

- **`roundtable bind [--since YYYY-MM-DD] [--columns-dir <dir>] [--dry-run]`** —
  - Enumerate the columns in the columns directory (default the same slot
    roundtable-session/conning-tower writes to) dated on or after `--since`
    (default: since the last bound issue, tracked in roundtable's own state).
  - Run `new-yorker issue` to bind those columns into one issue, then
    `new-yorker cover` to stamp a generative cover; optionally
    `newyorker-house-style` over the assembled text if exposed as a CLI.
  - Record the issue (period + included column dates) in roundtable's state so
    the next `bind` starts after it.
- **Tool resolution** reuses roundtable-session's pattern (`$PATH` / `--bin-dir`
  / `ROUNDTABLE_NEWYORKER_BIN`); missing binary → actionable non-zero error.
- **Idempotent per period**: re-running `bind` for an already-bound period is a
  no-op that reports the existing issue (a test asserts this). `--since` lets the
  user re-bind a custom range explicitly.
- **Empty range**: if no columns exist since `--since`, report "no columns to
  bind" and exit 0 (not an error).
- `--dry-run` lists the columns that would be bound and the issue path without
  writing.

MSRV 1.85. Reuses roundtable-session's tool-resolution + state code. The binding,
masthead, cover, and house-style all stay in `new-yorker`.

## Acceptance criteria

1. `cargo test --release` passes; `cargo clippy -- -D warnings` passes.
2. `roundtable bind --dry-run --since 2026-06-01` lists the candidate columns
   and the intended issue path, mutating nothing.
3. With stub `new-yorker` binaries on `--bin-dir` and a fixture columns dir,
   `roundtable bind` selects the in-range columns and invokes the issue + cover
   stages — unit-tested against the fixture.
4. Re-running `bind` for an already-bound period is a no-op that reports the
   existing issue (a test asserts no second issue is produced and exit 0).
5. A missing `new-yorker` binary makes `bind` exit non-zero naming the binary.
6. `roundtable bind` with no columns in range reports "no columns to bind" and
   exits 0 (a test asserts the empty-range path).
