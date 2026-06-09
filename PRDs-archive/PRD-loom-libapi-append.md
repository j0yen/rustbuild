# PRD-loom-libapi-append

Status: Draft v0.1

build_target: self-mod
build_priority: medium
build_into: /home/jsy/.claude/skills/build
Vision: visions/loom.md

## TL;DR

The sibling PRD `build-shared-cli-dispatch-merge-safe` makes parallel branches
register CLI subcommands append-only so `main.rs` stops conflicting. But the
2026-06-05 `anchor-probe`-vs-`anchor-reconcile` deferral conflicted on **both
`main.rs` AND `lib.rs`** — the library surface (`pub mod` / `pub use`
re-exports) has the exact same hand-edited-shared-hunk problem and no
convention. This PRD adds the `lib.rs` analogue: an append-only re-export region
and a `lib-register.sh` helper, so two branches adding distinct modules/types to
a shared library never touch the same lines.

## Why this exists

- journal 2026-06-05: "`anchor-probe`: … integrate blocked by merge conflict
  with anchor-reconcile (**both touched lib.rs+main.rs same tick**) — branch
  kept for next tick rebase." The dispatch-merge-safe PRD covers `main.rs`; the
  `lib.rs` half is uncovered.
- gossip 2026-06-06T06:39:07Z lists "lib.rs pub API" alongside "Cargo.lock,
  main.rs subcommand enum" as the recurring same-line collision surfaces.
- Pattern: a new `anchor <sub>` / `quicken <sub>` subcommand adds a new module
  and re-exports its public types from `lib.rs` (`pub mod probe;` + `pub use
  probe::{ProbeReport, Severity};`). Two branches insert these into the same
  alphabetically-sorted block → conflict.
- `build-shared-cli-dispatch-merge-safe` already establishes the
  anchored-end-of-list-append pattern (`// @build:subcommands`) and a
  `cli-register.sh`; this PRD is the deliberate, evidence-backed extension of
  that exact pattern to the library surface (it does not redefine it).

## What this builds

1. **`scripts/lib-register.sh <repo> <module> [<reexport-line>...]`** — appends,
   idempotently, at a unique `// @build:modules` anchor placed at the **end** of
   the module-declaration region of `lib.rs`:
   - `pub mod <module>;`
   - any provided `pub use <module>::{…};` re-export lines, each appended at a
     companion `// @build:reexports` end-of-list anchor.
   Re-running with the same args is a no-op (grep-guard before append). Two
   different modules appended by two branches produce **non-overlapping** diffs
   at the end of their respective regions, which git auto-merges.
2. **Anchor seeding.** On first use in a repo lacking the anchors,
   `lib-register.sh` inserts the two anchor comments at the end of the existing
   `pub mod` block and after the existing `pub use` block (a one-time, in-place,
   anchor-based edit — the same idempotent style as
   `wintermute-kernel/pkg/apply-agentns.py`, not a brittle line-number patch).
3. **Branch-prompt wiring.** The /build SKILL's shared-rust-extend branch prompt
   instructs agents whose target is a recurring shared **library** repo to wire
   new public surface via `lib-register.sh` instead of free-form editing
   `lib.rs`, mirroring the existing cli-register.sh instruction.

Constraints / shape:
- sigpipe-safe / pure bash; absolute paths; exits 0/1/2 like the sibling helper.
- Opt-in per repo: a repo that never adopts the anchors integrates exactly as
  today (back-compat — the helper is additive, the convention is documented but
  not forced on legacy `lib.rs` layouts).
- Sorted-region tolerance: append at end-of-list (not sorted-insert) so two
  concurrent appends never collide; a later cosmetic `cargo fmt`/sort is fine
  because it runs post-merge, single-threaded.
- Composes with `loom-rebase-retry`: if a `lib.rs` collision still happens
  (legacy repo), rebase-retry is the safety net; this PRD removes the collision
  at the source for adopting repos.

## Acceptance criteria

1. `scripts/lib-register.sh` exists, is executable, and appends `pub mod <m>;`
   plus any `pub use` lines at the `// @build:modules` / `// @build:reexports`
   end-of-list anchors; re-running with identical args is a verified no-op.
2. In a repo lacking the anchors, the first `lib-register.sh` call seeds both
   anchors in-place (idempotent, anchor-based — second run does not re-seed) and
   leaves `lib.rs` compiling.
3. Regression test: two simulated branches each register a **distinct** module +
   re-exports into a test repo's `lib.rs`; the resulting diffs `git merge`
   **auto-resolve with zero conflicts**, and the merged `lib.rs` compiles and
   exposes both modules' public types.
4. A malformed/legacy `lib.rs` with no recognizable `pub mod` block causes
   `lib-register.sh` to exit non-zero with a clear message rather than corrupting
   the file (no silent partial edit).
5. The /build SKILL "Worktree isolation" section documents `lib-register.sh`
   alongside `cli-register.sh` and instructs shared-library branch prompts to use
   it for new public surface.
6. Back-compat: a shared library repo that does not adopt the anchors integrates
   exactly as today; the helper and convention are opt-in per repo.
