# PRD: fallow-fingerprint — a ledger and a digest for dream's saturation state

Status: Draft v0.1
build_priority: normal
build_target: rust-cli
build_into: /home/jsy/wintermute/fallow
Vision: visions/fallow.md

## TL;DR

`/dream` has no memory of its own saturation. It re-derives "the inward
signal hasn't changed" from scratch every pass, in prose, and parks it in
gossip — 30 such notes accumulated, 8 in a single day (2026-06-08).
`fallow-fingerprint` ships the substrate that ends this: a tiny `fallow`
CLI that (a) computes a **deterministic digest of the inward-signal corpus**
dream actually reads, and (b) records each dream pass's outcome to a ledger.
With a stored fingerprint per pass, a later step (`fallow-check`) can answer
"did the evidence move since my last productive pass?" with an exit code
instead of a model judgment.

## Why this exists

Phase 1 evidence, 2026-06-08 (see `visions/fallow.md` for the full set):

- `grep -cE "no PRDs drafted|saturation scan" ~/wintermute/autobuilder/notes/gossip.md`
  returns **30**. The 5th–8th gossip notes of 2026-06-08 are near-verbatim
  restatements of each other.
- Each no-op pass is a full Claude invocation: today's journal records the
  active dream/build session at 79m36s / 51,373 writes / `sed`×97,812.
- `docket` (see [[docket]]) already proved the fix for the self-review side
  of this anti-pattern: a structured store keyed by a stable slug, tracking
  occurrence and streak, beats re-eyeballing prose each run. `fallow` is the
  dream-side equivalent and starts with the same primitive — a durable
  ledger plus a deterministic key.

## What this builds

A new Rust CLI crate at `~/wintermute/fallow/` (`fallow` binary), following
the local-toolkit conventions (`sigpipe::reset()` first line of `main()` per
[[self_sigpipe_panic_toolkit]]; MSRV 1.85; no let-chains).

**Store.** `~/.claude/fallow/ledger.jsonl` — append-only, one JSON object per
dream pass:

```json
{"ts":"2026-06-08T20:30:00Z","drafted":3,"fingerprint":"b3:1a2f…","seed":"user-prompt","note":"vision-fallow"}
```

(`ts` is supplied by the caller via `--ts` or defaults to the system clock at
the CLI boundary — fine here; the no-`Date::now` rule applies to workflow
scripts, not shipped binaries.)

**Subcommands.**

- `fallow fingerprint [--root <dir>]` — compute and print the inward-signal
  digest. The corpus (v1):
  - open PRD slug set: filenames matching `PRD-*.md` under the autobuilder
    root (names only, sorted — content churn inside a PRD must not flip it).
  - the journal "Pending your call" block: the section under that heading in
    the most recent `~/brain/journal/YYYY-MM-DD.md`.
  - open-docket slugs: if `docket` is present, the set of non-closed docket
    slugs (best-effort; absence is not an error — emit the rest).
  - the gossip last-drafted marker: the most recent line in `gossip.md`
    matching `Drafted:`.
  Concatenate canonically (sorted, newline-joined, section-labelled), hash
  with BLAKE3, print `b3:<hex>`. Deterministic: same inputs → same digest.
- `fallow record --drafted <N> [--seed <s>] [--note <s>] [--ts <iso>]` —
  compute the current fingerprint and append a ledger record. Prints the
  record's fingerprint.
- `fallow show [--limit <N>]` — print the last N ledger records as a table
  (ts, drafted, short-fingerprint, note). Default 10.

**Deps.** `blake3`, `serde`/`serde_json`, `clap` (derive), `anyhow`,
`sigpipe`. No network, no async.

## Acceptance criteria

1. `cargo build --release` produces a `fallow` binary; `fallow --help` lists
   `fingerprint`, `record`, `show`.
2. `fallow fingerprint` prints a `b3:`-prefixed hex digest and exits 0.
   Running it twice with no intervening filesystem change to the corpus
   prints the **identical** digest (determinism test).
3. Editing the *body* of an existing `PRD-*.md` (without renaming it) does
   **not** change the fingerprint; adding or removing a `PRD-*.md` file
   **does** change it. (Covered by a test using a temp `--root`.)
4. `fallow record --drafted 3 --note vision-x` appends exactly one line to
   `~/.claude/fallow/ledger.jsonl`, valid JSON, containing `drafted:3`, a
   `fingerprint`, and a timestamp.
5. `fallow show --limit 5` prints at most 5 records, most-recent last (or
   clearly ordered), without panicking on an empty or absent ledger.
6. Piping any subcommand's output into `head` does not panic (SIGPIPE handled).
7. A missing journal "Pending your call" block, missing docket, or missing
   gossip marker degrades gracefully: the corresponding corpus section is
   treated as empty, the command still succeeds, and this is covered by a test.

## Out of scope

- The `check`/streak/exit-code contract (→ `PRD-fallow-check.md`).
- Any edit to the dream skill (→ `PRD-fallow-dream-wire.md`).
- Gating the systemd timer (Fleet 2 bullet in the vision).
