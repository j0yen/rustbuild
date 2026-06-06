# Vision: warrant — a close that names a different mechanism must prove it

**Authored by:** /dream (Claude Opus 4.8), with jsy
**Created:** 2026-06-06
**Status:** active
**Seed:** bare `/dream` + Phase-1 live inspection. Like `assay` yesterday,
the strongest *unaddressed* signal this pass was not a missing feature —
it was a **closed PRD whose closing claim is mechanically false**, caught
only by exercising the live pipeline instead of trusting the close note.

## TL;DR

When a PRD is closed with *"outcome achieved live by a different
mechanism"*, that sentence retires a finding. Nothing ever checks that the
named mechanism actually delivers the outcome. So a confidently-worded,
plausible-but-false close note can sit for days hiding a still-open gap —
until something rediscovers the symptom from scratch and re-drafts a whole
fleet, never noticing the prior close was a fiction.

This pass proved it live. `PRD-ctrace-session-end-resilient.md` was
**Closed 2026-06-02** with the note:

> "`ctrace-reap.timer` runs `ctrace-orphan-reap --apply` every 2 min; its
> `render_log` step summarizes any orphaned `*.ndjson` — the practical
> equivalent of AC1's backfill."

That is false. Verified on the booted system this pass:

- `ctrace-orphan-reap --apply` renders a log **only when it finds a *live
  orphaned tracer*** to stop (`verdict: orphaned-tracer`). The reaper's own
  service comment says: *"session.bt now self-terminates on root exit, so
  this should normally find nothing."*
- So in the **common** SIGKILL case (headless `/build`/`/dream`/
  `/self-review` tick), `session.bt` self-terminates cleanly → there is **no
  orphaned tracer** → the reaper returns `healthy`/`no-tracer` and renders
  **nothing** → and the SessionEnd hook never fired either (SIGKILL). The
  log is summarized by **nothing but self-review's hand-run `scribe
  backfill`.**
- The two shipped fixes are *mutually defeating*: making the tracer
  self-terminate is exactly what removes the orphan the reaper needed to
  trigger its render. The close note never saw this because it never ran
  the path.

The gap silently reopened — **623 of 1874 logs (33%) un-summarized by
2026-06-05** — which is why the **`coda`** vision had to be drafted yesterday
*from scratch*, re-counting the backlog, never aware that the same gap had
been "closed" three days earlier on a mechanism that cannot fire.

This is the **second consecutive `/dream` pass** to catch a false-mechanism
close by verifying live:

1. **2026-06-06 (assay):** `onramp`'s shipped `claude-agentns-wrap` is futile
   — it calls `unshare(CLONE_NEWAGENT)` which the kernel rejects (EINVAL,
   `0x100 == CLONE_VM`). A *remedy* shipped against a *cause never exercised*.
2. **2026-06-06 (this pass):** `session-end-resilient` closed on a reaper
   render path that is dead code for the common case. A *finding retired*
   against a *mechanism never exercised*.

(A third, softer corroboration: `signet`'s TL;DR still asserts agentns
all-zeros means "unwrapped, fine" — a mechanism claim `assay` has since
falsified. The pattern is systemic, not a one-off.)

`assay` builds the missing layer between *read the surface* and *ship the
remedy*, for **kernel/process primitives** it can fork-and-exercise.
`warrant` builds the parallel layer for **closes** — including the
systemd/shell/timer/pipeline closes `assay` structurally can't fork: a
**closing claim is not discharged until a reproducible assertion proves the
named mechanism delivers the named outcome**, and that assertion is
re-checked so a regression re-opens the finding instead of rotting.

## End-state

When this is done:

- Every archived PRD closed with *"achieved by a different mechanism"* /
  *"superseded by"* carries a **warrant**: a one-line, side-effect-free,
  re-runnable assertion that the substitute mechanism produces the closed
  outcome. A close without a warrant is **`Unwarranted`** — a visible
  backlog item, not an invisible assumption.
- `warrant audit` answers, in one command, *"which of our closes are still
  true?"* — `Proven` / `Refuted` / `Unproven` / `Unwarranted` — instead of a
  human re-deriving it by re-running each pipeline by hand.
- A `Refuted` close (the reaper render path; a futile wrap) surfaces as a
  **tracked docket signal under a stable slug**, edge-triggered, the moment
  the assertion fails — so the next `/dream` or `/build` sees *"this close is
  a lie"* before re-drafting a fleet to rediscover it.
- The `session-end-resilient` close is the first warrant written: its
  assertion greps the live SessionEnd hook + reaper verdict and reports
  `Refuted`, pointing at `coda` as the real superseding fix.
- The pattern generalizes: any close that retires a finding "by a different
  mechanism" gets a warrant before it is allowed to discharge the finding,
  so a from-scratch rediscovery (the `coda` fleet, the `assay` fleet) is
  never again the detector of a false close.

## Why this is real (Phase 1 evidence, 2026-06-06 ~08:30 UTC)

- **The false close, verified live.** `ctrace-orphan-reap --help` +
  `~/.config/systemd/user/ctrace-reap.service` + the live
  `journalctl -u ctrace-reap.service` run (`verdict: healthy` →
  `apply: state is healthy; nothing to do`) prove the reaper renders only
  on `orphaned-tracer`, which `session.bt` self-termination now prevents.
  `~/.claude/scripts/ctrace-session-end.sh` still calls the slow shell
  `summarize-ctrace-session.sh`, never `scribe`, and never fires on SIGKILL.
- **The cost, measured.** The gap reopened to 623/1874 (33%) by 2026-06-05;
  the entire `coda` vision + 4 PRDs (`PRD-coda-{sweep,audit,close,boot}.md`,
  in-flight) exist *only because* the false close hid the still-open gap.
- **The recurrence, dated.** Two false-mechanism closes caught by live
  verification on consecutive `/dream` passes (assay/onramp 2026-06-06;
  this 2026-06-02 close re-verified 2026-06-06). `feedback_verify_before_concluding`
  is the standing memory; `assay`'s vision already named *"any future
  'shipped fix but symptom persists' primitive"* — but scoped itself to
  forkable primitives, leaving pipeline-shaped closes uncovered.
- **The shape is a known-good family.** `docket` (findings get a memory),
  `coda` (summary debt self-heals), `quicken` (primitive liveness),
  `assay` (primitive mechanism). `warrant` is the next sibling: **close-claim
  attestation** — same inward-toolkit ethos (pure core, read-mostly,
  proposal-first, fail-open, publishes as a `j0yen` repo).

## Components (one bullet per future PRD — drafted set first)

- **warrant-corpus** — new repo `~/wintermute/warrant`; the corpus. Shared
  types (`CloseClaim` / `ClaimKind` / `Warrant` / `WarrantStatus` /
  `WarrantVerdict` / `AuditPlan`), a `CloseSource` trait abstracting where
  close notes live (archived PRDs + docket entries), a `FakeSource` for
  tests, and a **pure** `classify(notes) -> Vec<CloseClaim>` that parses the
  close-note grammar (`Status: Closed … "by a different mechanism" /
  "superseded by" / deferred_acs`) with **zero IO**. `warrant list` prints
  it. (Mirrors `coda-sweep` / `anchor-roots`.) *Root.*
- **warrant-audit** — rust-extend; the live read + assertion runner.
  Implements the real `CloseSource` over `~/wintermute/autobuilder/PRDs-archive/`
  (+ docket if present, fail-open), loads a declarative `warrants.toml`
  registry (a close source → a side-effect-free assertion:
  `command-exit` / `file-exists` / `grep-absent` / `counter-nonzero`), runs
  each, and emits `WarrantVerdict` (human table + `--format json`). A close
  with no registry entry → `Unwarranted`. Read-only beyond the contractually
  side-effect-free assertions; per-assertion failure counted, never fatal.
  (Mirrors `coda-audit` / `anchor-probe`.) Depends on corpus.
- **warrant-docket** — rust-extend; the edge-triggered bridge. `Refuted`
  (and aged `Unwarranted`) verdicts are reported to `docket` under a stable
  slug (`warrant:<source>`), re-opening the finding so a false close is a
  tracked signal, not silent rot. Fail-open if `docket` is absent (print +
  exit 0). The first warrant shipped is the `session-end-resilient` one,
  which reports `Refuted` and cross-references `coda`. (Mirrors
  `keel-beacon` edge-trigger.) Depends on audit.

## Order

```
warrant-corpus              (new repo; pure classify + types + FakeSource)
   └──► warrant-audit       (real CloseSource + warrants.toml runner)
            └──► warrant-docket   (edge-triggered re-open of Refuted closes)
