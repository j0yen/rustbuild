# PRD: plumb-independence — flag probes whose oracle is a tautology of the verdict

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/plumb
Vision: visions/plumb.md

## TL;DR

`plumb` pairs every self-review probe with an "independent" oracle and
trusts agreement between them as evidence that the probe reads true. But
nothing checks that the oracle is *actually independent*. The live
`~/.config/plumb/probes.toml` already contains a probe whose verdict and
oracle both `grep` the **same file** — agreement there proves nothing,
because the two readings can only fail together. This PRD adds
`plumb lint`: it extracts a coarse mechanism signature (tools invoked +
paths touched) from each probe's verdict and oracle and flags any probe
where the two share the same mechanism as `tautological`, so a fake
second opinion can't masquerade as calibration.

## Why this exists (Phase 1 evidence, 2026-06-13)

- The shipped `~/.config/plumb/probes.toml` `ctrace-backfill-wired` probe
  (read live this session) has:
  - `verdict = grep -q "scribe backfill" ~/.claude/hooks/session-end.sh`
  - `oracle  = grep -qE "scribe.backfill|backfill.*scribe" ~/.claude/hooks/session-end.sh`
  Both invoke `grep` against the **same file**. The config's own comment
  admits it: *"Independent oracle: use a slightly different grep pattern
  on the same file."* A different regex on the same file by the same tool
  is not an independent measurement — if the file is wrong, renamed, or
  missing, both sides return `absent` and "agree" falsely.
- This is [[feedback_agent_written_fixtures_tautology]] manifest inside
  the calibration layer built to prevent it: when the same agent writes
  both the rule and the thing that checks the rule via the same
  mechanism, agreement is circular. The recall feedback memory was
  recorded after a real incident (wm-router safety 100%→73.5% on a
  held-out set).
- By contrast `memlog-active` is genuinely independent: verdict uses
  `getent group` + a string gate; oracle uses `id -nG` + `stat -c %G`.
  Different tools, different surfaces. A good lint must pass this one and
  fail the ctrace one.
- plumb's own vision (`visions/plumb.md`) names oracle independence as
  the load-bearing assumption ("measures the same condition by a
  *different mechanism*") but ships no enforcement of it.

## What this builds

A new `lint` subcommand on the existing `plumb` binary
(`~/wintermute/plumb`, edition 2021, MSRV 1.85, clap-derive CLI over a
`Registry` loaded from `probes.toml`).

- New module `src/lint.rs`:
  - `fn mechanism_signature(cmd: &str) -> Mechanism` — a heuristic
    extractor. `Mechanism { tools: BTreeSet<String>, paths: BTreeSet<String> }`.
    - tools: first bare word of each pipeline segment (split on `|`, `&&`,
      `||`, `;`, command substitution boundaries), skipping shell
      builtins/keywords (`[`, `test`, `echo`, `then`, `else`, `fi`, `&&`
      noise) — keep recognizable executables (`grep`, `getent`, `id`,
      `stat`, `awk`, `jq`, `adopt`, …).
    - paths: tokens that look like paths (`/`-containing or `~`-prefixed),
      with `~` expanded to a normalized home placeholder so two spellings
      of the same file match.
  - `fn classify(verdict: &Mechanism, oracle: &Mechanism) -> Verdict`
    where `Verdict ∈ { Independent, Tautological { shared_tools, shared_paths } }`.
    Default rule (drafted): **tautological iff the primary tool sets
    overlap AND the path sets overlap** (the ctrace case: `{grep}` ∩
    `{grep}` non-empty and same file → flag; memlog: `{getent}` vs
    `{id,stat}` disjoint → independent).
- New CLI command `plumb lint`:
  - `plumb lint` / `plumb lint --all` — lint every registered probe.
  - `plumb lint <probe-id>` — lint one.
  - `--format json` — array of `{id, result, shared_tools, shared_paths}`;
    default human output lists each probe with `OK` / `TAUTOLOGICAL` and
    the shared mechanism.
  - `--strict` — widen the rule to flag on shared tools **OR** shared
    paths (per the vision open question); default is AND.
  - Exit nonzero (2) when any linted probe is `Tautological`; 0 when all
    independent; 1 on usage/load error. SIGPIPE-safe (reset at top of
    main, consistent with [[self_sigpipe_panic_toolkit]]).

Reuses the existing `Registry`/`ProbeEntry` from `src/registry.rs`; no
new deps. No change to `check`/`trust`/`list` behavior.

## Acceptance criteria

1. `plumb lint ctrace-backfill-wired` reports `TAUTOLOGICAL` and names
   the shared tool (`grep`) and shared path (the session-end hook file);
   the command exits nonzero.
2. `plumb lint memlog-active` reports `OK`/`Independent` and exits zero
   for that single-probe invocation (verdict tools and oracle tools are
   disjoint).
3. `plumb lint --all` over the live seed config exits nonzero (because at
   least one probe — ctrace — is tautological) and its human output lists
   every registered probe with a per-probe verdict.
4. `plumb lint --all --format json` emits valid JSON: an array with one
   object per probe, each having `id`, `result` (`independent` or
   `tautological`), and (when tautological) non-empty `shared_tools`
   and/or `shared_paths`.
5. `mechanism_signature` unit tests: a `grep X /a && echo y || echo n`
   verdict yields tools⊇`{grep}` and paths⊇`{/a}` and does NOT include
   shell noise (`echo`, `[`, `test`, `&&`); a `~/.claude/x`-spelled path
   and a `$HOME/.claude/x`-spelled path normalize to the same path token.
6. `--strict` makes a probe whose verdict and oracle share only a path
   (different tools) flag as tautological, where the default (AND) rule
   passes it. A unit or CLI test demonstrates the difference on a
   constructed probe entry.
7. `cargo test` green and `cargo build --release` clean on MSRV 1.85
   (edition 2021, no let-chains). `plumb lint --help` lists the
   subcommand and its flags.
