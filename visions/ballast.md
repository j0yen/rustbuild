# Vision: ballast — jettison the build weight the laptop no longer carries

**Author:** /dream (Claude Sonnet 4.6), for jsy
**Created:** 2026-06-16
**Status:** active
**Seed:** self-review journal (2026-06-16: "Disk jumped 92% in one day — major
  growth event. Build targets are the likely culprit … Priority: `du -sh
  ~/wintermute/*/target` sweep.") + verified `df` 94% / 28G free.

## TL;DR

The laptop is 94% full (28G free of 468G), and the cause is not data — it is
**205G of Rust build artifacts** spread across **174 `target/` directories**
under `~/wintermute/`. Most of that weight is *fossilized*: heavy compiles now
route to Hetzner via `/cloudbuild` (`AUTOBUILDER_CLOUD=1`), so the live binaries
on `$PATH` came from the cloud box, while the local `target/` dirs that produced
their ancestors sit untouched for weeks. `wintermute-brain/target` is **13G last
written 2026-05-29**, yet `~/.local/bin/wmd` was installed **2026-06-15** — the
local target is dead weight the running system does not depend on. Today the only
remedy is `/self-review` printing a `du` suggestion for a human to act on by
hand. ballast builds the missing autonomous layer: **measure honestly →
classify what is safe to drop → reap it behind a dry-run gate with an
append-only ledger → watch the disk SLO so 94% never surprises us again.**
ballast never deletes a `target/` whose binary isn't reproducibly installed, and
never reaps a build in flight. It reclaims fossil weight, not working state.

## End-state

When this vision is fulfilled:

1. **One command inventories all reclaimable weight.** `target/` dirs, cargo
   registry/git caches, stray `node_modules`/`.venv`, and big `~/.cache`
   subtrees are walked, sized, and classified by age, last-build mtime, whether
   the crate's binary is installed-and-current, and whether the crate builds in
   the cloud. Output is structured JSON — measurement before deletion.
2. **Cloud-built crates are recognized as carrying no local debt.** A crate
   whose compiles route to Hetzner and whose installed binary is *newer* than
   its local `target/` mtime is flagged "fossil — local target is pure waste";
   these are the safest, biggest wins (brain's 13G, audio's 11G).
3. **Reclamation is gated, logged, and reversible-in-spirit.** Reaping is
   dry-run by default; `--apply` is required to delete; every reaped path is
   recorded to an append-only ledger with reclaimed bytes, the classification
   that justified it, and a timestamp. A `target/` is never reaped while a
   `cargo`/build process holds it.
4. **The disk defends a high/low-water SLO autonomously.** When usage crosses a
   high-water mark, ballast reaps the safest candidates until it drops below the
   low-water mark, emitting a structured alert event (no transport hardcoded) so
   a notifier can compose it. The 94% surprise becomes a self-correcting dip.
5. **The loop is auditable.** The ledger answers "how much have we reclaimed,
   what keeps re-growing, what was the biggest recurring waste" so `/self-review`
   reports a number instead of a hunch.

## Components (PRD-sized)

- **ballast-survey** — KEYSTONE. Read-only inventory + classifier; emits JSON.
- **ballast-cloudaware** — extends survey with the cloud-built / fossil-target
  classification dimension (cross-refs cloudbuild routing + installed-binary
  mtime vs target mtime).
- **ballast-reap** — safe gated reclaimer; consumes survey JSON, dry-run
  default, `--apply` gate, append-only ledger, in-flight-build guard.
- **ballast-guard** — disk-SLO high/low-water watcher; triggers survey+reap on
  the safest candidates; structured alert events; exit-code SLO contract.

## Order

ballast-survey → ballast-cloudaware (extends survey) → ballast-reap (consumes
survey JSON) → ballast-guard (orchestrates survey+reap behind the SLO).

survey first: it is the honest measurement everything else trusts. cloudaware
extends survey's classifier before reap exists so reap inherits the safest
signal. reap is the keystone deletion path. guard composes the two under an SLO.

## Open questions

- Should the reclamation ledger sink into an existing audit surface (e.g.
  mqo-decision-log) or stay a free-standing append-only file? Default:
  free-standing JSONL the survey/report can read, no coupling.
- Should ballast manage `~/.cache/sccache` (shared with cloudbuild) at all, or
  treat it as off-limits because the cloud box reuses it? Default: off-limits —
  classify-and-report only, never reap, until proven safe.
- Where is the high-water default? Start advisory (report only) at 85%, reap at
  90%, target 80% — but these belong in a config file, not hard-coded.

## Extension — 2026-06-16: closing the loop (the fleet shipped but doesn't run)

All four original PRDs shipped (survey/cloudaware/reap/guard binaries are on
`$PATH`). But the autonomous SLO loop the end-state promises is **inert in
practice**, and the disk kept climbing — **86% → 92% → 96% across
2026-06-14/15/16** — while the toolkit sat idle. Three verified gaps:

1. **The guard is broken by version skew.** `ballast-guard run` calls
   `ballast-survey --json --candidates` (`ballast-guard/src/guard.rs:141`), but
   survey v0.3.0 dropped `--candidates`; every pass aborts with "unexpected
   argument '--candidates'". End-state #4 cannot happen because the guard can't
   take step one. The guard already parses survey's `--json` schema
   (`guard.rs:162-193`) — it just passes a dead flag.
2. **Nothing winds the guard up.** No `claude-ballast.timer`, no
   `~/.config/ballast/guard.toml` (both verified absent). The watcher has no
   cadence and no policy file; end-state #4 ("defends an SLO autonomously")
   is unreachable until something schedules it.
3. **We measure stock, never flow.** Survey says what's big *now*; nothing
   records the derivative. End-state #5 explicitly wants "what keeps
   re-growing," but self-review can only *guess* ("build targets are the likely
   culprit") because no time-series exists.

**Extension components (PRD-sized):**

- **ballast-contract-repair** — KEYSTONE for this extension. rust-extend INTO
  ballast-guard: drop the removed `--candidates` flag, derive candidates from
  survey v0.3.0's stable JSON, add a contract test so the next schema bump fails
  red instead of bricking the guard. Nothing else runs until this lands.
- **ballast-pilot** — shell/config: default `guard.toml` (water marks from the
  open question above; `mode = "report"` so a fresh install observes before it
  ever deletes), `ballast-guard.service` + `.timer` with a JSONL event-sink,
  idempotent install/uninstall. Closes end-state #4. Reaping stays jsy-opt-in
  via `mode = "enforce"`.
- **ballast-trend** — rust-cli: snapshot successive survey runs into a bounded
  ring, diff for per-path bytes/day growth, rank fastest-growing, project
  ETA-to-high-water. Answers end-state #5's "what keeps re-growing."
- **ballast-digest** — rust-cli: fuse latest survey + trend report + guard event
  log into one ranked block self-review pastes instead of its `du` suggestion
  (the disk-side counterpart to the shipped drydock-digest).

**Extension order:** contract-repair → pilot (needs a working guard) →
trend (independent of pilot; can build in parallel) → digest (consumes trend
+ guard events from pilot).

**Updated open question:** the 210G of `target/` is dominated by *active* repos
(recall 13G, brain 13G fossil, audio 11G). cloudaware/reap handle the fossils;
warm-but-huge active targets remain by design untouched (reaping them forces a
cold rebuild on a CPU-only box). Should ballast ever offer a `cargo clean`-style
reclaim for active targets untouched > N days, or is that permanently jsy's
manual call? Left as an open question — not drafted, since the vision scopes to
fossils and never reaps working state.
