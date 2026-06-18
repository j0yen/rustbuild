# PRD: trim-cron — automate relief and tell self-review the truth

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/trim
Vision: visions/trim.md

## TL;DR

trim-survey/attribute/policy/relief/psi give the box a memory-pressure
brain, but only if something runs them. trim-cron wires a 6-hourly
systemd-user timer (mirroring `adopt-cron` / `consign-cron`) that runs
`trim survey` + `trim relief --no-dry-run` within policy, and replaces
self-review's "swap … not actionable today" line with trim's attributed
one-liner via a guarded, additive skill-doc block. WRAP, don't replace,
self-review reporting; degrade to an honest one-liner + exit 0 when
trim is absent — never break a self-review run.

## Why this exists

Live evidence:

- The "swap 5.4G/8G — not actionable" finding is *recurring* (journals
  2026-06-12..18); a one-shot manual `trim relief` would fix it once
  and let it drift back. The disk fleet already learned this:
  `ballast-pilot` exists precisely to put `ballast-guard` on a timer.
  trim needs the same automation tier.
- `adopt-cron` (6h timer) and `consign-cron` (gossip 2026-06-18) are
  the established precedent for "reclaim tool on a systemd-user timer,
  dry-run-safe defaults, receipts." Mirror their unit shape rather
  than inventing one.
- Self-review's swap line is the natural consumer: it already reports
  disk via `ballast`'s digest block; the memory line should likewise
  call trim instead of printing a hardcoded "not actionable."

## What this builds

- `~/wintermute/trim/contrib/trim-relief.timer` +
  `trim-relief.service` (systemd-user): `OnCalendar` every 6h (mirror
  adopt-cron cadence), `ExecStart` = `trim relief --no-dry-run`
  (policy still gates every action), `Type=oneshot`. Installed to
  `~/.config/systemd/user/` by an idempotent installer script
  (`contrib/install.sh`), enabled with `systemctl --user enable --now`.
- A guarded, ADDITIVE self-review skill-doc block (via the
  update-config / skill-doc-edit pattern colophon-digest used): in the
  laptop's memory/swap reporting step, call `trim survey --format json`
  and emit an attributed one-liner ("swap 5.4G used; top: homeward-embed
  476M [relief-eligible], recalld 237M; PSI some60=0.0%"). If the
  `trim` binary is absent, emit the honest legacy one-liner and exit 0.
- A `trim psi --watch` unit is OPTIONAL and left as a follow-on (the
  watcher is useful but not required for the recurring-finding fix);
  document it in the README, don't enable it here.

## Acceptance criteria

1. `contrib/install.sh` is idempotent: running it twice leaves exactly
   one `trim-relief.timer` + `trim-relief.service` in
   `~/.config/systemd/user/`, enabled, with no duplicate drop-ins.
2. The timer fires `trim relief --no-dry-run` on a 6-hourly
   `OnCalendar` (matching the adopt-cron cadence family); `systemctl
   --user list-timers` shows it scheduled after install.
3. `trim relief --no-dry-run` invoked by the unit acts ONLY within
   trim-policy (a non-eligible system never gets touched); the service
   exits 0 on a no-op run (nothing eligible) and logs a receipt when it
   acts.
4. The self-review skill-doc block is ADDITIVE — it wraps, does not
   replace, the existing memory/swap reporting; the diff touches only
   the swap-line region and adds no removal of existing hooks/steps.
5. When the `trim` binary is absent from `$PATH`, the self-review block
   degrades to the honest legacy one-liner and exits 0 — a self-review
   run never fails because trim is missing (test by running the block
   with `trim` unresolvable).
6. The timer is disable-clean: `systemctl --user disable --now
   trim-relief.timer` removes it from `list-timers` and the installer
   documents re-enable; no residual transient `MemoryHigh` properties
   are left applied (relief receipts remain for audit).
7. Installer + skill-doc edit are exercised in a dry sandbox (`sbx`)
   and the self-review degrade path is asserted. (No cargo here —
   shell target; the rust binaries it invokes were built via
   /cloudbuild in their own PRDs.)
