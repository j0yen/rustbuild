# PRD: drydock-classify — route every drifted item to its safe remediation lane

**Status:** Draft v0.1
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/drydock-survey
**Vision:** visions/drydock.md
**Repo:** j0yen/drydock-survey (NEVER AtScaleInc)

## TL;DR

A drift inventory is only actionable once each item is sorted into the lane that
can *safely* fix it. A plain CLI rebuild is harmless; restarting wm-stt mid-turn
deafens the user; installing a kernel package needs a reboot; touching the
self-review apply-guardrail needs jsy's sign-off. drydock-classify is the
risk-tier router: it consumes drydock-survey's inventory and assigns every item a
lane — `auto` / `window` / `reboot` / `approval` — deny-by-default (anything it
cannot positively classify lands in `approval`, never `auto`), and emits the
exact one-line remediation command per item.

## Why this exists

Verified 2026-06-16 (Phase-1 ecosystem map):

- The fleet has detect (`binstale`), truthful verdicts (`scion`), plan
  (`rollout plan`), and an adopt-loop converger (`fixpoint`) — but **no vision
  proposes a risk-tier framework** that says "voice daemons → human-windowed;
  plain CLIs → auto; kernel → reboot." The Explore pass over ten
  rollout-ecosystem visions found this "Decide (selectivity)" layer entirely
  absent: every current path either parks *everything* (the immutable guardrail)
  or converges only the adopt-marker subset (fixpoint).
- The cost of the gap is concrete: self-review's "Pending your call" mixes a
  one-command plain-CLI install (`adopt apply`) in the same undifferentiated
  prose as a kernel `pacman -U` that wants a reboot and a wm-stt restart that
  wants a quiet window. The human supplies the routing by reading every line.
- The voice daemons are a known hard floor: `changeover.md` documents that
  restarting wm-audio/dialog/stt/tts mid-conversation drops subscribers, which is
  exactly why those must never be lane `auto`.

Classification is the missing decision layer. It is small, pure, and testable,
and it unblocks both the digest (group-by-lane) and the gated auto-drain.

## What this builds

Extends the `drydock-survey` crate (new `classify` module + `drydock-survey
classify` subcommand, or a sibling binary in the same crate — builder's choice;
the lane logic must be a library fn unit-tested independently of IO).

**Lane policy**
- A built-in default policy plus an optional override at
  `~/.config/drydock/lanes.toml` (merged over defaults).
- **Hard floors the policy cannot downgrade** (encoded in code, not config):
  - any item whose `item` is in the voice-daemon set
    (`wm-audio`, `wm-dialog`, `wm-stt`, `wm-tts`) ⇒ at most `window`;
  - `kind == "kernel-pkg"` ⇒ `reboot`;
  - any item that would require running `rollout apply` on a guarded daemon, or
    any `source-unavailable` sentinel, or any item not matched by a positive
    rule ⇒ `approval`.
- **`auto` is opt-in per rule and only reachable by** plain non-voice CLIs
  (`kind == "cli"`) and non-voice daemon `deleted-exe`/`behind-head` items whose
  remediation is an idempotent `rollout install` — i.e. no live-conversation
  service. Deny-by-default: an item reaches `auto` only by matching an explicit
  allow rule.

**Remediation command** — per item, emit the exact one-liner:
- cli ⇒ `adopt apply` (scoped to the item where adopt supports it);
- non-voice daemon ⇒ `rollout install <repo>` / `rollout apply` for that unit;
- voice daemon ⇒ `rollout install <repo> --window 30s`;
- kernel-pkg ⇒ `sudo pacman -U <staged-pkg-path>` (+ "reboot to activate");
- approval ⇒ a human-readable reason string, no command.

**Output** — pass through every survey field plus `lane` and `command`
(nullable). JSON array preserves survey order; a `--format table` groups by lane.

**Deps:** inherits survey's; add `toml`. No new network. Lane assignment is a
pure function `classify(item, policy) -> (Lane, Option<Command>)`.

**UX**
```
drydock-survey --json | drydock-classify --json
drydock-classify --json                 # runs survey internally
drydock-classify --explain wm-stt       # why this lane
```

## Acceptance criteria

1. `classify(item, policy)` is a pure library function with no IO, unit-tested
   across all four lanes.
2. Every voice-daemon item (`wm-audio|wm-dialog|wm-stt|wm-tts`) is assigned at
   most `window` — a config rule attempting to set it `auto` is ignored (hard
   floor verified by test).
3. Every `kind == "kernel-pkg"` item is assigned `reboot`; its command is the
   `pacman -U` of the staged pkg with a reboot note.
4. An item matched by no positive rule, and any `source-unavailable` sentinel,
   is assigned `approval` with a reason and a null command (deny-by-default
   verified).
5. A plain non-voice `cli` not-current item is assigned `auto` with an
   `adopt apply` command; a non-voice daemon `deleted-exe` is assigned `auto`
   with a `rollout install` command.
6. `~/.config/drydock/lanes.toml` overrides default rules where the override is
   *more* restrictive or assigns lanes within the allowed set; it can never
   downgrade a hard floor.
7. Output JSON preserves every survey field and adds `lane` and `command`
   (command nullable for `approval`).
8. `--explain <item>` prints the rule (or floor) that decided the item's lane.
9. `cargo test` green; `cargo clippy` clean on the crate's own code.