```

Strict chain, mirrors `coda`/`anchor`: **do not** start either rust-extend
until `warrant-corpus` has shipped and the repo exists, or extend-validate
fails (the rule that bit relay/concord/quicken/keel/anchor).

## Open questions (HELD — not yet drafted; Fleet 2 / user)

- **warrant-gate (prevention, not detection).** A `/build` close-time hook
  that *refuses* to write a "by a different mechanism" close note unless a
  warrant is registered in `warrants.toml`. This is the load-bearing half —
  audit catches false closes *after*, gate stops them *before* — but it
  touches the `/build` skill close path and the classifier, which I have not
  yet traced. Draft after warrant-audit ships and the close-note write site
  is located.
- **assay-bridge.** When a close's mechanism *is* a kernel/process primitive
  (the agentns-wrap case), the warrant should delegate to `assay <name>`
  rather than re-implement a local assertion — unifying the creation-side
  (`assay`) and close-side (`warrant`) attestations under one verdict. Draft
  once both `assay agentns` and `warrant-audit` are live so the JSON contract
  is real, not guessed.
- **warrant-witness.** Emit `wm.warrant.*` to the bus edge-triggered (like
  `keel-beacon`) so a newly-`Refuted` close pings without waiting for the
  next audit run. Depends on the shared `wm.health.*` envelope question still
  open in `quicken`/gossip.
- **Registry authorship.** Should `warrants.toml` be hand-written at close
  time, or should `warrant audit` propose a stub assertion per `Unwarranted`
  close for a human to fill? v1 is hand-written + `Unwarranted` backlog;
  auto-stub is a Fleet 2 nicety.

## Relationship to neighbors

- **assay** exercises a *primitive's creation mechanism* (forks a child).
  **warrant** attests a *close's claimed mechanism* (runs a declared
  assertion) — including pipeline/timer/hook closes assay can't fork. Where
  the two overlap (a primitive-shaped close), warrant delegates to assay
  (Fleet-2 `assay-bridge`). Disjoint, composable.
- **quicken** reads *is the primitive live now*; **warrant** reads *was the
  close that retired this finding ever true*. Different subjects (live
  primitive vs retired finding), same read-mostly ethos.
- **docket** is the *ledger*; warrant is a *producer* into it (re-opening
  false closes), exactly as self-review is. Keep the slug namespace distinct
  (`warrant:<source>`).
- **coda** is the *correct* superseding fix for the specific summary-debt
  gap; warrant does **not** re-fix it — warrant's first warrant simply
  *records that the old close was false and points at coda*.
