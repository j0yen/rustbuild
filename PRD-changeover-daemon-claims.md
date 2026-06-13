# PRD: changeover-daemon-claims

Status: Draft v0.1
build_target: mixed
build_into: /home/jsy/wintermute/wintermute-audio
Vision: visions/changeover.md
Depends: PRD-changeover-claim-guard.md

## TL;DR

The problem: warm-swap restart cannot work until the daemons themselves
hold an agorabus claim. None of the four voice daemons currently acquire
one, so the successor process never becomes a claim holder and warm-swap
falls back to a hard restart — the deafness window the whole changeover
vision exists to close. This PRD wires each of `wm-audio`, `wm-dialog`,
`wm-stt`, and `wm-tts` to hold `agorabus://daemon/<unit>` for its
lifetime using the `ClaimGuard` from `changeover-claim-guard`, and to
release it on graceful shutdown. After this lands, a freshly started
successor announces itself as the sole claim holder, which is exactly the
signal `rollout`'s `wait_for_claim_holders` polls for.

## Why this exists

Phase-1 research, 2026-06-13:

- `grep -rl "claim_acquire\|ClaimAcquire" ~/wintermute/wintermute-{audio,stt,tts}/src`
  → **0 files**. `wintermute-dialog`'s only two `claim` hits
  (`src/family.rs:294`, `src/distress.rs:727`) are FSM-internal
  transcript-claiming, not the agorabus lease. So **no** voice daemon
  acquires a bus claim.
- `~/wintermute/rollout/src/warmswap.rs` (v0.6.0) starts the successor,
  then `wait_for_claim_holders(claim_path, …, 1, …)` before stopping the
  predecessor. With zero claim participants this poll always times out →
  warm-swap aborts to hard restart → the deafness window stays open.
- All four crates already depend on `agorabus = { path = "../agorabus" }`
  (`Cargo.toml`), so they can call `client.hold_claim(...)` from
  `changeover-claim-guard` with no new dependency.
- Live units restart these via systemd
  (`ExecStart=%h/.local/bin/wm-stt start`, etc.), so each daemon's
  graceful-shutdown path runs on `SIGTERM` from `systemctl --user stop` —
  the right place to release the claim before exit.
- Journal 2026-06-13 and every self-review 2026-06-08→13 park
  fleet-binary-staleness on "daemon restarts drop subscribers." This PRD
  removes the root cause of that block.

## What this builds

In each of `wintermute-{audio,dialog,stt,tts}`:

- After the daemon connects to agorabus at startup, call
  `client.hold_claim("agorabus://daemon/<unit>", ttl)` (unit =
  `wm-audio`/`wm-dialog`/`wm-stt`/`wm-tts`) and keep the returned
  `ClaimGuard` alive for the process lifetime.
- On graceful shutdown (existing SIGTERM/stop path), call
  `ClaimGuard::release(...)` (awaited) before exit so the predecessor's
  claim is gone the instant the successor can take it.
- The claim path convention `agorabus://daemon/<unit>` must match what
  `rollout` derives for warm-swap (derive-with-override, per the vision's
  Fleet-1 open question). Document the exact string each daemon uses.
- If the bus is unavailable at startup, the daemon proceeds as today
  (claim is best-effort, never blocks boot) and the guard's renew loop
  picks the claim up when the bus returns.

The change is the same small pattern in four sibling crates; /build may
fan out one branch per crate. `build_into` names `wintermute-audio` as
the canonical first; the other three are identical mirrors.

Out of scope: the claim-guard primitive itself (its own PRD), any rollout
change, mic/ALSA device handoff (tracked as a vision open question).

## Acceptance criteria

1. Each of the four daemons, on startup after bus connect, calls
   `hold_claim("agorabus://daemon/<unit>", …)` and retains the guard for
   its lifetime; all four crates build clean on MSRV 1.85.
2. With a daemon running, `agorabus claim list agorabus://daemon/<unit>`
   reports exactly one holder for that daemon's path (verifiable live for
   at least one daemon, or via an integration test with a stub bus).
3. On `systemctl --user stop <unit>` (graceful SIGTERM), the daemon
   releases its claim before exit; `claim list` then reports zero holders
   for that path.
4. Claim acquisition is best-effort: a daemon started with the bus down
   still reaches its normal running state and acquires the claim once the
   bus is reachable (test or documented manual verification).
5. The claim path string each daemon uses is documented and matches the
   path `rollout` warm-swap derives for that unit (no warm-swap fallback
   to hard restart on account of a path mismatch).
6. `cargo test` green for each touched crate; no new clippy warnings
   beyond the documented red baseline ([[self_recall_baseline_gate_red]]).
7. Each touched crate's CHANGELOG notes the new claim participation and
   the claim path it holds.
