# PRD-constellation-wm-daemons-ryzen

**Status:** Draft v0.1
**Vision:** visions/constellation.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-constellation-bus-ryzen (agorabus fleet bus live on ryzen-work)

## TL;DR

ryzen-work (Ryzen 7 5825U, 28GB, AMD Radeon Barcelo, SSH-reachable as `ryzen-work`)
has no wintermute daemons installed. This PRD deploys agorabus, wm-brain, wm-stt,
wm-tts, wm-audio, wm-dialog to ryzen-work and wires all systemd user units so
`wintermute.target` starts the full stack.

## Why this exists

**Evidence (2026-06-20, live SSH probes):**
- `ls ~/.local/bin/ | grep wm` on ryzen-work → empty (no wm-* binaries)
- `/etc/wintermute/` → does not exist on ryzen-work
- ryzen-work has Docker + sudo available; user jsy uid=1000

The agorabus on ryzen-work is active but the wintermute voice stack is absent. Once
PRD-constellation-bus-ryzen bridges ryzen-work onto the fleet bus, the voice daemons
need to be there to handle turns routed to that node.

## What this builds

Runs on: **wintermute** (laptop), deploying to **ryzen-work** via SSH.

The strategy: build all binaries on Hetzner cloudbuild (x86_64, same arch as ryzen-work),
then rsync to ryzen-work + write service files and config.

### Step 1 — Build binaries via cloudbuild

Build the full wintermute crate set:
```
bash ~/.claude/skills/cloudbuild/cloudbuild.sh build wintermute-audio -- --release
bash ~/.claude/skills/cloudbuild/cloudbuild.sh build wintermute-stt -- --features whisper --release
bash ~/.claude/skills/cloudbuild/cloudbuild.sh build wintermute-tts -- --release
bash ~/.claude/skills/cloudbuild/cloudbuild.sh build wintermute-dialog -- --release
bash ~/.claude/skills/cloudbuild/cloudbuild.sh build wintermute-brain -- --release
```

### Step 2 — Deploy binaries

Rsync built binaries from wintermute's `~/.local/bin/` (where extend-handler.sh install
already placed them) to ryzen-work:
```
ssh ryzen-work mkdir -p ~/.local/bin
rsync -avz ~/.local/bin/wm-audio ~/.local/bin/wm-stt ~/.local/bin/wm-tts \
           ~/.local/bin/wm-dialog ~/.local/bin/wm-brain ~/.local/bin/agorabus \
           ryzen-work:~/.local/bin/
```

### Step 3 — Config

Create `/etc/wintermute/conf.d/00-bootstrap.env` on ryzen-work (via sudo ssh):
```
WM_NODE=ryzen-work
WM_ANTHROPIC_API_KEY=<inherited from wintermute — user must supply>
WM_BRAIN_SKIP_TIERS=local-8b
WM_BRAIN_DEFAULT_TIER=haiku
WM_STT_MODEL=small.en
WM_VAD_SILENCE_MS=300
```

### Step 4 — Systemd units

Deploy all wm-* unit files to ryzen-work (copy from `~/.config/systemd/user/wm-*.service`
on wintermute). Key units: `agorabus.service`, `wm-audio.service`, `wm-stt.service`,
`wm-tts.service`, `wm-dialog.service`, `wm-brain.service`, `wintermute.target`.

Enable the target:
```
ssh ryzen-work "systemctl --user daemon-reload && systemctl --user enable --now wintermute.target"
```

### Step 5 — Smoke test

Verify all daemons active on ryzen-work:
```
ssh ryzen-work "systemctl --user status wintermute.target wm-audio wm-stt wm-tts wm-dialog wm-brain agorabus"
```

## Acceptance criteria

1. `ssh ryzen-work "ls ~/.local/bin/wm-{audio,stt,tts,dialog,brain}"` — all 5 binaries present.
2. `ssh ryzen-work "systemctl --user is-active wm-audio wm-stt wm-tts wm-dialog wm-brain"` — all active.
3. `/etc/wintermute/conf.d/00-bootstrap.env` exists on ryzen-work with `WM_NODE=ryzen-work`.
4. `ssh ryzen-work "agorabus peers"` shows wm-audio, wm-stt, wm-tts, wm-dialog, wm-brain all announced on the local bus.
5. No ERRORs in `ssh ryzen-work "journalctl --user -u wm-brain -n 20"` (brain started and connected to API or local tier).
