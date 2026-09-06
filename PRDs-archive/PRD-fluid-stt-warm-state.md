# PRD-fluid-stt-warm-state

**Status:** Draft v0.1
**Vision:** visions/fluid-voice.md
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-stt

## TL;DR

Every voice turn currently rebuilds a 97 MB Whisper decode buffer from scratch.
Persist the `WhisperState` across turns so the 97 MB allocation happens once at
daemon start, not once per utterance.

## Why this exists

`journalctl --user -u wm-stt` shows `whisper_init_state: compute buffer (decode)
= 97.27 MB` on **every** utterance (observed 2026-06-20). The cause is in
`wintermute-stt/src/whisper_engine.rs:finalise()`:

```rust
let mut state = ctx
    .create_state()
    .map_err(|e| EngineError::Internal(format!("create_state: {e}")))?;
state.full(params, &samples)...
// `state` is a local — dropped at end of finalise()
```

`WhisperContext::create_state()` allocates the decode buffer each call. On
`small.en` that is ~97 MB. Dropping it at end-of-turn and rebuilding it next
turn adds ~500 ms–1 s of setup time to every utterance on this laptop's CPU.

The fix is to hoist `state` into the `WhisperEngine` struct and reuse it across
turns, resetting only the decode buffers (not re-allocating them) between turns.

## What this builds

Extend `~/wintermute/wintermute-stt` (`whisper_engine.rs`, `WhisperEngine` struct):

- Add `state: Mutex<Option<WhisperState<'static>>>` field (or use `Arc<Mutex<WhisperState>>`
  depending on lifetime constraints in whisper-rs — see AC4).
- `WhisperEngine::load()` — call `create_state()` once after loading the context;
  store in `self.state`.
- `finalise()` — lock `self.state`, call `state.full(params, &samples)` on the
  existing state rather than creating a new one.
- `reset()` — clear `self.buffer`; keep state alive (buffer clear is sufficient).
- `reload_model()` — replace both `self.ctx` and `self.state` (new model requires
  new state).
- Add `WhisperEngine::state_allocated(&self) -> bool` diagnostic helper (for AC2).

No changes to public trait surface (`TranscriptionEngine`), no changes to
`processor.rs`, `daemon.rs`, or bus events.

## Acceptance criteria

1. **AC1 — whisper_init_state fires once.** After starting `wm-stt` and completing
   three consecutive utterances, `journalctl -u wm-stt` contains exactly one
   occurrence of `whisper_init_state` (the startup allocation), not one per
   utterance.
2. **AC2 — state is allocated after load.** Unit test: construct a `WhisperEngine`
   (integration test with real model binary), assert `state_allocated() == true`
   before any `finalise()` call.
3. **AC3 — consecutive finalisations produce correct transcripts.** Integration
   test: call `accept_chunk` + `finalise` on a known PCM fixture twice in
   succession on the same `WhisperEngine` instance; both transcripts must be
   non-empty and match the fixture ground truth (or be within edit distance 3
   for the short fixture used in existing tests).
4. **AC4 — lifetime/Send safety.** `WhisperEngine` must satisfy `Send + Sync`
   (required by the trait bound in `daemon.rs`). If `WhisperState` is not
   `Send`, wrap in `Mutex<Option<Box<WhisperState<'_>>>>` with `unsafe impl Send`
   justified by the mutex (document why it is safe). The PR must not introduce
   any new `unsafe` without this comment.
5. **AC5 — reload_model replaces state.** Unit/integration test: load model A,
   call `finalise` once, call `reload_model` to model B, call `finalise` again.
   The second `finalise` must succeed and `whisper_init_state` must appear
   exactly twice total (once per model load).
6. **AC6 — cargo test green.** `cargo test --features whisper` on
   `wintermute-stt` passes with no regressions.
