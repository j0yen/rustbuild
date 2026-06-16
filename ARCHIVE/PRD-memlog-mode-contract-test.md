# PRD: memlog-mode-contract-test — three sources of truth for the device mode must agree

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/memlog
**Vision:** visions/continuity.md (Activation Fleet 2.0 — memlog capture)

## TL;DR

The memlog write outage was caused by three sources of truth disagreeing on one
octal number: the kernel driver says the device node is `0660`, the README says
`0660`, but the packaged udev rule said `0640` — and the rule wins. No test
caught the contradiction. This PRD extends the `memlog` repo with a contract
test that parses the device mode from all three sources and asserts they agree,
so the skew cannot silently return across kernel-version bumps.

## Why this exists

Root-caused live 2026-06-16 (full evidence in `PRD-memlog-udev-mode-repair`):

- driver `~/wintermute/memlog/driver/memlog.c:401` → `.mode = 0660`.
- `~/wintermute/memlog/README.md:47` → documented udev rule `MODE="0660"`.
- packaged `~/wintermute/wintermute-kernel/pkg/linux-wintermute-memlog.rules` →
  `MODE="0640"` (the installed, authoritative value — and the bug).

These three files live in two repos and drift independently every time the
kernel package is regenerated or the driver is touched. The driver and README
happened to stay correct; the packaged rule did not, and the failure mode was
silent — the only symptom was an empty ring that everyone read as "expected."
A mechanical cross-source assertion is the durable guard, in the spirit of the
`apply-agentns.py` anchor-based pattern the kernel package already uses for
durability across version churn (see CLAUDE memory: kernel-asset arc).

## What this builds

A `rust-extend` addition to the existing `memlog` crate — a new test module
(and a tiny supporting parser, exposed as a library function so it is unit- and
integration-testable) that:

1. **Parses the driver mode.** Reads `driver/memlog.c`, finds the `.mode =`
   assignment for the device/cdev registration, extracts the octal literal
   (`0660`). Anchor-based (match on `.mode` near the device-create/cdev block),
   not line-number-based, so it survives edits above it.

2. **Parses the README documented mode.** Reads `README.md`, finds the
   `KERNEL=="memlog"` udev-rule example line, extracts `MODE="...."`.

3. **Parses the packaged udev rule mode.** Reads the packaged rule file. Its
   path is resolved in this order: (a) an env override
   `MEMLOG_UDEV_RULE_PATH` (for test fixtures / CI); (b) the in-tree kernel-pkg
   copy at `../wintermute-kernel/pkg/linux-wintermute-memlog.rules` relative to
   the memlog repo; (c) the installed
   `/usr/lib/udev/rules.d/70-linux-wintermute-memlog.rules`. The first that
   exists wins; the test records which source it used.

4. **Asserts agreement.** All three parsed modes must be byte-equal as
   normalized octal (e.g. `0660`). On mismatch the test fails with a message
   that prints each source, its path, and its parsed value — so the failure
   names the divergent file directly.

5. **Sanity floor.** Independently asserts the agreed mode grants group-write
   (`(mode & 0o020) != 0`) and group membership is the intended access path —
   catching a future "all three agree on `0640`" regression where they are
   consistent but still wrong. A consistent-but-broken state must fail too.

### Shape / deps

- Pure Rust string parsing; no new heavy deps (regex already in the memlog
  workspace, or hand-rolled scanning if not — prefer no new dep). Parser is a
  `pub fn parse_mode_from_*` set returning `Result<u32, _>`.
- Lives under `tests/` plus a small `src/` helper module so the parser is
  reachable both as a unit test and the integration contract test. Ensure the
  integration test is reachable from a top-level `tests/<name>.rs` entry (not an
  orphaned `tests/mocks/` subdir) so it actually runs — verify `Running
  tests/<name>.rs` appears in `cargo test` output.
- No wall-clock, no network. Fully deterministic given the three files.

## Acceptance criteria

1. A library function parses `0660` from a fixture mirroring `driver/memlog.c`'s
   `.mode = 0660` line, and is robust to surrounding whitespace/comments.
2. A library function parses `0660` from a fixture README udev-rule line
   `KERNEL=="memlog", GROUP="memlog", MODE="0660"`.
3. A library function parses the mode from a fixture udev rule file, honoring
   the `MEMLOG_UDEV_RULE_PATH` env override to point at the fixture.
4. The contract test passes when all three fixtures agree on `0660`.
5. The contract test **fails** with a message naming the divergent source and
   its path when the udev-rule fixture is set to `0640` while the others are
   `0660` (this reproduces the actual 2026-06-16 bug as a regression fixture).
6. The contract test **fails** when all three agree on `0640` (the
   consistent-but-broken case), citing the group-write sanity floor.
7. The integration test is wired through a top-level `tests/*.rs` entry and is
   observed `Running` in `cargo test` output (no silently-skipped orphan).
8. `cargo test` for the memlog crate is green with the new module, and `cargo
   build` is unaffected (the parser helper does not change runtime behavior of
   the CLI or driver).
9. The test resolves the real packaged rule path (resolution order (a)→(b)→(c))
   when no fixture override is set, and records which source it used in the test
   output, so running it on this box exercises the live files.

## Out of scope

- Fixing the live mode or the packaged rule (that is
  `PRD-memlog-udev-mode-repair`). This PRD is the *guard* that keeps the fix
  from silently regressing; it asserts, it does not mutate.
- Parsing any kernel surface beyond the three named files (no `/sys` probing,
  no live device `stat` — that runtime check is `memlog-capture-selfcheck`'s
  job; this is a static cross-source contract).
- Validating other udev attributes (GROUP, SYMLINK) beyond MODE — could be a
  follow-on if a group-name skew ever appears, left as a vision open question.
