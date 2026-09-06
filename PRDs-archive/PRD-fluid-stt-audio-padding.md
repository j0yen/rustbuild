# PRD-fluid-stt-audio-padding

**Status:** Draft v0.1
**Vision:** visions/fluid-voice.md
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-stt

## TL;DR

Whisper requires at least 1000 ms of audio. Short utterances (e.g. "yes", "stop",
"story") are silently dropped today. Zero-pad them to 1000 ms so they transcribe.

## Why this exists

`journalctl --user -u wm-stt` on 2026-06-20 shows repeated:

```
whisper_full_with_state: input is too short - 760 ms < 1000 ms
whisper_full_with_state: input is too short - 980 ms < 1000 ms
whisper_full_with_state: input is too short - 600 ms < 1000 ms
```

Every short command ("yes", "tell me a story", "stop") below the 1000 ms
Whisper floor is rejected. With VAD hangover at 300 ms (the current floor),
many real commands land under 1000 ms. The log lines appear with no
`wm.stt.final` or `wm.stt.error` event — the utterance is silently swallowed.

The fix is a padding shim in `WhisperEngine::finalise()`: if the accumulated
sample buffer is shorter than `WHISPER_MIN_SAMPLES` (1000 ms × 16 kHz = 16 000
samples), append zero-valued `f32` samples to reach the minimum before passing
to `state.full()`. Silence padding does not affect transcription quality for
real speech — Whisper is trained on padded windows.

## What this builds

Extend `~/wintermute/wintermute-stt` (`whisper_engine.rs`):

- Add constant `WHISPER_MIN_SAMPLES: usize = 16_000` (1000 ms × 16 kHz).
- In `finalise()`, after `let samples = std::mem::take(&mut self.buffer)`:
  ```rust
  let samples = if samples.len() < WHISPER_MIN_SAMPLES {
      let mut padded = samples;
      padded.resize(WHISPER_MIN_SAMPLES, 0.0_f32);
      padded
  } else {
      samples
  };
  ```
- Log a `tracing::debug!` when padding fires: `padded_samples`, `original_samples`,
  `padded_ms` — so the operator can see it happened without logspam at info level.
- No changes to the trait surface, processor, or bus events.

## Acceptance criteria

1. **AC1 — sub-1000 ms utterance transcribes.** Integration test: call
   `accept_chunk` with exactly 800 ms of synthetic PCM (silence + a single
   recognisable tone segment), then `finalise`. Must return `Ok(EngineFinal)`
   without the `whisper_full_with_state: input is too short` error. (Use the
   `WhisperEngine` directly with the real model binary.)
2. **AC2 — padding is zero-filled.** Unit test (no model needed): construct a
   mock/stub engine or extract the padding logic into a standalone function
   `pad_to_min(samples: Vec<f32>) -> Vec<f32>` and assert:
   - input length 8000 → output length 16000, last 8000 elements are `0.0`.
   - input length 16000 → output length 16000 (no-op).
   - input length 20000 → output length 20000 (no truncation).
3. **AC3 — debug log fires on pad.** Integration test: capture `tracing`
   output; verify `padded_samples` field appears in debug output when a
   sub-minimum buffer is passed.
4. **AC4 — full-length utterances unaffected.** Integration test: a 2000 ms
   PCM fixture transcribes identically before and after this change (no
   regression).
5. **AC5 — cargo test green.** `cargo test --features whisper` on
   `wintermute-stt` passes with no regressions.
