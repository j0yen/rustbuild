# PRD-fluid-stt-partial-transcribe

**Status:** Draft v0.1
**Vision:** visions/fluid-voice.md
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-stt

## TL;DR

`current_partial()` returns `None` — a deliberate iter-7 stub. Implement
mid-stream partial transcription using whisper.cpp's encode/decode split so
`wm.stt.partial` events fire during speech and the brain can pre-warm recall
before `speech.end` arrives.

## Why this exists

`wintermute-stt/src/whisper_engine.rs:current_partial()`:

```rust
fn current_partial(&mut self) -> Option<String> {
    // iter-6: no mid-stream partial decoding — whisper.cpp's
    // streaming hooks (`encode` / `decode` split) land in iter-7
    // alongside `spawn_blocking` so the partial path doesn't
    // serialise behind the final inference.
    None
}
```

This stub means `processor.rs` never calls `publish_partial()` mid-utterance.
`wm.stt.partial` is defined in the bus event schema (`bus.rs`) but never
emitted. The brain receives the full transcript only after `speech.end`, adding
~200–500 ms before it can even begin recall lookup.

With partials flowing, the brain can start a speculative recall query on the
first partial and have results ready when the final transcript arrives —
collapsing the recall-lookup latency out of the perceived response time.

**Depends on:** PRD-fluid-stt-warm-state (warm state must be stable before
partial inference is added on top of it — both touch `WhisperEngine::finalise`).

## What this builds

Extend `~/wintermute/wintermute-stt` (`whisper_engine.rs`, `processor.rs`):

**`whisper_engine.rs`** — implement `current_partial()`:
- Add `partial_buffer: Vec<f32>` to the struct — a rolling window of the most
  recent 3 s of audio kept in sync with `self.buffer`.
- `current_partial()`: if `partial_buffer.len() >= WHISPER_MIN_SAMPLES`:
  - Pad to min (re-use the padding logic from PRD-fluid-stt-audio-padding).
  - Call `state.full(params, &partial_buffer)` in a `tokio::task::spawn_blocking`
    context (non-blocking — caller must be async or partial is skipped if busy).
  - Return the segment text, or `None` if inference is still running.
- `accept_chunk()` — also append to `partial_buffer`; drop samples older than 3 s
  to bound memory.
- `reset()` — clear both `buffer` and `partial_buffer`.

**`processor.rs`** — call `current_partial()` on a cadence:
- After each `speech.chunk` event processed, if `engine.current_partial()` returns
  `Some(text)`, publish `wm.stt.partial { text, ts, turn_id }` (existing bus event).
- Throttle to at most once per 500 ms (don't spam partials on every chunk).

## Acceptance criteria

1. **AC1 — partial events fire mid-utterance.** Integration test: send 2 s of
   PCM via `speech.chunk` events, then `speech.end`. Assert at least one
   `wm.stt.partial` event is published before `wm.stt.final`.
2. **AC2 — partial does not block final.** Integration test: time from
   `speech.end` received to `wm.stt.final` published must be ≤ 10% slower
   than without partials (i.e. partial inference must not serialise the final
   inference — spawn_blocking isolation).
3. **AC3 — partial text is a prefix of final.** Integration test with a known
   fixture: the last `wm.stt.partial` text must share a common prefix of at
   least 5 tokens with `wm.stt.final` text (partials are a rolling window, not
   guaranteed to match, but should not be completely disjoint for typical speech).
4. **AC4 — throttle respected.** Unit test: call `current_partial()` 10 times
   within 200 ms; assert `wm.stt.partial` published at most once (500 ms
   throttle).
5. **AC5 — partial buffer bounded.** Unit test: push 10 s of PCM via
   `accept_chunk`; assert `partial_buffer.len() <= 3 * 16_000` (3 s cap).
6. **AC6 — cargo test green.** `cargo test --features whisper` on
   `wintermute-stt` passes with no regressions.
