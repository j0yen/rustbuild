---
name: rustbuild
description: PRD-driven, rigorously validated Rust code generation. Use when the user wants to build a Rust CLI or library from a Product Requirements Document under an autonomous iterate-and-prove loop with structured receipts and a 7-receipt risk gate. Synthesizes autoresearch's locked-harness model, jankurai's anti-pattern catalog, and jeryu's proof-receipt gate into one pipeline.
---

# autobuilder

## What this skill does

Takes a PRD (file path or pasted text) and drives a 5-stage pipeline that yields a Rust project where every artifact is (a) generated from a structured `intent-card.json` derived via 4/5-Whys, (b) iterated under a narrow falsifiable advance-or-revert loop, (c) accompanied by per-iteration `EvidencePack` receipts and `FailureCapsule`s on crash, (d) gated by 8 receipts before declaring "ready," and (e) fed into a postmortem that queues self-improvement proposals.

## Where cargo runs (merged from /cloudrustbuild, 2026-09-02)

Fleet names: **RedBaron** is the Rust build machine; **carbon** and **ryzen7**
are the other workstations; **Wintermute Hub** is the Hetzner box that runs NATS
and builds nothing. The rule:

- **On RedBaron: cargo runs locally.**
- **On any other node: cargo runs on RedBaron.** Every stage that touches cargo
  first prepends this skill's shim to PATH:
  `export PATH="$HOME/.claude/skills/rustbuild/bin:$PATH"`. The shim
  (`skill/bin/cargo`) is a pass-through on RedBaron; elsewhere it syncs the
  crate to RedBaron over Tailscale SSH, runs the same cargo command there with
  RedBaron's sccache and mold, and pulls `target/` back (`cargo install`, `new`,
  `--version` and the like stay local). Config lives in
  `~/.config/wm-burst/hub.json` (`{"ip":"100.73.175.108","user":"jsy",
  "build_root":"/home/jsy/build","sccache_dir":"/home/jsy/.cache/sccache",
  "prefer":"always"}` — `hub.json` is the script's legacy name for the build
  machine, not Wintermute Hub).
- **RedBaron unreachable: stop.** The shim exits 2; the stage records
  `blocked: RedBaron unreachable`. It never falls back to a local build and
  never rents a server (the Hetzner burst box was retired 2026-09-01).

Direct use: `bash ~/.claude/skills/rustbuild/scripts/cargo-on-redbaron.sh
status|doctor|route <crate>|build <crate> -- <args>|test <crate> -- <args>|ssh`.

## When to invoke

Invoke when the user:
- Hands you a PRD and asks for a Rust project (CLI or library).
- Says "build me a Rust X" with concrete acceptance criteria.
- Asks to dogfood autobuilder against one of its own sub-tools.

Do NOT invoke for:
- Greenfield non-Rust projects.
- Surgical edits to an existing Rust crate (use direct tools).
- WASM or embedded targets. (A crate that serves HTTP is still a `--target cli`
  or `lib` crate and goes through this skill — a `build_target: rust-*` PRD is
  never routed around /rustbuild because of what the binary does. 2026-09-02.)

## Resolved decisions (locked 2026-05-21)

1. **Skill + companion Rust binary.** Skill orchestrates Claude subagents; the companion `autobuilder` binary (Cargo workspace at `../autobuilder/` in the repo, installed via `cargo install --path autobuilder`) owns the metric harness, receipt writing, risk gate, and experiment-loop runner. Skill shells out to the binary. Binary is itself dogfooded.
2. **Target scope: CLIs + library crates.** `--target cli` or `--target lib`. For libs add cargo-semver-checks, docs-coverage, `cargo public-api` diff. A CLI/lib that happens to run a server is in scope; WASM/embedded → v2.
3. **Hybrid autonomy.** Loop and risk gate run fully autonomous. Human checkpoint only when the agent wants to add/relax a MUST acceptance criterion, widen hard constraints, or modify the skill itself. Trigger via `intent_card_amendment_request.json`.
4. **First PRD is the metric harness.** Build `autobuilder-metric-harness` (input: project path; output: normalized `metrics.json`) before throwing external PRDs at autobuilder.

