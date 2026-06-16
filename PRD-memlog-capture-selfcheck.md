# PRD: memlog-capture-selfcheck — an empty ring after a compaction is an alarm, not a shrug

**Status:** Draft v0.1
**build_target:** rust-cli
**Vision:** visions/continuity.md (Activation Fleet 2.0 — memlog capture)

## TL;DR

For weeks, `/dev/memlog` has been empty and self-review read it as *normal*
every single run. The actual cause — a wrong udev mode that broke every write —
went undetected precisely because nobody distinguished "empty because nothing
should have written yet" from "empty although the writer fired and failed."
This PRD ships `memlog-capture-selfcheck`: a small CLI that cross-references the
PreCompact writer's own firing log against the ring's write count and emits a
**RED, docket-consumable** finding when the writer fired since boot but the ring
is still empty. It turns a silently-misread health line into a self-correcting
alarm.

## Why this exists

The failure that motivated `PRD-memlog-udev-mode-repair` survived for weeks not
because the evidence was hidden but because the *interpretation* was wrong.
Primary evidence, 2026-06-16:

- `~/.cache/memlog/precompact.log` records 36 PreCompact firings, the last 17+
  all `FAIL(1): … Permission denied`. The proof of breakage was sitting in a
  log the whole time.
- Yet `~/brain/journal/2026-06-15.md` and siblings repeatedly conclude: "Ring
  empty (total_writes=0) … is expected." The self-review playbook had no rule
  that says: *if the writer fired and the ring did not grow, that is a fault.*
- The memlog primitive shipped 2026-05-24; the gap went uncaught for ~3 weeks.

The lesson generalizes: a health signal that is always interpreted as benign is
indistinguishable from a broken one until something correlates *intent to write*
with *evidence of a write*. That correlation is exactly what this CLI encodes.

## What this builds

A standalone Rust CLI `memlog-capture-selfcheck` (published `j0yen/memlog-capture-selfcheck`,
installed to `~/.local/bin`). It is read-only and deterministic.

### Inputs it reads

- `memlog stats` (parse `total_writes`, `records_in_ring`, `newest_ts_ns`).
  Shell out to the installed `memlog` binary; if absent, that is itself a
  finding (`memlog-missing`).
- `~/.cache/memlog/precompact.log` — the writer's own append-only log. Each line
  begins with an ISO-8601 UTC timestamp; entries are classified `OK:`, `FAIL(`,
  or `SKIP:`.
- System boot time, via `/proc/stat` `btime` (seconds since epoch). No
  wall-clock call in logic — `--now <epoch>` overrides for deterministic tests;
  default reads a single `date +%s` at the process boundary, never inside
  pure functions.

### The check

Classify into one verdict:

- **GREEN (`capturing`)** — `total_writes > 0` and the newest ring write is at
  or after the newest `OK:` line in the log. The pipeline is live.
- **RED (`firing-but-empty`)** — there is at least one `FAIL(` *or* `OK:` log
  entry with a timestamp ≥ boot time (the writer fired this boot) **and**
  `total_writes == 0`. This is the exact failure class that masked the udev
  bug. Message names the last failure reason verbatim from the log.
- **AMBER (`untested`)** — no log entries since boot at all (no compaction has
  happened this boot, so we genuinely cannot tell). Distinguished from RED so
  it does not cry wolf on a fresh boot.
- **`memlog-missing`** — the `memlog` binary or `/dev/memlog` is absent.

### Output / wiring

- `--format human` (default): one line per verdict with the evidence (write
  count, last log verdict + reason, boot-relative firing count).
- `--format json`: `{verdict, total_writes, fired_since_boot, last_fail_reason,
  last_ok_ts, boot_ts}` for docket / self-review ingestion.
- Exit code: `0` GREEN/AMBER, `3` RED, `4` memlog-missing — so self-review's
  Phase B.5 playbook can gate on it.
- A `--docket` flag prints a single `docket report`-shaped line (key
  `memlog-capture-firing-but-empty`, title, evidence) so the finding flows into
  the standing-findings ledger instead of being re-discovered each run.

### Shape / deps

- `clap` for args, `serde_json` for JSON out. No async, no network. Parsing the
  log and `memlog stats` is plain string work.
- Pure verdict function takes `(stats, log_entries, boot_ts)` and returns the
  verdict enum — fully unit-testable with fixtures, no I/O.

## Acceptance criteria

1. `memlog-capture-selfcheck --format json` against a system with
   `total_writes > 0` and a recent `OK:` log line emits `verdict: "capturing"`
   and exits 0.
2. Given a fixture log containing a `FAIL(` entry timestamped after a supplied
   `--now`/boot time and `total_writes == 0`, the tool emits
   `verdict: "firing-but-empty"`, includes the verbatim last failure reason in
   `last_fail_reason`, and exits 3.
3. Given an empty/no log since boot, the tool emits `verdict: "untested"` and
   exits 0 (does not false-alarm on a fresh boot).
4. With the `memlog` binary or `/dev/memlog` absent (simulated via a
   `--memlog-bin <path>` override pointing at a nonexistent file), the tool
   emits `verdict: "memlog-missing"` and exits 4.
5. The core verdict function is exercised by unit tests over all four verdicts
   using in-memory fixtures (no filesystem, no shelling out).
6. `--docket` prints exactly one line keyed `memlog-capture-firing-but-empty`
   with non-empty evidence when the verdict is RED, and prints nothing (exit 0)
   when GREEN/AMBER.
7. Boot time is read exactly once at the process boundary (or taken from
   `--now`); no `Date`/wall-clock call appears inside the verdict logic (grep
   the verdict module for `SystemTime::now`/`Instant::now` → none).
8. `--format human` output names the write count, the last log verdict, and the
   firing-count-since-boot on a single readable line.

## Out of scope

- Fixing the underlying permission bug (that is `PRD-memlog-udev-mode-repair`;
  this PRD only *detects* the symptom and would flip GREEN once repair lands).
- Wiring the check into the self-review timer or docket cron — the tool exposes
  the exit code and `--docket` line; the actual playbook edit is a separate,
  user-gated self-review change.
- Reading or interpreting snapshot *contents* — this checks that writes land,
  not what they say.
