# PRD: mend-bridge — an escalated finding with no PRD should draft its own PRD

Status: Draft v0.1
build_target: rust-cli
build_into: /home/jsy/wintermute/mend
Vision: visions/mend.md

## TL;DR

`docket` names a chronic finding and counts how many runs it has survived;
crossing the escalation threshold (3+ consecutive runs) is supposed to mean "do
something." But "do something" is still a human reading `docket list` every
morning and deciding to write a PRD — which is exactly why findings sit open for
5, 8, 21 runs (measured live). `mend-bridge` closes that loop: it reads
escalated, PRD-less docket entries and drafts a `PRD-mend-<key>.md` stub plus a
gossip note, so /dream or /build can pick it up. It is a **proposal** generator
— it never auto-builds, never resolves the finding, never edits an existing PRD.
This is the keystone that turns the three hand-built mend fixes into a repeatable
pattern.

## Why this exists

- **The threshold is codified but eyeballed.** docket vision: *"The threshold is
  codified but eyeballed … The mechanism for 'recurs across 3+ runs' is a
  human/agent reading `recall query` results."* Live proof:
  `agentns-session-zeros` has runs_seen: 8 / report_count: 10 and still sits
  open; the docket digest itself says `oldest: agentns-session-zeros (8 runs)`.
- **Findings stay open for ~20+ runs with no action.** docket vision cites
  "agentns agent_session all-zeros has been Pending for ~21 consecutive runs."
  Counting the survival did not produce a fix; nothing consumes the escalation.
- **The two manual halves now exist.** This same dream drafts three by-hand
  fixes (ctrace-render, binstale-wire, warden-doctor) for the current owner-less
  findings. mend-bridge is the tool that would have drafted those stubs itself —
  building it stops the next three from needing a human.
- **docket is built and exposes the interface.** Verified live: `docket list
  --escalated --format json`, `docket show <key>`, `docket report`, record
  fields `key / severity / title / first_seen / last_seen / runs_seen /
  consecutive_runs / report_count / evidence`. The bridge has a real,
  inspected API to read — no invented contract.

## What this builds

A new rust-cli at `~/wintermute/mend`:

1. **`mend bridge`** — reads `docket list --escalated --format json` (and open
   findings whose `runs_seen` / `consecutive_runs` cross a configurable
   threshold, default 3). For each finding **without an existing PRD**, drafts:
   - `~/wintermute/autobuilder/PRD-mend-<key>.md` — a stub carrying
     `Status: Draft v0.1`, `Vision: visions/mend.md`, a TL;DR templated from the
     finding's `title`, a "Why this exists" section that quotes the finding's
     `runs_seen` / `report_count` / `first_seen` / `evidence` (the citation is
     generated from real docket data, satisfying the cite-the-research rule), and
     a placeholder acceptance-criteria block the human/next-dream fills in.
   - an append to `~/wintermute/autobuilder/notes/gossip.md` noting the drafted
     stub and its source finding.
2. **PRD-existence check** — a finding is "already owned" if
   `PRD-mend-<key>.md` exists (naming convention) OR a `prd:` annotation is
   present on the docket record. The bridge **never** overwrites or edits an
   existing PRD (hard rule #2 of /dream).
3. **`--dry-run`** (default) vs `--write` — dry-run prints what it *would*
   draft without touching the filesystem; `--write` actually creates stubs.
   Safe-by-default: a bare `mend bridge` shows the plan, never writes.
4. **`mend status`** — a compact report of open escalated findings and whether
   each has a PRD (owned / orphan), for the self-review banner.

Hard constraints (the fleet's settled proposal-only stance, mirroring
recourse-contest):
- **Proposal only.** Never invoke /build, never compile, never mark a finding
  resolved, never `docket resolve`. Drafting a stub is the entire action.
- **Never mutate existing PRDs or docket records** beyond an optional additive
  `prd:` annotation (and only with `--annotate`, off by default).
- **Idempotent.** Running `mend bridge --write` twice drafts each stub once;
  the second run is a no-op (existence check).
- SIGPIPE reset per [[self_sigpipe_panic_toolkit]]. rustc 1.85, no let-chains.

## Acceptance criteria

1. `cargo build --release` + `cargo test` green in `~/wintermute/mend`.
2. Against a fixture `docket list --escalated --format json` containing two
   escalated findings, one with an existing `PRD-mend-<key>.md` and one without,
   `mend bridge --dry-run` reports it would draft exactly one stub (the orphan).
3. `mend bridge --write` creates `PRD-mend-<key>.md` for the orphan only; the
   owned finding's PRD is untouched (byte-identical before/after).
4. The generated stub contains `Status: Draft v0.1`, `Vision: visions/mend.md`,
   and a "Why this exists" section quoting the finding's real `runs_seen` and
   `evidence` from the fixture (citation is data-derived, not boilerplate).
5. Idempotence: a second `mend bridge --write` makes no filesystem changes
   (verified: no new files, existing stub unchanged).
6. The bridge never calls `docket resolve`, never invokes cargo/build, and never
   edits a pre-existing PRD — verified by asserting no writes outside the single
   new stub + the gossip append.
7. `mend status` lists each open escalated finding as `owned` or `orphan` and
   exits 0; SIGPIPE: `mend status | head -1` exits 0, no panic.
8. A bare `mend bridge` (no flags) is dry-run: it writes nothing to disk.
