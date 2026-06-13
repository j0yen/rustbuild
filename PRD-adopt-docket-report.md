# PRD: adopt-docket-report — unadopted artifacts get a docket entry, a streak, and an escalation

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/adopt
Vision: visions/docket.md (Fleet 2 — the adoption forcing function)

## TL;DR

`adopt scan` (PRD-adopt-scan) detects shipped artifacts that never got
installed. But a detector that only prints is the same trap docket was
built to escape: noticed every run, parked every run. This PRD wires
`adopt scan` into the **docket** ledger so each unadopted artifact gets
a stable key, accrues a consecutive-run streak, auto-escalates at the
3-run threshold, and auto-closes the run after it's finally installed —
turning "tracked forever" into "tracked, escalated, then closed." This
is the first non-self-review producer reporting to docket, exactly the
"who else reports?" boundary the docket vision left open.

## Why this exists (Phase 1 evidence, 2026-06-12)

- **docket is live and proven.** `~/.local/bin/docket` is installed
  (v0.5.0; `~/.local/share/docket/docket.db` exists). The `docket
  report --run <id> --key <slug> --title <t> [--severity] [--evidence
  <kind>:<ref>]` contract is shipped and stable (verified `docket
  report --help`). Evidence kinds include `path:` and `commit:` —
  exactly what an adoption finding needs to point at.
- **The producer boundary is explicitly waiting.** docket vision open
  question #4: "Future producers (vigil's binstale … /build blockers)
  could all report to one docket — left as a vision boundary note … until
  the contract proves out with the self-review alone." It has proven
  out (v0.5.0, self-review binding shipped). adopt is the first new
  producer.
- **The thing to track is real and aging.** `rollout` unadopted 9 days
  (built 2026-06-03, not on PATH); `warden` inert; the staleness prose
  recurs in self-review every run 2026-06-08→12. A docket key
  `adopt:rollout` with a 5-run streak is a forcing function; a prose
  note is not.

## What this builds

A new subcommand `adopt report` in the existing `adopt` crate (extends
PRD-adopt-scan's binary), that consumes `adopt scan --format json` and
emits one `docket report` invocation per actionable artifact.

### Behavior

- `adopt report --run <id>` runs the scan internally (or accepts
  `--from-json -` to read a prior scan), and for each artifact whose
  verdict is `not-installed` or `installed-stale`:
  - Computes a stable key `adopt:<bin-name>` (e.g. `adopt:rollout`).
  - Calls `docket report --run <id> --key adopt:<bin> --title
    "<bin> built but not adopted (<verdict>)" --severity warn
    --evidence path:<repo> --evidence commit:<source_sha>`.
  - Severity escalates to `error` when `is_daemon` is true (a stale or
    missing daemon binary is higher-stakes than a missing CLI).
- Artifacts that are `installed-current` are **not** reported on (their
  absence from the run is what lets `docket sweep` auto-close a
  previously-open `adopt:<bin>` entry — the close path is docket's, not
  adopt's).
- `--dry-run` prints the `docket report` commands it would run without
  executing them.
- If `docket` is not on `$PATH`, `adopt report` exits non-zero with a
  clear message naming the missing dependency (and, fittingly, its own
  `adopt`-style fix line for installing docket) — it does **not**
  silently no-op.

### Run identity

`--run <id>` is caller-supplied and opaque (docket's documented model).
self-review passes its review run id (e.g. `2026-06-12.1`); a manual
invocation may pass any string. adopt does not invent run identity.

### Shape

- Pure orchestration over two CLIs (`adopt scan` self-call + `docket
  report` subprocess). No new heavy deps.
- Subprocess invocation must not be vulnerable to argument injection:
  pass artifact-derived strings as discrete `args`, never via a shell
  string.
- SIGPIPE-safe like the rest of the crate.

## Acceptance criteria

1. `adopt report --run R --dry-run` prints one `docket report …` command
   per `not-installed`/`installed-stale` artifact and executes nothing.
2. Each printed/executed `docket report` carries `--key adopt:<bin>`,
   a non-empty `--title`, and at least one `--evidence` ref of kind
   `path:` (the repo) and one of kind `commit:` (the source sha).
3. `installed-current` and `not-a-bin` artifacts produce **no** `docket
   report` invocation.
4. A `not-installed` daemon-backed artifact is reported with
   `--severity error`; a `not-installed` plain CLI with `--severity
   warn`.
5. Running `adopt report --run R` twice with the same `R` does not
   create duplicate streak increments (relies on docket's documented
   within-run dedupe; the test asserts the same key/run pair is passed,
   not adopt-side dedupe).
6. With `docket` absent from `$PATH`, `adopt report` exits non-zero and
   the message names `docket` as the missing dependency. It does not
   crash or silently succeed.
7. `adopt report --from-json <file>` accepts a previously captured
   `adopt scan --format json` payload and reports from it without
   re-running the scan.
8. No artifact-derived string reaches a shell; subprocess args are
   passed as a discrete argument vector (verified by a test with a repo
   name containing shell metacharacters — it is passed literally).
