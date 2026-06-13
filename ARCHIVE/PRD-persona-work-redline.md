# PRD: persona-work-redline — the work persona's scope rules become a runtime gate

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/persona-work
Vision: visions/persona.md

## TL;DR

`~/wintermute/persona-work/CLAUDE_WORK.md` states hard scope rules for Joe's
AtScale laptop — never commit to `j0yen`, no auto-publish, no autonomous repo
creation, no voice/agorabus/family features, no `/build` or `/dream`, no force
push. Today those rules reach the agent only as prose it may or may not honor.
This PRD makes them a runtime gate: a work-scope `answerable` redline policy
plus a Claude Code `PreToolUse` hook that classifies a pending action and
blocks personal-scope actions on the work box. It is the action-side analogue
of `persona-redline`, which turned the elder persona's `forbidden_terms` from
prompt advice into an enforced guarantee.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The rules are real and concrete but unenforced.** Read live on this box,
  `~/wintermute/persona-work/CLAUDE_WORK.md` lists enforceable boundaries:
  "Never commit to j0yen personal repos", "No cross-publishing between
  joeyen-atscale and j0yen", "No auto-publish. No autonomous GitHub repo
  creation", "No voice features. No agorabus. No family reach", "Do not run
  /build or /dream skills", "No force push". `persona-work` (commit 558f961)
  ships the file + `install.sh` + `validate.sh` and nothing that enforces a
  single one of these at action time.
- **This is the exact gap persona-redline closed for the elder.** The vision's
  own framing: `forbidden_terms` was prompt-only advice (`wintermute-brain
  src/lib.rs:237`) until `persona-redline` scanned the generated reply and
  enforced it. The work persona's scope rules are at that same pre-enforcement
  rung. A stated boundary the running system never holds is the shared failure
  class across both personas.
- **The enforcement primitive already shipped and was validated today.**
  `answerable check --action <X> --attr k=v --policy redline.toml`
  (answerable v0.5.0; `answerable-redline` shipped 2026-06-12) returns exit
  0=allow / 1=flag / 2=redline against a declarative policy. The work guard
  does not need new enforcement logic — it needs a work-scope policy and a hook
  that feeds pending actions into `answerable check`.
- **`PreToolUse` hooks are the harness's action-gate surface.** Claude Code
  runs `PreToolUse` hooks before a tool executes and honors a non-zero/deny
  result to block the call — the correct place to intercept a `git push` to a
  `j0yen` remote or a `gh repo create` before it happens, not after.

## What this builds

Extends `~/wintermute/persona-work/` (published as `j0yen/persona-work`):

- **`redline-work.toml`** — a declarative `answerable` redline policy encoding
  CLAUDE_WORK.md's hard rules as `--action`/`--attr` matches, e.g.:
  - `action=git-push attr remote~=j0yen` → redline
  - `action=git-push attr force=true` → redline
  - `action=repo-create` → redline
  - `action=publish attr scope=personal` → redline
  - `action=daemon-launch attr kind=voice|agorabus|family` → redline
  - `action=skill attr name=build|dream` → redline
  Anything not matched → allow. The policy file is the single source of the
  rules; the hook is a thin classifier.
- **`hooks/pretooluse-work-guard.sh`** — a `PreToolUse` hook (reads the tool
  call JSON on stdin per the harness hook contract) that classifies the pending
  action into one of the policy's `--action`/`--attr` shapes (e.g. a `Bash`
  call whose command is `git push … <j0yen-url>` → `--action git-push --attr
  remote=<host/owner>`), calls `answerable check --policy <redline-work.toml>`,
  and emits a deny decision when the verdict is redline (exit 2). flag (exit 1)
  is surfaced as a warning but does not block. Non-classifiable calls pass
  through untouched. The hook is a no-op (exit 0, allow-all) unless the work
  identity is the installed one (`persona-work/validate.sh` reports `work`), so
  installing the repo on the personal box is harmless.
- **`install-guard.sh`** — installs `redline-work.toml` to
  `$XDG_CONFIG_HOME/answerable/redline-work.toml` and registers the hook in the
  work machine's `~/.claude/settings.json` `PreToolUse` array (idempotent,
  backs up settings, reversible via an `uninstall-guard.sh`). Never edits
  settings without the work identity installed.

### Acceptance criteria

1. `redline-work.toml` parses as a valid `answerable` policy
   (`answerable check --policy redline-work.toml --action noop` exits 0, not 3
   malformed) and is committed in the repo.
2. For each hard rule in CLAUDE_WORK.md there is a corresponding redline rule:
   a test drives `answerable check` with the matching `--action`/`--attr` and
   asserts exit 2 (redline) for the forbidden case AND exit 0 (allow) for a
   benign sibling (e.g. `git-push remote=joeyen-atscale` → allow;
   `git-push remote=j0yen` → redline). At least the six rules above covered.
3. `hooks/pretooluse-work-guard.sh` given a fixture `git push … <j0yen remote>`
   tool-call JSON on stdin emits a block/deny decision; given a
   `joeyen-atscale` push it allows. Verified with fixture JSON in `tests/`.
4. The hook is allow-all (exit 0, no deny) when `validate.sh` reports the
   installed identity is NOT `work` — proven by a fixture run on the personal
   box. The guard never blocks personal-box actions.
5. `install-guard.sh` is idempotent: running it twice yields one hook entry in
   `settings.json`; `uninstall-guard.sh` restores the pre-install settings.
   Both refuse to touch `settings.json` unless the work identity is installed.
6. The classifier covers `git push`, `gh repo create`/repo-create,
   force-push (`--force`/`+`refspec), voice/agorabus/family daemon launch, and
   `/build`·`/dream` skill invocation; a non-classifiable Bash call (e.g.
   `ls`) passes through with an allow verdict and no answerable call.
7. README documents the action→attr classification table and the
   install/uninstall/validate flow.

## Out of scope

- The eval number (block rate on a held-out set) — that is persona-work-eval.
- The drift watch — that is persona-work-doctor.
- Any change to `answerable` itself; this PRD only authors a policy + hook and
  consumes the shipped `answerable check`.
