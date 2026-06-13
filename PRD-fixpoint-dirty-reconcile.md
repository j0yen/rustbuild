# PRD: fixpoint-dirty-reconcile — stop leaving dirty-tree repos in permanent limbo

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/fixpoint.md

## TL;DR

`adopt reconcile` skips every repo with a dirty working tree, so the ~30
chronically-dirty repos on this laptop can *never* get a lineage marker
minted — they stay permanently `clock-fallback` and keep the not-current
count off zero no matter how many cron ticks run. This PRD makes
reconcile handle dirty trees honestly: seed a marker from *committed HEAD*
when the installed binary matches that committed fingerprint, and
otherwise classify the repo as a distinct `dirty-blocked` reason instead
of silently inflating the stale count.

## Why this exists

Phase-1 live inspection, 2026-06-13 (`adopt reconcile --dry-run`):

```
skip  ac-judge          (dirty working tree)  repo=/home/jsy/wintermute/ac-judge
[dry-run] would seed  adopt  fp=e601fcb3…
skip  bon-mot-anagram   (dirty working tree)  repo=/home/jsy/wintermute/bon-mot-anagram
skip  bon-mot-epigram   (dirty working tree)
...
skip  christen          (genuinely behind HEAD)
```

Reconcile correctly seeds clean repos and correctly refuses
genuinely-behind ones — but for dirty trees it does neither: it skips,
leaving the binary on the `clock-fallback` basis forever. Self-review
reports ~30 dirty repos every run (recall reflective 2026-06-11/12/13);
the bon-mot-* binaries are dirty-skipped *and* show in `adopt verify` as
"0d newer" false-stale. The dirtiness is uncommitted *source* — it has no
bearing on what the already-installed binary was built from. A binary
built from committed HEAD is genuinely current even if someone later
edited the tree without committing.

## What this builds

In `~/wintermute/adopt` (`src/reconcile.rs`, marker logic in
`src/marker.rs`, reason enum in `src/types.rs`):

- When a working tree is dirty, compute the **committed HEAD fingerprint**
  (ignore the dirty/uncommitted delta) and check it against the installed
  binary's build provenance:
  - If the installed binary provably corresponds to committed HEAD (same
    rule reconcile already uses for the clean "not behind" case, applied
    to HEAD rather than the dirty tree), **seed the marker from committed
    HEAD**. The marker is authoritative thereafter; the uncommitted source
    is irrelevant to it.
  - If it does not (the binary predates HEAD, or provenance can't be
    established), **do not seed** — emit a distinct `dirty-blocked`
    classification so the repo stops counting as plain stale under a false
    `clock-fallback` label.
- Add a `--include-dirty` flag (default **on** for the seed-from-HEAD
  path; the conservative classify-only behavior is reachable via
  `--no-include-dirty` for auditing) so the cron and a human can choose.
- Surface `dirty-blocked` in `reconcile`'s output and ensure `scan`/`verify`
  can distinguish it from `clock-fallback` stale (no silent merge).

Scope guard: do not weaken the genuinely-behind refusal — a dirty tree
whose HEAD is behind the installed binary's source is still not seeded.
Reuse scion's existing fingerprint/marker code; do not fork it. MSRV 1.85,
no let-chains, `sigpipe::reset()` in `main`.

## Acceptance criteria

1. On a fixture repo with a dirty working tree but an installed binary
   matching committed HEAD, `adopt reconcile --execute` seeds a lineage
   marker (verified by `getfattr`/marker read or by a subsequent
   `adopt scan` reporting `freshness_basis: lineage` for that bin).
2. On a fixture repo with a dirty tree whose committed HEAD is *ahead* of
   the installed binary, reconcile does **not** seed and reports the bin
   as `dirty-blocked` (not `clock-fallback`, not silently skipped).
3. `--no-include-dirty` restores classify-only behavior: no markers seeded
   for any dirty tree, all reported `dirty-blocked`.
4. The genuinely-behind refusal is unchanged: a repo whose HEAD is behind
   the binary is never seeded regardless of dirty state.
5. `cargo test` green (fixtures are independent/held-out, not written by
   the same step that writes the rules), `cargo build --release` succeeds.
