# PRD-constellation-wm-daemons-ryzen

**Status:** Shipped v1.0 — wm-brain, wm-tts, wm-dialog deployed via initial PRD; wm-audio + wm-stt completed via constellation-voice-boot-ryzen (native build on ryzen7, GLIBC 2.39, target-cpu=native/no-AVX512). All 5 ACs green.
**Vision:** visions/constellation.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-constellation-bus-ryzen (agorabus fleet bus live on ryzen7)

## TL;DR

ryzen7 (Ryzen 7 5825U, 28GB, AMD Radeon Barcelo, SSH-reachable as `ryzen7`)
has no wintermute daemons installed. This PRD deploys agorabus, wm-brain, wm-stt,
wm-tts, wm-audio, wm-dialog to ryzen7 and wires all systemd user units so
`wintermute.target` starts the full stack.

## Why this exists

**Evidence (2026-06-20, live SSH probes):**
- `ls ~/.local/bin/ | grep wm` on ryzen7 → empty (no wm-* binaries)
- `/etc/wintermute/` → does not exist on ryzen7
- ryzen7 has Docker + sudo available; user jsy uid=1000

The agorabus on ryzen7 is active but the wintermute voice stack is absent. Once
PRD-constellation-bus-ryzen bridges ryzen7 onto the fleet bus, the voice daemons
need to be there to handle turns routed to that node.

## What this builds

Runs on: **wintermute** (laptop), deploying to **ryzen7** via SSH.

The strategy: build all binaries on Hetzner cloudbuild (x86_64, same arch as ryzen7),
then rsync to ryzen7 + write service files and config.

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
already placed them) to ryzen7:
```
ssh ryzen7 mkdir -p ~/.local/bin
rsync -avz ~/.local/bin/wm-audio ~/.local/bin/wm-stt ~/.local/bin/wm-tts \
           ~/.local/bin/wm-dialog ~/.local/bin/wm-brain ~/.local/bin/agorabus \
           ryzen7:~/.local/bin/
```

### Step 3 — Config

Create `/etc/wintermute/conf.d/00-bootstrap.env` on ryzen7 (via sudo ssh):
```
WM_NODE=ryzen7
WM_ANTHROPIC_API_KEY=<inherited from wintermute — user must supply>
WM_BRAIN_SKIP_TIERS=local-8b
WM_BRAIN_DEFAULT_TIER=haiku
WM_STT_MODEL=small.en
WM_VAD_SILENCE_MS=300
```

### Step 4 — Systemd units

Deploy all wm-* unit files to ryzen7 (copy from `~/.config/systemd/user/wm-*.service`
on wintermute). Key units: `agorabus.service`, `wm-audio.service`, `wm-stt.service`,
`wm-tts.service`, `wm-dialog.service`, `wm-brain.service`, `wintermute.target`.

Enable the target:
```
ssh ryzen7 "systemctl --user daemon-reload && systemctl --user enable --now wintermute.target"
```

### Step 5 — Smoke test

Verify all daemons active on ryzen7:
```
ssh ryzen7 "systemctl --user status wintermute.target wm-audio wm-stt wm-tts wm-dialog wm-brain agorabus"
```

## Acceptance criteria

1. `ssh ryzen7 "ls ~/.local/bin/wm-{audio,stt,tts,dialog,brain}"` — all 5 binaries present.
2. `ssh ryzen7 "systemctl --user is-active wm-audio wm-stt wm-tts wm-dialog wm-brain"` — all active.
3. `/etc/wintermute/conf.d/00-bootstrap.env` exists on ryzen7 with `WM_NODE=ryzen7`.
4. `ssh ryzen7 "agorabus peers"` shows wm-audio, wm-stt, wm-tts, wm-dialog, wm-brain all announced on the local bus.
5. No ERRORs in `ssh ryzen7 "journalctl --user -u wm-brain -n 20"` (brain started and connected to API or local tier).