## Pipeline

```
PRD ──► Stage 1: Intake (5-Whys)         ──► intent-card.json
        Stage 2: Scaffold (locked harness) ──► <project>/ tree
        Stage 2.5: Spec-Drift Probe         ──► spec-drift.json (block if PRD names absent CLI verbs)
        Stage 3: Iterate-and-Prove loop    ──► EvidencePack per iter, FailureCapsule on crash
        Stage 4: Risk Gate (9 receipts)    ──► ready / blocked + diagnostic
        Stage 5: Postmortem + Self-Evolve  ──► gated proposal queued for review
```

### Stage 1 — Intake (5-Whys)

Run `prompts/prd-intake-5whys.md` against the PRD. Output validates against `schemas/intent-card.schema.json`. Refuse to proceed if ambiguity remains after 5 Whys — surface what's missing and ask the user.

### Stage 2 — Scaffold

Generate a project where harness is read-only, agent edits only `src/`:

```
<project>/
├── Cargo.toml, clippy.toml, deny.toml, rust-toolchain.toml
├── src/                                  ← agent edits ONLY this
├── tests/acceptance_*.rs                 ← one test per AC (read-only)
├── tests/mocks/ac<N>.rs                   ← mock per hardware-deferred AC (see "Hardware mock convention")
├── tests/proptest_invariants.rs          ← read-only
├── tests/fuzz/                           ← cargo-fuzz harness
├── scripts/run-metrics.sh                ← read-only; emits metrics.json
├── scripts/audit.sh                      ← read-only; BAD_RUST scan
├── scripts/risk-gate.sh                  ← read-only; checks 7 receipts
├── agent/
│   ├── AUTOBUILDER_PROGRAM.md            ← autoresearch-style instructions
│   ├── intent-card.json
│   ├── owner-map.json
│   ├── test-map.json
│   └── proof-lanes.toml
├── target/autobuilder/
│   ├── receipts/<sha>.json               ← EvidencePack per iteration
│   ├── failure-capsules/                 ← FailureCapsule per crash
│   ├── results.tsv
│   └── postmortem.md
└── .github/workflows/                    ← CI mirror of local gate
```

### Stage 2.5 — Spec-Drift Probe

Run `scripts/spec-drift-probe.py <PRD-path>` after the Stage 2 scaffold,
before entering the Stage 3 iterate-and-prove loop. The probe extracts every
backticked CLI invocation the PRD names (e.g. `recall list --since 7d`,
`letter-curate aggregate`), runs each binary's `--help`, and diffs the
PRD-asserted subcommand verbs against the tool's real subcommand surface.

- **exit 0** — no drift: every asserted verb exists, or the only flagged
  tools are `unavailable` (binary not on `$PATH` — intentional future-state)
  or `help_fails` (help unparseable / flat CLI — no positive surface to diff).
- **exit 4** — drift: the PRD names a verb a real binary does not expose.
  Abort before Stage 3 and surface `target/autobuilder/spec-drift.json` as the
  diagnostic. This catches the cadence-bind-letters failure mode (PRD assumed
  `letter-curate aggregate`; the binary only does `triage`/`show`/`list`)
  before any /rustbuild cycle — or any parallel /build sibling branch — is
  burned chasing a hallucinated verb.
- **exit 2** — bad invocation (missing PRD path).

Hedged inline mentions ("presumably `wm-tts metrics`", "e.g. `tool foo`") and
non-clap help formats are treated conservatively (no drift) so the gate never
false-positives on speculative future-design prose. `--strict` additionally
blocks on `help_fails`.

The verdict (`spec-drift.json`) is preserved as Receipt #8 in the Stage 4 gate
once Stage 3 completes. Costs <2s. Receipt schema: `schemas/spec-drift.schema.json`
(`spec-drift.v1`). PRD: autobuilder-spec-drift-probe.

### Stage 3 — Iterate-and-Prove

