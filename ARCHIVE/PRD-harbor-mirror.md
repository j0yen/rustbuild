# PRD: harbor-mirror — in-DC git mirror + a fallback remote for unpushed WIP

Status: Draft v0.1
build_target: shell
build_into: /home/jsy/wintermute/constellation-burst-builder
Vision: visions/harbor.md

## TL;DR

Burst pods clone the crate-under-build from GitHub over the public internet, and
the laptop carries work that never reaches GitHub. This PRD stands up a **bare git
mirror on the permanent hub**: build pods clone in-datacenter (fast), and the
laptop gets a **fallback push remote** for the WIP that self-review keeps flagging
as unpushed. The hub becomes the fleet's local git origin.

## Why this exists

- Self-review, run after run (latest 2026-06-12): **"24 dirty repos, rollout +
  wintermute-desktop unpushed."** That work has no home but the laptop's disk —
  one reboot-gone-wrong from loss. A hub-side mirror is a cheap safety net.
- Burst pods today clone from `github.com` across the public internet on every
  build; an in-DC bare mirror on the same Hetzner network is markedly faster and
  removes a GitHub-availability dependency from the build path.
- The hub is already permanent (harbor-hub) and already holds the warm cache
  (harbor-cache) — adding a git mirror to the same box is near-zero marginal cost.

## What this builds

**`scripts/harbor-mirror-up.sh`** — against the hub:
- Create a `git` user + `~/mirror/` holding bare repos; enable `git-daemon` (read)
  on the private interface and ssh push for the `jsy` key.
- Seed mirrors for a configurable repo list (default: the repos under
  `~/wintermute/` with a remote + the two named unpushed ones); idempotent — an
  existing bare repo is `fetch`ed, not re-created.
- Print the clone/push URL form (`hub:mirror/<repo>.git`).

**`scripts/harbor-mirror-sync.sh`** — run on the laptop: for each repo in the list,
add (idempotently) a `hub` remote pointing at the bare mirror and `push --mirror`
to it. This is the "back up my WIP" command — explicitly NOT a GitHub push, so it's
safe for work that isn't ready to be public.

**`scripts/harbor-mirror-check.sh`** — offline gate: asserts the up script is
idempotent (re-seed = fetch, not clone), the sync script adds the `hub` remote
without clobbering `origin`, and the URL form is derived from `hub.json`.

## Acceptance criteria

1. `harbor-mirror-up.sh --dry-run` lists the planned bare-repo seeds + git-daemon
   unit and exits 0 without touching the hub.
2. Idempotency: a second `--dry-run` against a "mirror-exists" fixture reports
   "fetch existing, no re-create" for each seeded repo (assert via marker).
3. `harbor-mirror-sync.sh --dry-run` adds a `hub` remote for each repo **without
   touching `origin`** (assert: planned `git remote add hub …`, never
   `set-url origin`).
4. The clone/push URL is derived from `hub.json` (harbor-hub state); with no
   `hub.json`, both scripts exit non-zero with "no hub — run `wm-burst hub up`".
5. The default repo list includes `rollout` and `wintermute-desktop` (the named
   unpushed repos from self-review) and is overridable via an arg/file.
6. `harbor-mirror-check.sh` passes as the offline acceptance gate and is invocable
   from the `scripts/` runner.
7. The sync script refuses to mirror a repo whose only difference from `origin` is
   uncommitted (it pushes committed refs, not the working tree) — documented, and
   the dry-run notes any repo with uncommitted changes so jsy commits first.
8. README gains a "git mirror" section: the in-DC clone benefit + the WIP-backup
   flow + the explicit "this is not a GitHub push" caveat.
