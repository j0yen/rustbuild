# PRD: vest-path — make the systemd user PATH resolve ~/.local/bin and ~/.cargo/bin

Status: Draft v0.1
build_target: config
Vision: visions/vest.md

## TL;DR

The systemd **user-manager** environment exposes
`PATH=/usr/local/bin:/usr/bin` — it does not include `~/.local/bin` or
`~/.cargo/bin`, the two directories where every wintermute CLI and the
Rust toolchain actually live. Interactive zsh shells get these dirs (via
`~/.zshenv`), but user *services* do not, so any `systemd --user` unit
that execs an adopted binary by name (not absolute path) silently fails
to find it. `adopt` only works around this because it probes the
convention dirs directly. `vest-path` adds these directories to the
user-manager PATH via `environment.d`, the canonical mechanism, so the
running system can actually reach what was vested into it.

## Why this exists (Phase 1 evidence, 2026-06-13)

- **The gap, measured.** `systemctl --user show-environment` →
  `PATH=/usr/local/bin:/usr/bin`. Neither `~/.local/bin` nor
  `~/.cargo/bin` is present.
- **The convention is `~/.local/bin`.** `CLAUDE_SELF.md` and recall
  both state local tools live under `~/.local/bin`; `adopt`'s own
  `local_bin()`/`cargo_bin()` (`scan.rs:33-40`) hardcode these as the
  laptop convention.
- **Services already trip on it.** The `adopt-cron.service` ExecStart
  uses absolute paths (`/home/jsy/.local/bin/adopt`) precisely because
  the bare name would not resolve — a workaround that every future unit
  would otherwise have to repeat.
- **`environment.d` is the right surface.** `~/.config/environment.d/`
  already exists and holds `wintermute.conf` (it currently sets
  `WM_ANTHROPIC_API_KEY`), so the mechanism is live and in use.

## What this builds

A single declarative `environment.d` drop-in. No code.

### The drop-in

`~/.config/environment.d/10-path.conf`:

```
PATH=%h/.local/bin:%h/.cargo/bin:/usr/local/bin:/usr/bin
```

- Uses systemd's `%h` specifier (the user's home) rather than a literal
  path, so it is host-portable across the constellation fleet.
- Prepends the two convention dirs ahead of the system dirs so adopted
  binaries shadow any stale system copies — matching interactive-shell
  precedence.
- A separate drop-in (not appended to `wintermute.conf`) so the secret
  and the PATH live in different files.

### Activation note

`environment.d` is read by the user manager at next login / `systemctl
--user import-environment` is **not** sufficient; the documented path is
`systemctl --user daemon-reexec` followed by restarting affected units,
or a re-login. The PRD's install step writes the file and documents the
activation command; it does **not** force a re-login.

## Acceptance criteria

1. `~/.config/environment.d/10-path.conf` exists and sets `PATH` to
   `%h/.local/bin:%h/.cargo/bin:/usr/local/bin:/usr/bin`.
2. The existing `wintermute.conf` is left unmodified (the API key drop-in
   is untouched).
3. After applying and reloading the user manager, `systemctl --user
   show-environment | grep PATH` includes `/home/jsy/.local/bin` and
   `/home/jsy/.cargo/bin`.
4. A user unit that execs a bare adopted binary name (e.g. `ExecStart=adopt
   scan`) resolves it after activation.
5. The build records the exact activation command in its receipt /
   install notes; it does not force a re-login or kill the user session.

## Out of scope

- Changing interactive shell PATH (already handled by `~/.zshenv`).
- Migrating the `WM_ANTHROPIC_API_KEY` secret out of `environment.d`
  into a secrets manager (separate security concern; see note to user).
- Any adopt code change (PRD-vest-root-guard / -verify / -incremental).