```
LOOP UNTIL all-MUST-ACs-green AND risk-gate-passes OR budget-exhausted:
  1. git state check; branch is autobuilder/<intent-slug>
  2. Edit src/ ONLY (edit-agent generates the diff)
  3. git commit -m "iter-<n>: <hypothesis>"
  4. scripts/run-metrics.sh > target/autobuilder/run.log 2>&1
     4b. If the crate has tests, invoke
         `~/.claude/skills/rustbuild/scripts/run-mutants.sh <crate_dir>`
         (PRD autobuilder-mutation-testing, Phase 1 — telemetry only). It
         runs cargo-mutants (installing it once if absent), merges
         `mutants_total / mutants_killed_count / mutants_alive_count /
         mutation_kill_rate / mutation_wall_seconds` into metrics.json, and
         writes `target/autobuilder/mutants-receipt.json`. Phase 1 is NOT a
         hard gate: a low kill_rate is logged, never blocks (the script
         exits 0 unless cargo-mutants infrastructure fails). Cached by
         sha256(src/+tests/+Cargo.toml); slow, so safe to skip on rapid
         inner iterations and run before the Stage 4 gate. Phase 2 (a
         calibrated `mutation_kill_rate < THRESHOLD` hard gate) is a
         follow-on PRD after 20 crates ship mutation data.
  5. Parse target/autobuilder/metrics.json
  6. If crash: tail -n 50 run.log; ≤3 fix attempts; else FailureCapsule + status=crash
  7. Append to results.tsv: <sha> <quality_score> <ac_passing> <status> <description>
  8. Advance if: all hard gates pass AND quality_score improved AND no MUST-AC regression
     Else: git reset --hard HEAD~1; status=discard
  9. Emit EvidencePack JSON for the iteration
 10. (Optional) Adversarial sub-step: spawn the adversarial-agent
     (prompts/adversarial-agent.md) to write tests/adversarial_<id>.rs
     attempting to falsify the AC against its English description, not
     the implementation. If any adversarial test fails on the
     implementation, downgrade verdict from advance to concern and
     surface the failure to the next edit-agent iteration. Closes the
     "edit-agent wrote both impl and test" tautology gap.
 11. After all hard gates pass (and before the Stage 4 gate), run the
     semantic AC judge:
     `ac-judge run --prd <prd_path> --crate-root <crate_dir>`
     (binary from j0yen/ac-judge; install via that repo's `install.sh`).
     It pairs each declared AC's English text with the test claiming to
     verify it and asks an independent model two questions: does the test
     exercise the AC's behavior, and does it assert the AC's invariant or
     merely restate the impl? The judge runs on a pluggable backend
     (`--backend auto`, the default): **codex** (`codex exec`, preferred —
     a genuinely different model family from the Claude implementer) first,
     then the Anthropic API (`$ANTHROPIC_API_KEY`), then `claude login`'s
     headless CLI (`claude -p`) as the last resort. Emits
     `target/autobuilder/ac-semantic-judge.json` (Receipt #9, schema
     `schemas/ac-semantic-judge.schema.json`, `backend` field records which
     one judged). Exits 4 if any AC has `behavior_match: no` OR
     (`assertion_kind: restates-impl` AND `confidence >= 0.7`); that exit is
     the Stage 4 `ac-semantic-judge` block. Requires a judge backend:
     `codex login`, `$ANTHROPIC_API_KEY`, or `claude login` (exits 6 if none
     is available, no network call). Complements mutation testing: mutation
     asks "would the test catch a broken impl?"; the judge asks "does the
     test check the right thing at all?"
```

Hard gates (all must pass to advance):
- `cargo check --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo deny check`
- `cargo +nightly miri test` (when `--allow-unsafe`)
- BAD_RUST audit scan (`rules/bad-rust.md` + `rules/audit-checks.sh`)
- Proof-lane routing: every changed path resolves to ≥1 lane, all lanes green

