# Vision: mend — a finding that escalates and never gets fixed is the docket failing at its one job

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-08
**Status:** active
**Seed:** bare `/dream` (interactive). Both the inward and outward arcs are
saturated (recourse closed the outward return path at 07:00 today). The
strongest *unaddressed* signal this pass was not a new direction — it was the
self-review's own chronic escalations, named and counted by `docket` but owned
by no vision and fixed by nothing.

---

## TL;DR

The laptop already has two halves of a maintenance loop:

- **`docket`** gives each recurring self-review finding an identity, a streak,
  and an escalation threshold (3+ runs → "do something"). It *names* the
  problem and counts how long it has survived.
- **`assay`** verifies that a shipped fix actually outlived its symptom, so a
  finding doesn't get marked resolved while still firing.

The middle is missing. Between "docket has counted this finding across N runs"
and "assay confirms the fix held" there is **no remediation step** — nothing
converts a long-escalated, owner-less docket entry into a shipped fix. So the
docket fills with findings that have survived 5, 8, even 21 runs (measured live
this pass) and just… sit there, re-counted every night.

`mend` is that middle. It does two things:

1. **Closes the specific long-lived findings by hand** (this dream's PRDs):
   the three chronic, owner-less entries in the live docket each get a real,
   buildable fix.
2. **Ships the bridge so it never has to be done by hand again**: a tool that
   reads `docket list --escalated --format json`, finds entries past the
   escalation threshold with no linked PRD, and drafts a PRD stub + gossip note
   for /dream or /build to pick up. The docket stops being a parker and becomes
   a feeder.

The narrative is deliberate: mend does the first cut by hand to prove the
remediation pattern, then automates the pattern it just demonstrated.

## Live evidence (Phase 1, 2026-06-08)

`docket list` open findings, this box, this pass:

```
[open] agentns-session-zeros (warn)  runs_seen: 8  report_count: 10   ← claimed by vision-assay
[open] ctrace-sessionend-flake (warn) runs_seen: 5  report_count: 5    ← OWNER-LESS
[open] pacman-kernel-update-blocked   runs_seen: 1                     ← user-gated reboot, not buildable
[open] warden-enforcer-inert (warn)   first_run: 2026-06-03           ← OWNER-LESS
```

Plus a finding the self-review skips *without even docketing it*:

- **binstale not installed.** `~/wintermute/binstale/` is a complete, mature
  repo (5 src modules, verdict taxonomy README, git history through 2026-06-02)
  — but no binary is installed to `~/.local/bin/`, so `which binstale` fails and
  every self-review prints "fleet staleness check skipped." A *built* tool the
  fleet never wired in. Double gap: skipped *and* never reported to docket.

## End-state

When this vision is done:

1. The three owner-less chronic findings are resolved at the source, not
   backfilled: ctrace summaries render on session exit; binstale runs as part of
   the self-review and reports to docket; warden's inert-enforcer state is
   diagnosable with one command.
2. `docket sweep` can auto-close them because the symptom actually stops firing
   (the assay discipline: fix must outlive symptom).
3. A new escalation-crossed finding with no PRD gets a drafted PRD stub
   automatically — the docket→fix loop is closed without a human eyeballing
   `docket list` every morning.

## Components (PRD-sized)

- **mend-binstale-wire** — install binstale to `~/.local/bin`, add a
  self-review-friendly `binstale fleet --format json` aggregate over running
  daemons, and emit a `docket report` when any non-`fresh` verdict fires. Closes
  the "fleet staleness check skipped" recurring line. (rust-extend → binstale)
- **mend-ctrace-render** — fix the SessionEnd render-on-exit path so ctrace
  summaries write at session close; demote backfill from primary to true
  fallback; report to docket only when render *and* backfill both miss. Closes
  `ctrace-sessionend-flake` (5 runs). (mixed → ctrace-scribe)
- **mend-warden-doctor** — a read-only `bpolicy doctor` that diagnoses *why*
  the eBPF-LSM enforcer reads `loaded:false` (kernel `lsm=` cmdline list, BTF
  presence, attach errno, group membership) and prints a structured activation
  gap. Diagnostic only — never auto-arms. Advances `warden-enforcer-inert`.
  (rust-extend → bpolicy)
- **mend-bridge** — the keystone. Reads escalated, PRD-less docket entries and
  drafts a `PRD-<key>.md` stub + gossip note. Proposal only: never auto-builds,
  never resolves the docket entry, never edits an existing PRD. (rust-cli → mend)

## Order

```
mend-binstale-wire ─┐
mend-ctrace-render ─┼─ (independent leaf fixes, ship in any order)
mend-warden-doctor ─┘
                     mend-bridge  (independent of the leaves; needs only docket,
                                   which is already built)
```

The three leaves are the manual first cut. mend-bridge is what makes the next
cut automatic — it can ship first or last, it only depends on `docket` (built).

## Open questions

- **Finding↔PRD linkage.** mend-bridge needs to know which docket findings
  already have a PRD so it doesn't re-draft. Proposal: a `prd:` field on the
  docket finding (set by the bridge when it drafts, or by hand), or a naming
  convention (`PRD-mend-<docket-key>.md`). Which is more durable across
  docket-key renames? (Leaning naming convention + a `docket report --prd`
  annotation.)
- **warden-doctor scope.** Should it ever *offer* to arm the enforcer (gated,
  `--arm`), or stay strictly read-only? The fleet's settled stance (recourse,
  tribunal) is diagnose-don't-mutate; default to read-only and leave arming to
  a separate user-gated PRD if the diagnosis points somewhere safe.
- **binstale fleet scope.** Run over *all* running daemons, or just the
  wintermute voice/bus fleet + `~/.local/bin` tools? Cold-load cost of scanning
  every `/proc/PID/exe` vs a curated allow-list. (Leaning curated fleet list +
  `--all` opt-in.)
- **bridge cadence.** Should the bridge run inside /self-review (report → maybe
  draft, same tick) or as its own timer? Same-tick risks drafting on a transient
  spike; leaning: bridge only acts on `--escalated` (already past threshold), so
  same-tick is safe.
