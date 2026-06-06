# PRD: quicken-attest

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/quicken
Vision: visions/quicken.md

## TL;DR

The reason dark primitives rot is that the self-review escalates each one
"once" and then goes silent — so the same finding reappears boot after boot with
no sense of *how long* it's been broken or whether it's getting worse.
`quicken-attest` extends the `quicken` workspace to persist a timestamped
liveness receipt, compute a delta against the previous one, and keep a monotonic
per-primitive **inert-streak counter** so a primitive that stays dark gets
*louder* over consecutive boots instead of fading into the noise.

## Why this exists

- **Evidence — escalate-once-then-silent is the actual failure mode.** Every
  self-review since 2026-05-24 carries an identical "Pending your call" block:
  memlog staged, agentns zeros, warden inert. The 2026-06-03 journal even notes
  "Escalated once; will not repeat until pkgrel changes or state flips." The
  result is silent rot — the memlog pkgrel gap *widened* 5→10→11 while each
  review stayed quiet because nothing had "flipped."
- **Evidence — drift is invisible without a stored prior.** The only reason we
  know the gap widened is a human eyeballing two journals weeks apart. A
  persisted receipt + delta makes "memlog gap 5→11, inert for N boots" a
  computed fact.
- **Why a receipt, not a log line:** the wintermute ecosystem already favors
  structured receipts (`session-trace-receipt`, autobuilder receipts, memory
  `self_autobuilder_receipt_order`). A `quicken` receipt fits that grain and is
  consumable by self-review and a future `quicken-watch` bus publisher.

## What this builds

- A `quicken-attest` lib module + `quicken attest` subcommand (extends the
  binary; consumes `quicken-probe`'s `Vec<PrimitiveReport>`).
- **Receipt store**: JSON receipts under `~/.local/share/quicken/receipts/`
  (path injectable for tests), each `{ taken_at, boot_id, reports }`. `boot_id`
  read from `/proc/sys/kernel/random/boot_id` (injectable) so "consecutive
  boots" is well-defined.
- **Delta**: `quicken attest` loads the most recent prior receipt and computes a
  `Delta` per primitive: `Unchanged | Improved | Regressed | EvidenceChanged`
  (e.g. memlog pkgrel 5→11 with verdict unchanged → `EvidenceChanged` with the
  pkgrel pair surfaced).
- **Streak counter**: a derived `inert_streak` per primitive = number of
  consecutive prior receipts (distinct `boot_id`s) where the verdict was worse
  than `LiveDegraded`. The human/JSON output escalates wording by streak band
  (e.g. `1` = "dark", `≥3` = "dark for 3 boots", `≥7` = "DARK FOR 7 BOOTS —
  needs attention"), so silence is impossible by construction.
- **CLI**: `quicken attest` writes a new receipt and prints the delta + streaks;
  `quicken attest --json`; `quicken attest --no-write` (compute + print without
  persisting, for dry inspection).

## Acceptance criteria

1. `quicken attest --help` documents `--json` and `--no-write`.
2. `quicken attest` writes a well-formed receipt to the injected store path and
   the file round-trips back into the receipt type (golden test).
3. Given a seeded prior receipt and a current set of reports, the computed
   `Delta` is correct for each case — `Unchanged`, `Regressed` (Live→Inert),
   `Improved` (Inert→Live), and `EvidenceChanged` (memlog pkgrel 5→11, verdict
   unchanged) — asserted on fixtures.
4. `inert_streak` counts only distinct `boot_id`s: three prior receipts across
   two boot ids with the primitive dark yields the correct consecutive-boot
   streak (tested on a seeded receipt history).
5. The streak-band wording escalates: a fixture with streak `1`, `3`, and `7`
   produces the three distinct severity strings (asserted).
6. `quicken attest --no-write` produces identical stdout to `quicken attest` but
   creates no receipt file (asserted: store dir unchanged).
7. Tests perform **zero network access and write only inside the injected store
   tmpdir** (cloud-build-safe); `boot_id` and clock are injected, never read from
   the real host in tests (consistent with the no-`Date::now` constraint).
