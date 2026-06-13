# PRD: changeover-claim-guard

Status: Draft v0.1
build_target: rust-extend
build_into: /home/jsy/wintermute/agorabus
Vision: visions/changeover.md

## TL;DR

The problem: `rollout`'s warm-swap restart strategy (shipped v0.6.0) is
built entirely on the agorabus claim lease — it starts a successor
daemon, waits for it to appear as a claim holder, then stops the
predecessor. But no daemon actually acquires a claim, so the successor
never becomes a holder and warm-swap silently falls back to the hard
restart it was meant to replace. Before the daemons can participate, the
agorabus client needs an ergonomic, correct way to *hold* a claim for a
process's lifetime: acquire, auto-renew before the TTL expires, and
release cleanly on drop and on SIGTERM. This PRD adds that primitive —
`ClaimGuard` — to the agorabus client library. It is the shared building
block every voice daemon (and any future peer) will use; nothing in the
daemons themselves changes yet.

## Why this exists

Phase-1 research, 2026-06-13:

- `~/wintermute/rollout/src/warmswap.rs:131` waits for the successor to
  reach `>= 1` claim holders via `agorabus claim list <path>` before
  stopping the predecessor (`wait_for_claim_holders`,
  `CLAIM_POLL_TIMEOUT`). If no process ever acquires the claim, this poll
  times out and the swap aborts to hard restart.
- `~/wintermute/agorabus/src/client.rs:217` exposes `claim_acquire(...)`
  and the protocol carries `ClaimAcquire`/`ClaimRelease`
  (`src/protocol.rs:135`) broadcasting on `claim.acquire`/`claim.release`
  with a TTL lease. The acquire/release calls exist; there is no
  lifetime-bound holder that renews and auto-releases — so callers would
  have to hand-roll a renew loop and a Drop impl, which none have.
- Every voice daemon depends on this exact crate
  (`agorabus = { path = "../agorabus" }` in
  `wintermute-{audio,dialog,stt,tts}/Cargo.toml`), so a helper added here
  is immediately reusable by all four without a new dependency.
- [[self_agorabus_restart_kills_voice]]: a *crash* of these daemons once
  killed voice end-to-end. The claim lease, properly released on shutdown
  and re-acquired on start, is also what lets warm-swap guarantee exactly
  one live holder across a *planned* restart.

## What this builds

A new `ClaimGuard` type in the agorabus client (e.g.
`src/claim_guard.rs`, re-exported from the client module):

- `Client::hold_claim(&self, path: &str, ttl: Duration) -> Result<ClaimGuard>`
  — calls `claim_acquire` for `path`, then spawns/arms an auto-renew that
  re-acquires the lease at roughly `ttl / 3` before expiry. Returns a
  guard handle.
- `ClaimGuard` holds the claim path and renew task. On `Drop` it issues a
  best-effort `ClaimRelease` and cancels the renew task.
- A `ClaimGuard::release(self)` method for explicit, awaited release on a
  graceful shutdown path (so SIGTERM handlers can release before exit
  rather than relying on Drop ordering).
- Renew failures (bus unreachable) are logged and retried on the next
  tick, not fatal — a transient bus blip must not drop the claim
  permanently if the bus returns.

Shape:
- New module `src/claim_guard.rs`; public re-export so callers write
  `client.hold_claim(...)`.
- Reuse the existing `claim_acquire` request path; do not add new
  protocol messages.
- Async (tokio) consistent with the existing client; the renew task is a
  spawned future tied to the guard's lifetime.
- Deps: none new (tokio + the crate's existing time facilities).

Out of scope: daemon wiring (that is `changeover-daemon-claims`), any
rollout change, mic/device handoff.

## Acceptance criteria

1. `Client::hold_claim(path, ttl)` exists, compiles, and returns a
   `ClaimGuard`; the crate builds clean (`cargo build`) on MSRV 1.85 with
   no let-chains.
2. A unit/integration test against a stub or in-process bus shows that
   after `hold_claim`, a `claim list <path>` (or the client's holder
   query) reports exactly one holder for that path.
3. A test shows the guard re-acquires (renews) the lease at least once
   within a short TTL window (e.g. TTL=300ms, renew observed before
   expiry) without the holder count dropping to zero.
4. Dropping the `ClaimGuard` (and `ClaimGuard::release`) issues a
   `ClaimRelease`; a test confirms the holder count returns to zero after
   release.
5. A renew attempt against an unreachable bus logs and does not panic or
   permanently abandon the claim; on bus recovery the next renew
   succeeds (test may simulate with an injectable bus mock).
6. `cargo test` is green for the agorabus crate; `cargo clippy` introduces
   no new warnings beyond the documented red baseline
   ([[self_recall_baseline_gate_red]]).
7. README/CHANGELOG note the new `hold_claim`/`ClaimGuard` surface and a
   one-line usage example.