Quality score (drives advance/revert tiebreak):
```
score = 10*ac_passing_count
      +  5*mutation_kill_rate          # 0..1; PRD autobuilder-mutation-testing
      +  3*test_coverage_pct
      +  2*proptest_density
      +  1*doc_coverage_pct
      -  2*audit_findings_count
      -  1*clippy_warning_count
```

`mutation_kill_rate` (0..1) comes from `scripts/run-mutants.sh` (step 4b);
weighted high because mutation testing *proves* a test would fail if the
impl broke, a stronger signal than the `proptest_density` heuristic. When
mutation data is absent (`null` — script skipped or not yet run), treat the
term as 0 so the score stays defined. Phase 1 weights it; Phase 2 will gate
on it (per PRD autobuilder-mutation-testing).

### Stage 4 — Risk Gate (25 receipts)

| Receipt | Source | Pass condition |
|---|---|---|
| `intake` | Stage 1 | `intent-card.json` validates; all MUST-ACs declared |
| `spec-drift` | Stage 2.5 | `target/autobuilder/spec-drift.json` `summary.drift_count == 0` |
| `vti-plan` | Stage 2/3 | every changed path routed via `proof-lanes.toml`; confidence ≥ 0.70 |
| `proof-receipt` | Stage 3 | test/proptest/fuzz/miri/deny green on `HEAD` |
| `ac-semantic-judge` | Stage 3 step 11 | `target/autobuilder/ac-semantic-judge.json` `passed == true`: every AC verdict has `behavior_match != no` AND not (`assertion_kind == restates-impl` AND `confidence >= 0.7`) |
| `risk-gate` | Stage 3 | BAD_RUST audit clean (or only `advisory` findings with waivers) |
| `reviewer-agent` | sub-agent | independent Claude review of `HEAD~N..HEAD` vs intent-card; ∈ `{pass, concern, block}`. Verdict appended to `state/reviewer-calibration.jsonl` (see "Reviewer calibration & phased graduation"). **Phase A (current): both `pass` AND `concern` ship — `concern` is logged `shipped: true` and proceeds (advisory). `block` still blocks.** |
| `rollback-plan` | Stage 2 | every commit `git revert`-clean; steps in `target/autobuilder/rollback.md` |
| `ci-checks` | Stage 2 | `.github/workflows/` green on a fresh worktree clone |

Missing receipts → block + machine-readable diagnostic. No self-approval.

**Extended receipts (17).** `autobuilder gate` has checked all 25 receipts
below since PRD-extended-gates shipped (2026-05-23) — the 9-row table above
is the original core; these 17 extend it with supply-chain,
reproducibility, performance, API-contract, and test-quality checks plus a
campaign roll-up. Each row's "Pass" column is its `ProducerSpec`/
`ReceiptSpec` `pass_verdicts` (`autobuilder/crates/gate/src/lib.rs`
`RECEIPT_SPECS`; `autobuilder/crates/extended-gates/src/lib.rs`
`PRODUCER_SPECS` — the two tables are kept in sync and asserted equal by an
integration test). Every extended producer also accepts `block`, which
always fails the gate.

| Receipt | Purpose | Pass |
|---|---|---|
| `supply-audit` | scan `Cargo.lock` for deps listed in vendored RUSTSEC advisories | pass |
| `license-audit` | every transitive dep's `license` field is in the allowlist | pass |
| `secrets-scan` | scan tracked source files for high-confidence secret patterns | pass |
| `sbom` | emit a CycloneDX-shape SBOM JSON of the workspace deps | pass |
| `determinism` | two cold `cargo build --release` runs produce identical artifact sha256 | pass, skipped |
| `hermetic-build` | detect outbound network sockets during `cargo build --offline` | pass, skipped |
| `msrv-verify` | declared `rust-version` actually compiles + tests clean | pass, skipped |
| `binary-size` | every `target/release/*` binary is under its configured budget | pass, skipped |
| `cold-build-time` | clean `cargo build --release` wall-time under budget | pass, skipped |
| `bench-delta` | criterion benches don't regress >X% vs a frozen baseline JSON | pass, skipped |
| `semver-check` | pub-API diff between `HEAD~1` and `HEAD` is semver-compatible | pass, skipped |
| `cli-surface` | every declared bin's `--help` output matches its snapshot | pass, skipped |
| `schema-compat` | receipt JSON schemas added/changed are additive-only | pass, skipped |
| `ac-traceability` | every PRD AC id has ≥1 Rust test fn referencing it | pass |
| `mutation-kill` | a small mutation-operator pass on `src/lib.rs` causes the test suite to fail | pass, skipped |
| `flake-audit` | `cargo test` rerun K times produces identical outcomes | pass, skipped |
| `experiment` | roll up a multi-slice campaign's per-slice outcomes into one receipt | pass, skipped |

