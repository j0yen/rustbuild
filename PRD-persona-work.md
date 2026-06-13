# PRD: persona-work — Joe's work laptop is a different self

Status: Draft v0.1
build_target: shell
Vision: visions/persona.md

## TL;DR

`~/.claude/CLAUDE_SELF.md` defines a single identity for this machine —
wintermute: auto-publishing to personal repos, voice features, family reach,
sudo pre-approved, the whole personal-laptop posture. Joe's AtScale work
laptop is not wintermute and must not inherit that self. It needs a
**professional, narrower** identity: `joeyen-atscale` git scope, no
auto-publish to personal repos, no voice/family features, conservative
autonomy. This is the fourth and last named component of the persona vision,
and the only one still entirely unbuilt. `persona-work` ships a
`CLAUDE_WORK.md` self-file plus an idempotent install script that drops it as
`~/.claude/CLAUDE_SELF.md` on the work machine — two boxes, two selves, no
bleed.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **It is the last unshipped persona component.** Of the four bullets in
  `visions/persona.md` → Components, three are live in code this session:
  `persona-forbidden-vocab` (`wintermute-brain` v0.20.0),
  `persona-name-ceremony` (`src/introduction.rs` with
  `IntroductionMode::{FirstEverBoot,Explicit}` + `wm.persona.introduce` wired
  in `src/daemon.rs`), and `persona-consent-voice-ack` (`answerable` v0.5.0,
  `digest --speak --wait-ack`). `persona-work` has **no artifact** —
  `~/.claude/CLAUDE_WORK.md` does not exist (checked this session).
- **The current self is wired for the personal machine.** Live
  `~/.claude/CLAUDE_SELF.md` is a symlink into `~/dotfiles/.claude/` and
  encodes wintermute-specific posture: per-command `j0yen`/`Joe Yen` identities,
  auto-publish defaults, voice daemons, sudo pre-approved, local-toolkit
  reliance. Verbatim from the vision: *"Joe's AtScale machine is not
  wintermute… professional register, narrower autonomy, `joeyen-atscale`
  scope, no voice features, no family reach."* Running that self on a work box
  is a scope and confidentiality hazard.
- **A clean install path is required, not a hand-edit.** The personal self is
  a `~/dotfiles` symlink; the work machine needs its own file installed
  deliberately, with a backup of whatever was there, so the swap is reversible
  and auditable.

## What this builds

- **`CLAUDE_WORK.md`** — committed in the autobuilder repo under a
  `persona-work/` directory (source of truth; the work machine pulls/copies
  it). Mirrors `CLAUDE_SELF.md`'s seven-section shape so the lint contract is
  satisfied, but with work posture:
  - **Voice** — same terseness/no-emoji discipline, professional register.
  - **Values** — confidentiality-first; never auto-publish AtScale work to
    personal repos; flag anything that crosses the personal/work boundary.
  - **Defaults** — git identity `joeyen-atscale <jyen.tech@gmail.com>` only;
    no `j0yen` personal scope; **no auto-publish, no auto-push** without
    explicit ask; no voice daemons; no family/`wm.*` reach; tool/package
    conventions retained (`pnpm`/`cargo`/`uv`).
  - **Boundaries** — narrower autonomy than wintermute: confirm before any
    network-visible action; no irreversible ops without explicit confirmation;
    no reach into personal-machine services.
  - **Aspirations / Things I keep getting wrong / Changelog** — present and
    non-empty to satisfy the same lint contract `CLAUDE_SELF.md` uses
    (seven sections, ≤200 lines).
- **`install-work-persona.sh`** — idempotent installer:
  - Refuses to run unless an explicit `--i-am-the-work-machine` flag (or a
    `WM_WORK_MACHINE=1` env) is set, so it can never clobber the personal self
    by accident.
  - Backs up any existing `~/.claude/CLAUDE_SELF.md` (following the symlink)
    to `~/.claude/CLAUDE_SELF.md.personal.bak` before writing.
  - Installs `CLAUDE_WORK.md` as `~/.claude/CLAUDE_SELF.md` (copy, not symlink
    into personal dotfiles — the work machine must not depend on the personal
    dotfiles repo).
  - Re-running detects the work self is already installed (content hash match)
    and is a no-op.
  - `--uninstall` restores the backup if present.

This PRD ships files and a script only; no Rust, no daemon changes. It does
**not** attempt work-machine auto-detection or `chezmoi` integration — those
are deferred in the vision's open questions (3) and stay deferred.

## Acceptance criteria

1. `CLAUDE_WORK.md` exists, has the seven canonical sections
   (Voice/Values/Defaults/Things I keep getting wrong/Aspirations/Boundaries/
   Changelog), is ≤200 lines, and every section is non-empty.
2. `CLAUDE_WORK.md` contains **no** personal-scope leakage: grep finds no
   `j0yen` personal-repo auto-publish instruction, no `wm.*`/voice daemon
   reference, no "family"/Jocelyn reference; it **does** name
   `joeyen-atscale` as the git identity.
3. `install-work-persona.sh` without the work-machine flag/env exits non-zero
   and writes nothing (assert target `~/.claude/CLAUDE_SELF.md` unchanged).
4. With the flag against a temp `$HOME`, the script backs up a pre-existing
   `CLAUDE_SELF.md` to `CLAUDE_SELF.md.personal.bak`, then installs the work
   self; `diff` of installed file vs `CLAUDE_WORK.md` is empty.
5. Re-running the installer (same flag, work self already present) is a no-op:
   exit 0, no second backup created, target hash unchanged.
6. `--uninstall` restores `CLAUDE_SELF.md.personal.bak` over the target and
   exits 0; if no backup exists it exits non-zero without deleting the target.
7. `shellcheck install-work-persona.sh` passes with no errors (warnings
   documented if any), and the script runs under both `bash` and `zsh`.
8. The installer is idempotent and reversible end to end: install → uninstall
   → install leaves the target identical to a single install (hash compare).
