# PRD: fixpoint-cron-reconcile — run the cure on every cron tick

Status: Draft v0.1
build_target: config
Vision: visions/fixpoint.md

## TL;DR

`adopt-cron.service` reinstalls stale binaries every 6h but never mints
the lineage markers that would actually clear them, so the
`clock-fallback` false-stale set never drains. This PRD wires
`adopt reconcile --execute` into the cron, immediately before
`adopt apply --execute`, so the cure runs autonomously and the not-current
count converges instead of treading water.

## Why this exists

Phase-1 live inspection, 2026-06-13:

- `systemctl --user cat adopt-cron.service` ExecStart runs exactly two
  adopt commands: `adopt apply --execute --with-daemons` then
  `adopt report --run "$RUN_ID"`. **`reconcile` is absent.**
- `adopt scan --format json | jq 'group_by(.freshness_basis)'` →
  `clock-fallback: 16` — every not-current artifact is on the fallback
  basis, i.e. has **no lineage marker**. scion shipped `reconcile` (mints
  markers, no rebuild) on 2026-06-13, but nothing runs it.
- `adopt reconcile --dry-run` shows it *would* seed markers for dozens of
  clean repos (`[dry-run] would seed adopt fp=e601fcb3…`, `anchor`,
  `answerable`, `assay`, …). The cure is ready; it is simply never invoked.
- Consequence: `apply` updates `installed_ts` on reinstall, but with no
  marker the verdict stays `clock-fallback` and flaps back to
  `installed-stale` on the next source commit. Self-review has parked
  `adopt-scan-stale-binaries` / `fleet-binary-staleness` run after run
  (recall reflective seeds 2026-06-11/12/13).

## What this builds

A one-line edit to `~/.config/systemd/user/adopt-cron.service` ExecStart
so the pipeline reconciles before it applies:

```
ExecStart=/bin/bash -c '\
  RUN_ID=$(date +%%Y-%%m-%%d.%%H) ; \
  /home/jsy/.local/bin/adopt reconcile --execute 2>&1 | tee /tmp/adopt-cron.log ; \
  /home/jsy/.local/bin/adopt apply --execute --with-daemons 2>&1 | tee -a /tmp/adopt-cron.log ; \
  /home/jsy/.local/bin/adopt report --run "$RUN_ID" 2>&1 | tee -a /tmp/adopt-cron.log ; \
  echo "[adopt-cron] done $RUN_ID" \
'
```

`reconcile` runs first so markers exist before `report` records state.
(Open question in the vision: whether to instead run reconcile *after*
apply so a freshly-reinstalled binary is marked in the same tick. Default
here is reconcile-first to drain the existing fallback set; revisit once
verify-resolution lands.) No new binary, no network, no rebuild — pure
unit edit + `systemctl --user daemon-reload`.

## Acceptance criteria

1. `adopt-cron.service` ExecStart invokes
   `/home/jsy/.local/bin/adopt reconcile --execute` before
   `adopt apply --execute`.
2. `systemctl --user daemon-reload` succeeds and
   `systemctl --user cat adopt-cron.service` shows the reconcile line.
3. A manual `systemctl --user start adopt-cron.service` completes without
   error (`systemctl --user show -p Result adopt-cron.service` →
   `Result=success`), and `/tmp/adopt-cron.log` contains reconcile output
   (a `seeded`/`would seed`-class line or `nothing to reconcile`).
4. After that run, `adopt scan --format json | jq '[.[]|select(.freshness_basis=="lineage")]|length'`
   is strictly greater than 0 (at least one binary now verified by
   lineage rather than clock) — proving the wire changed live state.
5. The edit is idempotent: re-applying the change is a no-op; the existing
   `apply` and `report` invocations are preserved unchanged.