**Producing the extended receipts.** Install once:
`cargo install --path ~/wintermute/rustbuild/autobuilder/crates/extended-gates --locked`.
Run each producer against a crate: `<name> --project <crate>` (e.g.
`ac-traceability --project ~/repos/foo`), which writes
`target/autobuilder/receipts/<name>-receipt.json`. Optional
`extended-gates.toml` in the crate root tunes thresholds: `prd_path`,
`mutation_kill_min_pct`, binary-size budgets. Two invariants any change to a
producer must preserve: the audit's findings must carry real `path`/`line`
values, never a bare line number where a path belongs (`skill/rules/
audit-checks.sh` — every single-file `grep` needs `-H`); and no producer may
touch `target/autobuilder/` (`determinism`/`cold-build-time` build in an
isolated `CARGO_TARGET_DIR`, never the project's own `target/`, precisely
so their `cargo clean` can't wipe every other producer's receipt).

**Reviewer model: Sonnet.** The `reviewer-agent` sub-agent dispatches
on `model: "sonnet"`, the same tier as the implementation loop, matching
`/pybuild`'s Stage 4 reviewer. Independence comes from a fresh agent that
reads the intent-card's English ACs rather than the code, not from a
bigger model. Escalate the reviewer to `model: "opus"` only when the user
explicitly asks (changed 2026-09-02; was always-Opus). Set
`model: "sonnet"` on the Agent/Task call that produces the reviewer-agent
receipt.

**Reviewer calibration & phased graduation.** Every `reviewer-agent`
verdict is appended as one line to
`~/.claude/skills/rustbuild/state/reviewer-calibration.jsonl`
(append-only JSONL; one `write()` per line, fsync after — durability over
throughput, fires once per crate ship). Line shape:

```json
{"ts": "2026-05-28T22:30:00Z", "slug": "foo", "verdict": "concern", "concern_summary": "..", "shipped": true, "post_ship_revert": null}
```

`post_ship_revert` is `null` at ship; a weekly /self-review sweep updates
it to `true`/`false` by scanning each shipped repo's git log for revert
commits in the 7-day window. The verdict gate graduates in three phases,
calibrated on this log (PRD autobuilder-reviewer-promotion):

- **Phase A (current — ships with this PRD):** `concern` is *advisory*.
  It is recorded in the calibration log marked `shipped: true` and the
  build proceeds. No behavior change beyond logging. `block` still blocks;
  `pass` ships clean.
- **Phase B (auto-promoted by /self-review once calibration `n >= 30`):**
  `concern` becomes a *soft-block*. Bypass via PRD frontmatter
  `reviewer_override: true` + a one-line `reviewer_override_reason:`; the
  override is recorded in the calibration log. (Override is honored in
  Phase B only.)
- **Phase C (auto-promoted once `concern_to_revert_rate >= 0.50` over the
  last 30 shipped):** `concern` becomes a *hard block* — no frontmatter
  override.

Phases B and C are SKILL.md edits performed automatically by /self-review's
`reviewer_promotion_check` playbook when the thresholds trip; they are NOT
active today. This PRD ships Phase A only.

### Stage 5 — Postmortem & Self-Evolve

`target/autobuilder/postmortem.md` summarizes the run. A run-level `evolution-proposal.json` queues in `~/.claude/skills/rustbuild/proposals/`. `autobuilder evolve` aggregates across the last K runs and emits a diff against `SKILL.md` / `rules/bad-rust.md` / `templates/scaffold/`.

