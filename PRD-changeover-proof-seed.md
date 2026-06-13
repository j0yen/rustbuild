# PRD: changeover-proof-seed

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/rollout
Vision: visions/changeover.md
Depends: PRD-changeover-daemon-claims.md

## TL;DR

The problem: `rollout apply --auto` (v0.7.0) consults a per-daemon proof
ledger at `~/.config/rollout/proofs.json` and refuses any daemon without
a fresh green proof. That file does not exist — no one has ever run
`changeover probe` and fed it to `rollout record-proof` — so auto-apply
refuses everything and the fleet stays stale even though all the
machinery is built. Producing a proof today is a two-binary, two-step
manual dance (`changeover probe … --json` then `rollout record-proof
--from -`). This PRD adds a single `rollout prove --daemon <unit>` that
runs the whole cycle, plus a low-frequency systemd-user timer that keeps
the ledger fresh — re-proving a daemon whenever its installed binary hash
changes. After this lands, the gate has real green entries to gate on.

## Why this exists

Phase-1 research, 2026-06-13:

- `~/.config/rollout/proofs.json` is **absent** (`find` over
  `~/.config`/`~/.local/state` finds only `adopt/markers/changeover.json`,
  unrelated). So the v0.7.0 ledger the gate reads has never been written.
- `~/wintermute/rollout/CHANGELOG.md` v0.7.0: `rollout record-proof --from
  <probe-json>` ingests `changeover probe` output and `apply --auto`
  "skips any [daemon] with a Refuse verdict (no matching binary hash or
  events lost > 0)." A missing ledger ⇒ every daemon is Refuse ⇒ nothing
  auto-applies.
- `changeover` (binary on PATH at `~/.local/bin/changeover`) produces the
  probe JSON; the two tools are already designed to compose but nothing
  runs the composition.
- The gate is binary-hash-bound (vision Fleet-1 open question resolved to
  "hash-bound"): a proof is invalid once the daemon's exe changes, so the
  ledger must be re-seeded after every rollout — a one-shot manual seed is
  not enough; freshness needs a timer.

## What this builds

In `~/wintermute/rollout`:

- A `rollout prove --daemon <unit>` subcommand that:
  1. invokes `changeover probe` for `<unit>` capturing its JSON,
  2. feeds that JSON to the existing `record-proof` ingestion path,
  3. writes/updates the per-daemon entry in
     `~/.config/rollout/proofs.json`,
  4. prints the resulting verdict (Allow/Refuse + window ms + events lost
     + binary hash) and exits non-zero if the proof is Refuse, so a timer
     or human can tell prove-failed from prove-green.
  `--all` proves every daemon in the fleet recipe; `--dry-run` runs the
  probe and prints what *would* be recorded without writing the ledger.
- A systemd-user timer + oneshot service (`changeover-prove.timer` /
  `.service`) installed under `~/.config/systemd/user/` that runs
  `rollout prove --all` on a low frequency (default daily, off-peak), and
  re-proves on demand when a daemon's installed binary hash differs from
  its recorded proof hash (stale-hash detection in the service or a small
  wrapper).
- Reuse rollout's existing `autogate`/`record-proof` types; do not fork
  the ledger format.

Out of scope: flipping `apply --auto` on (that is `changeover-activate`),
the self-review guardrail edit (the blocked
`PRD-rollout-selfreview-apply.md`), the probe implementation itself
(already shipped in `changeover`).

## Acceptance criteria

1. `rollout prove --daemon <unit>` runs `changeover probe` for that unit,
   ingests the JSON via the existing record-proof path, and writes/updates
   `~/.config/rollout/proofs.json` with a per-daemon entry (binary hash +
   window + events-lost + verdict).
2. `rollout prove` exits zero on a green (Allow) proof and non-zero on a
   Refuse proof, so a timer can distinguish them.
3. `rollout prove --all` proves every daemon in the fleet recipe;
   `rollout prove --dry-run` prints the would-record proof without writing
   the ledger (verifiable against a fixture probe JSON).
4. The recorded proof is binary-hash-bound: a test (or documented manual
   check) shows that changing the daemon's exe hash invalidates the prior
   proof so the timer re-proves it.
5. A `changeover-prove.timer` + oneshot `.service` are provided under
   `~/.config/systemd/user/`, run `rollout prove --all`, and are
   `systemctl --user --dry-run`-installable without restarting any daemon
   (prove is read/measure-only beyond the probe's own controlled restart).
6. After running `rollout prove --all` once against the live fleet (post
   daemon-claims), `~/.config/rollout/proofs.json` exists with one entry
   per fleet daemon (acceptance may assert against a fixture if the live
   probe is unavailable in the build environment).
7. `cargo test` green; no new clippy warnings beyond the documented red
   baseline; CHANGELOG documents `prove` and the timer.
