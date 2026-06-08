# PRD: recourse-feedback — the world's disagreement becomes the next version's wisdom

Status: Draft v0.1
build_priority: high
build_target: mixed
build_into: /home/jsy/wintermute/recourse
Vision: visions/recourse.md

## TL;DR

Receipts, contests, amended corpus cases, and a pulse are all inputs with no
consumer until something **acts** on them. This PRD is the cycle's closing edge:
`recourse feedback` cuts a new **versioned ontology changeset** from the upheld
contests that justify it, ties the version to the contests that drove it, and —
reviewer-gated — publishes it through `herald-market` so installed `/conscience`
instances upgrade. The world's disagreement, proven by `tribunal`, becomes the
next shipped version's wisdom. reason → prove → distribute → receive → amend →
**re-ship**.

## Why this exists

- The arc must become a **cycle**, not a one-way street (vision recourse §TL;DR).
  Every prior `recourse` PRD produces signal; `feedback` is the only one that turns
  signal into a shipped improvement.
- `herald` end-state #2 already owns the publish path (`herald-market` maintains a
  `j0yen` `marketplace.json`); `feedback` consumes it rather than reinventing it —
  cross-vision dep, stubbed for tests.
- **Churn control is a stated open question** (vision recourse §"Open questions"):
  re-versioning per contest would thrash the marketplace, so a version cut must be a
  deliberate, batched, reviewer action — encoded here, not left to chance.
- **Reviewer-gated, never auto-publish** is the fleet's hard rule
  (tribunal-gate false-allow==0; lattice no-silent-merge). A version that changes
  what `/conscience` decides on strangers' machines is the highest-stakes publish in
  the whole fleet; it cannot be automatic.

## What this builds

Adds the `feedback` subcommand to the `recourse` crate (mixed: Rust CLI + a
documented shell-out to `tribunal gate` and `herald-market`, both stubbed for
tests).

- **`recourse feedback propose [--since <dur>]`** — gathers the upheld contests
  amended into corpus cases since the last version, plus the `pulse` summary, and
  renders a **changeset proposal**:
  - `changeset.toml` — proposed new ontology semver, the list of `field-<contest-id>`
    corpus cases included, and the pulse deltas that motivate the bump.
  - `CHANGELOG-<version>.md` — human-readable: "vN.M ships because of contests
    A, B, C — these field cases the prior version got wrong."
  The proposal lands in a reviewer-gated dir; it publishes **nothing**.
- **`recourse feedback ship <version> --confirm`** — the only path that publishes.
  Pre-flight: runs `tribunal gate` against the amended corpus at the proposed
  version (refuses to ship if the gate fails — a regression on any held-out case,
  field or original, blocks the release). On pass, invokes `herald-market` to
  publish the new version. Both tribunal and herald are recorded stubs in tests.
- **`--upstream`** (vision open-question): on a third party's install, propose the
  change to `j0yen` upstream instead of forking the local corpus. v1 records the
  intent + target; the actual PR mechanics are deferred to a successor PRD (noted,
  not hand-waved into ACs).

**Deps:** shares the `recourse` lib; `serde`, `toml`, `time`, `clap`. SIGPIPE reset.
rustc 1.85, no let-chains.

## Acceptance criteria

1. `recourse feedback propose` over a fixture of upheld+amended contests renders a
   `changeset.toml` listing exactly those `field-<id>` cases and a proposed semver
   strictly greater than the prior version; plus a `CHANGELOG-<version>.md` naming
   each driving contest.
2. `propose` publishes nothing: after it runs, the `herald-market` stub records
   **zero** publish calls and no marketplace file changes (asserted).
3. `recourse feedback ship <version> --confirm` runs `tribunal gate` **before** any
   publish; a stub gate **failure** blocks the publish and exits non-zero (no
   `herald-market` call made).
4. On a passing gate, `ship --confirm` invokes the `herald-market` stub exactly once
   with the proposed version; without `--confirm` it never publishes (dry preview
   only).
5. **Batching/churn AC:** `propose` with zero new upheld contests since the last
   version produces no changeset and exits cleanly ("nothing to ship") — no empty
   version is ever cut.
6. `--upstream` records the upstream target in the changeset and does **not** mutate
   the local marketplace; the deferred PR mechanics are documented as out of scope
   in `--help`.
7. SIGPIPE-safe; `cargo test` green (against tribunal + herald stubs);
   `feedback --help` lists `propose` and `ship` and documents `--confirm`.