**Auto-apply (default).** Each `Suggestion` is append-only by construction. `evolve` writes the appended lines to the target file in the skill tree, commits the change in the skill_root git repo when present (one commit per suggestion, message `evolve: <rationale>`), and records `applied-suggestion:<sha256-of-target-and-appended-lines>` in `proposals/applied.log` so the same suggestion does not re-emit on subsequent runs.

**Template-drift auto-apply (pure-additions only).** Postmortem captures `template_diffs` for every project script that has diverged from `templates/scaffold/scripts/*`. When the same diff body appears across ≥2 distinct slugs, evolve groups them and inspects the diff direction: a pure-additions diff (no `-` lines in hunk bodies) unambiguously means projects added content the template lacks → promoted to a `PatchSuggestion` and auto-applied via `patch --dry-run` guard + `patch -p0` + git commit. Anything with `-` lines is direction-ambiguous and surfaces as a `TemplateDriftAdvisory` for manual review.

Use `evolve --dry-run` to inspect both suggestion types without applying.

**Manual rejection still supported.** Add a basename to `applied.log` with a `#REJECTED:` comment block (existing convention) to suppress the source proposal entirely. Use this for suggestions whose underlying issue should be resolved elsewhere rather than by appending to the skill.

### Stage 6 — Publish (per-project repo)

