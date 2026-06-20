# PRD-constellation-voice-boot-ryzen

**Status:** Draft v0.1
**Vision:** visions/constellation.md
**build_target:** shell
**build_into:** /home/jsy/wintermute/constellation

**Depends on:** PRD-constellation-wm-daemons-ryzen (wm-* stack running on ryzen-work)

## TL;DR

With wintermute daemons running on ryzen-work, the final step is voice-on-boot:
greetd autologin → i3 → `wintermute.target` so speaking "wintermute" in the room
works from any machine in the fleet. This PRD also copies the desktop appearance
(gruvbox Alacritty, green-phosphor i3 borders, steel-blue wallpaper) so the
experience is visually identical to the laptop.

## Why this exists

**Evidence (2026-06-20, live SSH probes):**
- ryzen-work has i3 in the repo (`~/wintermute/constellation/`), but no display manager or autologin configured
- Voice on boot requires: display manager autologin → i3 → graphical-session.target → wintermute.target (via systemd user session)
- The laptop demonstrates this works: `wm-audio.service` starts via wintermute.target which WantedBy graphical-session.target
- ryzen-work has a Radeon Barcelo iGPU (gfx90c) — X11/i3 works fine without discrete GPU

Ryzen-work's users:98 score on swap (7.8G/8G) suggests it's been a headless/server machine.
This PRD adds the graphical session, designed to be lightweight (i3 + Alacritty, no compositor).

## What this builds

Runs on: **ryzen-work** (via SSH from wintermute).

### Step 1 — Install display + audio stack if missing

```
ssh ryzen-work "sudo pacman -Syu --noconfirm --needed greetd greetd-agreety i3-wm i3status alacritty xorg-server xorg-xinit pipewire pipewire-pulse wireplumber feh"
```

### Step 2 — greetd autologin config

`/etc/greetd/config.toml` on ryzen-work:
```toml
[terminal]
vt = 1

[default_session]
command = "agreety --cmd i3"
user = "jsy"

[initial_session]
command = "i3"
user = "jsy"
```

Enable and start:
```
ssh ryzen-work "sudo systemctl enable --now greetd"
```

### Step 3 — i3 config

Copy `~/.config/i3/config` from wintermute to ryzen-work (the green-phosphor theme,
gruvbox bar, steel-blue wallpaper, no picom, voice keybindings). Adjust:
- Remove YouTube autostart (that's laptop-specific)
- Keep `exec --no-startup-id feh --bg-fill ~/.config/wallpaper/steel-blue.ppm`
- Keep `exec --no-startup-id xrdb -merge ~/.Xresources`

Copy wallpaper:
```
rsync -avz ~/.config/wallpaper/steel-blue.ppm ryzen-work:~/.config/wallpaper/
```

### Step 4 — Alacritty config

Copy `~/.config/alacritty/alacritty.toml` from wintermute to ryzen-work (Gruvbox Dark,
padding=0, decorations=none — identical appearance).

### Step 5 — systemd graphical-session bridge

i3 needs to notify systemd that the graphical session is ready so user services start.
Create `~/.config/systemd/user/i3.service.d/notify.conf` on ryzen-work:
```ini
[Service]
ExecStartPost=/usr/bin/systemctl --user set-environment DISPLAY=:0
ExecStartPost=/usr/bin/systemctl --user start graphical-session.target
```

### Step 6 — Wire wintermute.target to graphical-session

The wintermute.target unit already has `WantedBy=graphical-session.target` (copied from
wintermute). Verify on ryzen-work after copy. `systemctl --user enable wintermute.target`.

### Step 7 — Smoke test (remote)

After rebooting ryzen-work:
```
ssh ryzen-work "systemctl --user is-active wm-audio wm-stt wm-brain"
```
All should be `active` within 30s of boot.

## Acceptance criteria

1. `ssh ryzen-work "systemctl is-active greetd"` returns `active`.
2. After ryzen-work reboot, `ssh ryzen-work "systemctl --user is-active wm-audio"` returns `active` (voice stack auto-started by wintermute.target).
3. `ssh ryzen-work "cat ~/.config/alacritty/alacritty.toml | grep gruvbox" -i` — Gruvbox colors present (identical appearance).
4. `ssh ryzen-work "ls ~/.config/wallpaper/steel-blue.ppm"` — wallpaper copied.
5. `ssh ryzen-work "systemctl --user is-active wm-brain"` returns `active` (brain connected to cloud API tier).
