# PRD: drydock-digest — the one ranked block self-review pastes instead of a wall

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/drydock.md
**Repo:** j0yen/drydock-digest (NEVER AtScaleInc)

## TL;DR

Self-review's "Pending your call" section is unranked prose the user re-reads
every morning. drydock-digest consumes drydock-classify's lane-routed inventory
and renders the single block self-review embeds: items grouped by lane, sorted
by `age_days` descending, one remediation command per line, with per-lane counts
and a delta-vs-last-run column. It turns "here is everything that drifted, in no
order, described in sentences" into "here are 3 auto / 1 window (wm-stt, 9d) /
1 reboot / 2 approval, oldest first, one command each."

## Why this exists

Verified 2026-06-16:

- The 2026-06-16 self-review journal's "Pending your call" is eleven bullets of
  prose mixing a one-command `adopt apply`, a windowed wm-stt restart, a kernel
  `pacman -U`, "45 dirty repos," and "5 unpushed repos" — different risk tiers,
  different owners, no ranking. The user triages this by hand daily.
- The same high-age items recur untouched: wm-stt has been "9d stale" while
  earlier reviews show it climbing (the long tail never gets surfaced as
  *urgent*, just repeated).
- drydock-classify produces lane + command + age per item; nothing yet renders
  that as the compact, ranked, self-review-ready artifact. Without the digest,
  classification's value never reaches the human.

## What this builds

A Rust CLI `drydock-digest` (clap) consuming classify JSON on stdin or by
invoking classify internally.

**Rendering**
- Group items by lane in fixed order: `auto`, `window`, `reboot`, `approval`
  (drainable-first, so the human sees what's already handled vs what needs them).
- Within a lane, sort by `age_days` descending; show `item`, `age_days`,
  `verdict`, and the remediation `command` (or reason for `approval`).
- A header line per lane: `auto: 3 (Δ-1)  window: 1  reboot: 1  approval: 2`
  where Δ is the change since the last ledger entry (drydock-ledger); absent the
  ledger, omit Δ gracefully.
- `--format md` emits the markdown block self-review pastes verbatim (a fenced
  table or list); `--format json` re-emits for further composition; default is a
  terminal table.
- **Escalation marker:** any item whose `age_days` exceeds a threshold
  (`--escalate-days`, default 7) is flagged (e.g. a `!` prefix) so a long-tail
  item like wm-stt 9d reads as urgent, not routine.

**Deps:** `clap`, `serde`/`serde_json`, `anyhow`, a small table writer
(`comfy-table` or hand-rolled). No network. Reads the optional ledger path for Δ
but never writes it (that's drydock-ledger/apply).

**UX**
```
drydock-classify --json | drydock-digest --format md     # paste into self-review
drydock-digest --escalate-days 5
drydock-digest                                            # terminal table
```

## Acceptance criteria

1. `drydock-digest` consumes classify JSON (stdin or internal call) and renders
   items grouped by lane in the order auto, window, reboot, approval.
2. Within each lane items are sorted by `age_days` descending; each line shows
   `item`, `age_days`, `verdict`, and `command` (reason for `approval`).
3. The per-lane header reports counts matching the rendered items.
4. `--format md` emits a markdown block (verified to render as a table/list);
   `--format json` round-trips the input fields; default is a terminal table.
5. Items with `age_days > --escalate-days` (default 7) are visibly flagged; a
   fixture with a 9-day item shows the flag, an 8-day item does not at
   `--escalate-days 8`.
6. When a ledger is present, the header shows a Δ per lane vs the last run; when
   absent, Δ is omitted without error.
7. Empty input renders a clean "fleet fresh — nothing drifted" message, exit 0.
8. `--help` documents every flag; `cargo test` green; `cargo clippy` clean on the
   crate's own code.