When the slice passes the gate and the user wants to distribute it, the slice becomes its own GitHub repo under `github.com/j0yen/<slug>` rather than being squash-imported into a monorepo. (This replaces the older "Import into `~/wintermute/`" convention; see `j0yen/wintermute`'s [`REPOS.md`](https://github.com/j0yen/wintermute/blob/master/REPOS.md) for the live index and `bootstrap/install.sh` for the bootstrap installer that picks up new repos.)

Per-slice steps:

1. **README + LICENSE.** Generate `README.md` from `agent/intent-card.json` (root_motivation as overview, MUST-level acceptance criteria as the AC list). Drop dual `LICENSE-MIT` + `LICENSE-APACHE` files into the repo root.
2. **Rename branch to `main`.** The autobuilder scaffold uses `autobuilder/<slug>` as the working branch. Before publishing: delete any stale `main` (the iter-0 scaffold baseline) and rename `autobuilder/<slug>` → `main`. Skip this if you want to preserve the autobuilder branch name on GitHub.
3. **Commit + push.** Single commit "Prep for standalone distribution: README + dual MIT/Apache-2.0 license" using the `Joe Yen <jyen.tech@gmail.com>` identity for wintermute-ecosystem repos, then `gh repo create j0yen/<slug> --public --source . --remote origin --push`.
4. **Update wintermute's `REPOS.md`** with a one-line description and category (pipeline/runtime/memory/session/artist). The bootstrap installer will then pick up the new repo on next `install.sh` run.

The autobuilder companion binary does not yet automate Stage 6; it is a manual convention. A future `autobuilder publish` subcommand may codify this.

## PRD frontmatter the skill reads

Beyond the prose body, autobuilder's intake parser reads structured
fields from the PRD's YAML-ish frontmatter. The hardware-bound subset:

```
deferred_acs: [3, 7]
    # ACs that cannot be verified against real hardware/OS at gate-time
    # (live audio device, whisper-cpp inference, OS scheduler under load,
    # systemctl side effects, …). Introduced by PRD-build-deferred-acs.
    # Deferring is honest about the constraint, but a deferred AC with no
    # mock has NO behavioral verification wired in. So each deferred AC
    # must take ONE of the two paths below.

mock_unjustified_for: [3, 7]
    # The subset of `deferred_acs:` that ALSO can't sensibly be mocked.
    # Requires a `mock_justifications:` companion entry per listed AC.
    # An AC listed here with no companion justification is a parser error.

mock_justifications:
  3: "AC3 verifies hardware fan speed via thermal pressure; no mock can
      simulate the real PWM signal without recreating the firmware."
  7: "AC7 requires the OS scheduler under load; a mock would either be a
      tautology or a different scheduler."
    # One sentence per AC in `mock_unjustified_for:` explaining why a
    # mock isn't tractable. Deferring is fine; deferring without an
    # explanation is not.
```

### Hardware mock convention

For every AC in `deferred_acs:` but NOT in `mock_unjustified_for:`, the
crate must ship `tests/mocks/ac<N>.rs` (e.g. `tests/mocks/ac3.rs`). The
mock test:

- exercises the **same public API surface** the real test would (same
  call sequence + signatures), so the boundary type-checks as the real
  path does;
- runs against a **documented in-crate fake** — a trait impl, channel
  pair, in-memory device, etc. — not a network or hardware dependency;
- **asserts the same invariant** the AC's English text declares.

The mock proves the call sequence + signature + invariant *at the type
level*; a later `cargo test --features=real-hardware` run proves they
hold *in the world*. Both, not either. This is the discipline the rest
of the Rust ecosystem uses for hardware-adjacent code (kernel drivers,
embedded crates, every IO library with a fake-fs feature). The mock
COMPLEMENTS reality; it does not REPLACE it.

Mock tests run under `cargo test` by default, so they count toward the
Stage 3 `cargo test --workspace` hard gate and toward /build's
verified-completed checklist (the OR-clause: an AC passes if it has a
real passing test, OR a passing `tests/mocks/ac<N>.rs` plus a
`deferred_acs:` listing, OR a `mock_unjustified_for:` +
`mock_justifications:` entry). An AC with none of the three is a hard
fail.

**Lint parity.** Mock test files are subject to the same lint discipline
as real test files (`unwrap`/`expect`/`panic` = deny per `rules/bad-rust.md`).
The mock is real Rust, not a magic affordance.

**Scope (v0.1).** Mocks are hand-written — the mock IS the documentation
of what the boundary looks like, so no auto-generation from a trait. Each
crate's mocks are local (no cross-crate mock libraries). A
`hardware-drift.json` receipt comparing mock vs. real-hardware outcomes is
scaffolded as a follow-on PRD, not invoked by default.

## Reused skills

- `/loop` — long-running experiment cadence.
- `/verify` — final end-to-end app-run check (Stage 4).
- `/code-review` — `reviewer-agent` receipt.

## Local tool integration

The companion binary owns the inner loop, but Claude running this skill
manually (or extending the binary) should reach for these wrappers
rather than rolling equivalents:

- **`wchg watch <project>`** at Stage 2 scaffold-complete; **`wchg
  since <project>`** between iterations as a write-scope guard. Stage 3
  promises the agent edits only `src/`. If `wchg since` surfaces a
  write outside `src/` (or outside the target/autobuilder/ receipts
  dir), that's an escape: emit a `FailureCapsule` with type
  `scope_escape`, revert via git, and don't count it as an iteration.
  Catches a class of failures git status would miss because the diff
  might also be reverted by the test command itself.
- **`sbx --no-net --profile pwd -- <cmd>`** wrapping
  `scripts/run-metrics.sh` and any `cargo test`/`cargo run` invocations
  in Stage 3. PRD-generated test code is untrusted-by-construction;
  isolating its network + write surface is the right default. Add
  `--bind <project>/target` so cargo's incremental build cache
  persists across iterations. Skip `--no-net` only if a specific
  acceptance test explicitly requires network egress (rare).
- **`txn-edit snap`** around the Stage 3 edit-agent diff application
  when the agent touches >1 file in one iteration. `git reset --hard
  HEAD~1` (the current revert path) loses partial progress on the
  iteration; `txn-edit rollback` preserves the diff for the next
  edit-agent prompt to learn from.
- **`procstat snap <cargo-pid>`** during long Stage 3 iterations.
  Capture peak `vm_rss_bytes` and `io_write_bytes` into the
  EvidencePack — a regression in resource cost is a quality signal the
  current score function doesn't reward but matters for tools shipped
  to `~/.local/bin/`.
- **`pevent run <cargo test --release>`** for the Stage 4
  full-test-suite verification when `cargo test` is expected to run
  longer than a single conversational turn. The pevent record survives
  Claude session boundaries; later turns retrieve the verdict via
  `pevent wait`/`pevent log` rather than re-running the suite.
- **`ctrace`** is already enabled at the session level (SessionStart
  hook). Stage 5 postmortem should grep `~/.cache/ctrace/sessions/` for
  the iteration's ndjson and aggregate openat/execve counts as a
  ground-truth complement to the harness's self-reported metrics.

## Layout

```
~/.claude/skills/rustbuild/
├── SKILL.md                              ← this file
├── prompts/
│   ├── prd-intake-5whys.md
│   ├── reviewer-agent.md
│   ├── edit-agent.md
│   ├── postmortem-writer.md
│   └── evolve.md
├── templates/
│   ├── scaffold/                         ← project skeleton
│   └── AUTOBUILDER_PROGRAM.md.tmpl
├── rules/
│   ├── bad-rust.md                       ← curated subset lifted from jankurai
│   ├── hlt-rules.toml                    ← HLT-* IDs we adopt
│   └── audit-checks.sh                   ← grep + clippy-restriction implementations
├── schemas/
│   ├── intent-card.schema.json
│   ├── evidence-pack.schema.json
│   ├── failure-capsule.schema.json
│   ├── proof-receipt.schema.json
│   ├── mutants-receipt.schema.json
│   ├── spec-drift.schema.json            ← Stage 2.5 receipt (spec-drift.v1)
│   └── merge-witness.schema.json
├── scripts/
│   ├── intake.sh
│   ├── scaffold.sh
│   ├── spec_drift_probe.py               ← Stage 2.5 probe (CLI: spec-drift-probe.py symlink)
│   ├── experiment-loop.sh
│   ├── metric-harness.sh
│   ├── run-mutants.sh
│   ├── risk-gate.sh
│   ├── postmortem.sh
│   └── evolve.sh
├── tests/
│   └── test_spec_drift_probe.py          ← AC1-AC4/AC8 fixtures for the probe
└── proposals/                            ← accumulated evolution proposals (gated)
```

## Reference repos (vendored in this repo as read-only context)

The autobuilder repo ships three sibling research trees as upstream
reference material. They live alongside the skill in any checkout of
`j0yen/autobuilder`:

- `autoresearch-macos/` — locked-harness loop model
- `jankurai/` — anti-pattern catalog, HLT rule IDs, proof-lane format
- `jeryu/` — 7-receipt gate, EvidencePack, FailureCapsule, cargo-witness/vrc/aer crates

When invoked outside a repo checkout (skill installed via curl|bash
sparse-clone), these dirs are absent. The skill still functions for
Stages 1-2; Stages 3-5 expect the companion binary, whose own crates
embed the parts of these repos it depends on.

## Status

**Operational — Phases A–C complete.** The schemas, rules, and prompts are
in place (Phase A); the companion Rust binary is built and installed at
`~/.cargo/bin/autobuilder`, exposing all pipeline subcommands —
`intake / scaffold / loop / metric-harness / adversarial / experiment /
gate / rollback-plan / reviewer-agent / vti-plan / ci-checks / postmortem /
evolve` (Phase B); and the metric-harness meta-PRD plus many external PRDs
have run through the full loop (Phase C). The pipeline is load-bearing in
the daily `/build` automation, which delegates every Rust-shaped PRD to it;
EvidencePack receipts and postmortems have accumulated across shipped
projects. Scope remains `--target cli|lib` — servers included; WASM/embedded are v2 —
and Stage 6 publish is still a manual convention (see that stage).

## Resolved block — recall-memory-linter (historical)

An early run had the reviewer-agent return `block` on acceptance criteria
1 and 7; `evolve` recorded it here as a known anti-pattern. The slice was
subsequently resolved and shipped to `github.com/j0yen/recall-memory-linter`.
Retained as a worked example of the reviewer receipt doing its job — not as
pending work.
