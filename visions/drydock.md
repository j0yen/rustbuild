# Vision: drydock — one muster of the fleet's drift, routed to its safe lane

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-16
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection. The strongest *recurring*
signal across a full week of self-reviews is not a missing detector — the
detectors exist and work — it is that **nothing closes the loop and nothing
routes**. Every self-review re-dumps the same flat "Pending your call" wall, the
same items survive for days (wm-stt 9d stale, kernel pkgrel staged-not-installed,
adopt N/N not-current), and the human re-triages the identical list each morning.

## TL;DR

The laptop already has a mature staleness toolchain: `binstale` classifies
running daemons (`fresh | deleted-exe | inode-drift | prov-stale | behind-head`),
`adopt` tracks unadopted/stale non-daemon artifacts, `rollout` does safe
serialized daemon restarts, `scion` made the verdicts truthful (lineage not
clock), `fixpoint` wired the *adopt* loop to converge on a cron. What's missing
is the layer **above** all of them: a single muster that aggregates every
staleness signal into one inventory, **routes each item to a remediation lane by
risk** (auto-safe plain CLI / time-windowed voice daemon / reboot-gated kernel
package / human-approval), emits one ranked digest that self-review embeds
instead of its sprawling wall, and tracks whether the fleet is actually
*converging* run-over-run. drydock is the refit yard: every drifted thing comes
in, gets sorted into the lane that can safely fix it, and the safe lane drains
itself.

## End-state

When drydock is fully built:

- `drydock survey --json` is the single source of truth for "what has drifted":
  daemons (from `binstale fleet`), CLIs (from `adopt verify`), and the
  kernel package (installed pkgrel/feature vs the patched pkg staged in
  `~/wintermute/wintermute-kernel/pkg/`) — one normalized inventory, no caller
  has to stitch three tools together by hand.
- Every item carries a **remediation lane**, assigned deny-by-default:
  `auto` (idempotent, no live-service disruption — `adopt apply` / `rollout
  install` of a plain CLI), `window` (voice daemons wm-audio/dialog/stt/tts —
  needs a quiet window), `reboot` (kernel package), `approval` (anything that
  would touch the immutable self-review guardrail, or any item drydock can't
  positively classify). Unknown ⇒ `approval`, never `auto`.
- `drydock digest` renders the one ranked, lane-grouped, one-command-per-item
  block self-review pastes into "Pending your call" — replacing today's
  unranked prose. It shows a convergence counter per lane and the delta since
  last run.
- A convergence ledger asserts the `auto` lane is non-increasing over time (if
  it grows, either a regression shipped or the safe drain isn't running) and
  ages the long-tail `window`/`reboot`/`approval` items so self-review can
  escalate one that's been stale too long (e.g. wm-stt crossing 7d).
- The `auto` lane drains itself: a narrowly-scoped, dry-run-default executor
  runs *only* the auto-lane commands, never touching window/reboot/approval —
  so it cannot violate the immutable guardrail by construction. The recurring
  "adopt N/N not-current" line finally shrinks without a human.

## Components

One bullet per future PRD (PRD-sized, dependency order):

- **drydock-survey** — KEYSTONE rust-cli. Aggregate `binstale fleet --format
  json` + `adopt verify --format json` + a kernel-package staleness probe into
  one normalized JSON inventory: `{item, kind: daemon|cli|kernel-pkg,
  current_ref, head_ref, verdict, age_days, source_repo}`. Read-only,
  deterministic age via `--now`/`DRYDOCK_NOW`. Deletes/restarts nothing.
- **drydock-classify** — rust-extend INTO drydock-survey. The risk-tier router:
  map each survey item to a lane (`auto`/`window`/`reboot`/`approval`) from a
  TOML lane policy, deny-by-default (unknown ⇒ `approval`). Emit the exact
  one-line remediation command per item. Voice-daemon set and guardrail-gated
  items are hard-coded floors the policy cannot downgrade.
- **drydock-digest** — rust-cli consuming classify JSON. Render the single
  ranked digest self-review embeds: grouped by lane, sorted by `age_days` desc,
  one command per line, per-lane counts + delta-vs-last-run.
- **drydock-ledger** — rust-extend. Append-only convergence ledger (JSONL):
  per-run per-lane counts + item fingerprints; assert `auto` lane
  non-increasing; age long-tail items and flag any past an escalation
  threshold. Feeds digest's delta column.
- **drydock-apply** — gated executor for the `auto` lane ONLY. Dry-run default,
  `--apply` required; refuses any item not in lane `auto`; writes the ledger.
  By construction never touches window/reboot/approval, so it stays inside the
  immutable guardrail.

## Order

survey → classify (extends survey) → digest (consumes classify) →
ledger (feeds digest delta) → apply (consumes classify, auto-lane only,
writes ledger). digest and ledger can land in either order; apply should land
last so its ledger writes match the ledger schema.

## Open questions

- **The auto lane and the immutable guardrail.** `SKILL.md:830` declares
  self-review *"never runs `rollout apply` autonomously … immutable."* drydock-apply
  is scoped to *plain-CLI installs and non-voice `adopt apply`* — it never
  restarts a guarded daemon — so it should sit *outside* the guardrail's intent.
  But the user has the final say on whether even auto-lane self-draining runs
  unattended. Default posture: drydock-apply ships dry-run-default and is NOT
  wired into the self-review cron until jsy opts in. This mirrors the approval
  changeover/vigil already await; drydock does not try to pre-empt it.
- **Kernel-package staleness is a feature question, not a version compare.**
  Verified 2026-06-16: booted `linux-wintermute 7.0.11.arch1-1` is a *higher
  base version* than the staged patched `7.0.10.arch1-12` pkgs that carry the
  agentns prctl fix. A naive `vercmp` would call the booted kernel "newer" and
  miss that it lacks the patch. drydock-survey's kernel probe must compare the
  *agentns-prctl capability* (does `/proc/self/agent_session` move, or does the
  boot hook still print ACTIVATION BLOCKED), not raw version. Until that probe
  is decided, the kernel item is hard-routed to `approval`.
- Should drydock-survey also surface unpushed/dirty repos? No — `tend` owns
  source-tree state; drydock is binary/daemon/kernel freshness only. Keep the
  surfaces disjoint.
- Does the `auto` lane overlap fixpoint's `adopt reconcile→apply`? Partially.
  fixpoint converges the adopt *marker* loop; drydock-apply runs adopt/rollout
  *installs* for items classified auto. They should share the adopt CLI, not
  re-implement it; if fixpoint already drains a given item, drydock-survey will
  simply see it as `fresh` and not list it. No double-action.
