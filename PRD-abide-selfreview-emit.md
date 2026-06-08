# PRD: abide-selfreview-emit — self-review's "acked" prose becomes durable ledger state

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/.claude/skills/self-review
Vision: visions/abide.md

## TL;DR

Self-review already *reports* findings into `docket` (shipped:
`docket-self-review-bind`, scripts under
`~/.claude/skills/self-review/scripts/`). But when it decides a finding is
"acked — carry until X changes," that decision is written only as journal
prose and re-typed every run. This PRD closes the prose↔ledger gap: when
self-review marks a finding acknowledged, it emits `docket ack <key> --reason
"<prose>" --until-change "<fingerprint>"` through the existing bind seam, so
the ledger carries the acknowledgement (and auto-resurfaces it on change) and
the banner goes quiet. The two live acked-in-prose findings —
`memlog-activation` and `warden-enforcer-inert` — are the first real consumers.

## Why this exists

Phase 1 evidence (2026-06-08 — see `visions/abide.md`):

- Self-review's 2026-06-08 reflection (`recall 01KTK2KF17SQYDZ4MZ3PQCK5T1`)
  lists `(4) memlog staged-awaiting-install pkgrel-5/staged-11, acked` and
  `(5) warden bpolicy inert, acked` — and the 2026-06-07/-06 reviews carried
  the same prose. The acknowledgement is real and durable in the human's head;
  it is ephemeral in the tooling.
- `docket list --open` (live) shows both still `[open] warn` runs_seen:3 — they
  surface in every digest/banner because nothing translates the prose "acked"
  into ledger state.
- The carry-condition is already computed each run: memlog's is the **installed
  pkgrel** (self-review reads "installed pkgrel-5"), warden's is the **bpolicy
  `loaded` bool** (self-review reads "loaded:false"). These are exactly the
  fingerprints `abide-ack-state`'s `--until-change` consumes.
- The emit seam exists: `scripts/docket-runid.sh` +
  `scripts/docket-bind-selftest.sh` already wire self-review to the `docket`
  binary, so this is an extension of a shipped producer, not new plumbing.
- Strict dependency: needs `abide-ack-state` (the `ack` subcommand) and
  `abide-digest-quiet` (so the ack actually quiets the banner) landed first.

## What this builds

`shell`, into `~/.claude/skills/self-review/scripts/` (+ a SKILL.md note):

- **An `docket-ack-emit.sh` helper** (sourced or called by the review's
  docket-bind step) that, given a finding key, a reason, and a fingerprint,
  runs `docket ack "$key" --reason "$reason" --until-change "$fp"`. **Fail-open:**
  if `docket` is absent or errors, print the would-be `ack` line and exit 0 —
  never block the review (per `self_build_jq_escape_reads_absent` /
  `self_delegate_run_300s_cap` fail-open discipline).
- **Wire the two live items.** When the review marks `memlog-activation` acked,
  emit with `--until-change "pkgrel:$(installed memlog pkgrel)"`; when it marks
  `warden-enforcer-inert` acked, emit with
  `--until-change "bpolicy:$(loaded bool)"`. The pkgrel / loaded values are
  already gathered by the existing snapshot step — reuse them, do not
  re-derive.
- **Idempotent.** Re-running the review re-acks with the same fingerprint,
  which (per `abide-ack-state`) leaves the existing ack untouched. If the
  fingerprint moved (pkgrel changed, bpolicy armed), the next `docket report`
  for that finding auto-clears the ack and it resurfaces — the review then
  surfaces it under "Pending your call" again, correctly.
- **SKILL.md note** under the docket-bind section documenting that "acked"
  findings now emit a durable `docket ack`, and that resurfacing is
  edge-triggered on the recorded fingerprint.

No Rust. Pure shell + the `docket` CLI. Reuses `docket-runid.sh` for the
current run-id.

## Acceptance criteria

1. **`docket-ack-emit.sh <key> <reason> <fingerprint>` emits a `docket ack`**
   with those args; after running it, `docket show <key> --json` reports
   `acknowledged_at` non-null, matching `ack_reason` and `ack_fingerprint`.
2. **Fail-open when `docket` is absent.** With `docket` not on `PATH`, the
   helper prints the would-be ack line and exits 0 (assert exit 0 + the printed
   line in a test that shadows `PATH`).
3. **The two live findings get acked with the right fingerprints.** A dry-run /
   test invocation of the review's docket-bind step acks `memlog-activation`
   with an `--until-change` value derived from the installed memlog pkgrel and
   `warden-enforcer-inert` with one derived from the bpolicy loaded bool (assert
   the emitted commands carry a non-empty fingerprint for each).
4. **After ack, the two findings drop out of the default digest.** Following the
   emit, `docket digest` (no `--include-acked`) does not count them in
   `open`/`escalated`, and `detail.acked` reflects them (depends on
   `abide-digest-quiet`).
5. **Re-running is idempotent** — a second emit with the unchanged fingerprint
   does not duplicate or reset the ack; the finding stays acked.
6. **Fingerprint change resurfaces.** A test that emits an ack, then `docket
   report`s the finding with a different fingerprint, shows the finding
   `is_acked() == false` and back in the default digest counts.
7. **`bash -n` clean** on the new/changed scripts and the existing
   `docket-bind-selftest.sh` continues to pass.
