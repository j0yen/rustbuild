# PRD: conning-tower-syndicate — route the day's column to its home (and optionally the zine)

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** (existing repo) `~/wintermute/conning-tower`
**Vision:** visions/conning-tower.md

## TL;DR

`conning-tower compose` renders a column to stdout or a path you name. That is
still a one-off artifact unless something decides *where* the column lives and
routes it there reliably, by date, idempotently. "The Conning Tower" was
*syndicated* — the same column placed into a standing slot every day. This PRD is
the syndication step: write the composed column into a canonical `columns/` slot
keyed by date, and, on a flag, drop a copy into the zine's input so the zine
finally has an answered artifact to print.

## Why this exists

`~/wintermute/conversations-zine`'s README is explicit that it is write-only at
the moment-extractor and that "layout/print/mail are explicitly downstream and
human-driven" — it has *no* answered input feeding its layout step. The umbrella
[roundtable.md](visions/roundtable.md) sets the goal that creativity "stops being
a pile of write-only artifacts" — syndication is the mechanism: a column that
reliably lands in a known slot can be picked up by `new-yorker` (issue assembly,
downstream) or the zine. `~/wintermute/daily-receipt` already demonstrates the
single-deterministic-artifact-per-day pattern this routes; the open question in
the umbrella ("published anywhere outward... default local") is answered here by
defaulting to a local `columns/` dir and leaving outward routing a deferred flag.

## What this builds

Extends `~/wintermute/conning-tower` (lib `conning_tower`, CLI `conning-tower`).

- **`syndicate.rs`** — given a rendered column and a date, write it to
  `<columns_dir>/YYYY-MM-DD.md` (default `~/wintermute/conning-tower/columns/`,
  overridable via `--columns-dir`/config). Idempotent: re-running for the same date
  overwrites unless `--no-clobber`, which errors if the slot exists.
- **`targets.rs`** — pluggable routing targets. Built-in targets: `columns` (the
  canonical local slot, always on) and `zine` (copy into
  `~/wintermute/conversations-zine`'s input dir, opt-in via `--to zine`, path via
  `--zine-input`, default skipped if the zine repo is absent — warn, don't fail).
- **CLI:**
  - `conning-tower syndicate [--date] [--ledger] [--columns-dir PATH] [--to
    columns,zine] [--no-clobber] [--dry-run]` — compose (reusing the column +
    contributors path) then route to each selected target. `--dry-run` prints the
    destination paths it *would* write and writes nothing.
- Deps: reuse existing; `fs` only (no new network dep — outward syndication stays
  deferred per the vision).

## Acceptance criteria

1. `conning-tower syndicate --ledger <fixture> --date 2026-06-08 --columns-dir
   <tmp>` writes `<tmp>/2026-06-08.md` whose contents are byte-identical to
   `conning-tower compose --out -` for the same inputs; the columns dir is created
   if absent.
2. Re-running the same syndicate command overwrites the slot by default and exits
   0; with `--no-clobber` and an existing slot it exits non-zero (exit 4) with a
   stderr message naming the existing path, and leaves the existing file unchanged.
3. `--to zine --zine-input <tmp2>` additionally writes the column into `<tmp2>`;
   with `--to columns` only (default), no zine copy is written.
4. `--to zine` when the zine input dir does not exist and was not explicitly passed
   prints a warning to stderr and still completes the `columns` target with exit 0
   (zine is best-effort, never fatal); an explicitly passed `--zine-input` that
   cannot be created is a hard error (exit 2).
5. `--dry-run` prints each destination path it would write, one per line, writes no
   files, and exits 0.
6. A date with no crowned record fails before any file is written (exit 3, same
   contract as `compose`); `--dry-run` on such a date also exits 3.
7. Each AC has a matching integration test under `tests/acceptance_ac<n>.rs` using
   committed fixtures and `tempfile`-style temp dirs; no test writes outside its
   temp dir.
