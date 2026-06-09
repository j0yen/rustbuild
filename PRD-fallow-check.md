# PRD: fallow-check — "has the field changed since my last harvest?"

Status: Draft v0.1
build_priority: normal
build_target: rust-extend
build_into: /home/jsy/wintermute/fallow
Vision: visions/fallow.md

## TL;DR

`fallow-fingerprint` gives dream a ledger and a deterministic digest of the
inward-signal corpus. `fallow-check` turns that into a **decision**: it
compares the current fingerprint against the last *productive* pass (the most
recent ledger record with `drafted > 0`) and answers, by exit code, whether
the evidence has actually moved. Exit 0 = fresh, go research and draft.
Exit 1 = fallow, the field is spent — and it reports the consecutive-saturation
streak so the caller can escalate deterministically instead of re-noticing in
prose for the ninth time.

## Why this exists

Phase 1 evidence, 2026-06-08 (full set in `visions/fallow.md`):

- The 5th–8th gossip notes of 2026-06-08 all independently re-conclude
  "drafting again = duplication" and "the honest move is to surface the
  outward choice to the user" — but each is a fresh prose re-derivation with
  no stored streak. The escalation they all describe never fires
  deterministically.
- `docket` (see [[docket]]) codifies exactly this missing primitive for
  self-review: SKILL.md's "a signal seen across 3+ separate runs justifies a
  durable playbook" was *eyeballed* across `recall query` output until docket
  gave it a streak counter. `fallow-check` is that counter for dream.

## What this builds

Extends the `fallow` crate (`~/wintermute/fallow/`) with one subcommand and
the streak logic. No new repo.

**`fallow check [--root <dir>] [--threshold <N>]`**

1. Compute the current fingerprint (reuse `fallow-fingerprint`'s logic — same
   corpus, same digest).
2. Read the ledger. Find the **last productive record** (most recent with
   `drafted > 0`). If none exists (fresh install), treat as fresh.
3. If current fingerprint ≠ last-productive fingerprint → **fresh**: print
   `fresh` (plus the differing corpus sections, for transparency) and exit 0.
4. If equal → **fallow**: compute the **streak** = number of consecutive
   most-recent ledger records that are saturated (`drafted == 0`) since the
   last productive pass, plus this evaluation. Print `fallow streak=<k>
   threshold=<N>` and exit 1.
5. `--threshold` (default 3) is echoed in output and surfaced via a third
   signal: when `streak >= threshold`, also print `escalate=true` so a caller
   can branch on it with a simple grep rather than arithmetic.

Add `--json` to `check` emitting `{"state":"fresh|fallow","streak":k,
"threshold":N,"escalate":bool,"fingerprint":"b3:…","last_productive_ts":"…"}`
for programmatic callers (the dream-wire step uses this).

**Exit-code contract (the load-bearing interface the wire depends on):**

| exit | meaning            |
|------|--------------------|
| 0    | fresh — draft      |
| 1    | fallow — rest      |
| 2    | usage/IO error     |

## Acceptance criteria

1. With a ledger whose last productive record's fingerprint **differs** from
   the current corpus, `fallow check` prints `fresh` and exits 0.
2. With a ledger whose last productive record's fingerprint **equals** the
   current corpus, `fallow check` prints `fallow` and exits 1.
3. The streak counts *consecutive saturated records since the last productive
   pass*: given ledger `[drafted=3, drafted=0, drafted=0]` and an unchanged
   fingerprint, `check` reports `streak=3` (two stored saturated passes + this
   evaluation). Covered by a test over a temp `--root` and a seeded ledger.
4. `--threshold 3` with `streak >= 3` emits `escalate=true`; with `streak < 3`
   emits `escalate=false` (or omits it). Default threshold is 3.
5. `fallow check --json` emits a single valid JSON object with `state`,
   `streak`, `threshold`, `escalate`, `fingerprint`, `last_productive_ts`.
6. Empty/absent ledger → treated as fresh, exit 0, no panic.
7. Exit code is **2** (not 1) on a genuine error (unreadable root, malformed
   `--threshold`), so callers can distinguish "fallow" from "broke".
8. `fallow check | head` does not panic (SIGPIPE).

## Out of scope

- Editing the dream skill to *consume* this contract (→ `PRD-fallow-dream-wire.md`).
- Auto-closing or expiring old ledger records (revisit if the ledger grows
  unwieldy; jsonl append is cheap and `show --limit` already bounds reads).
