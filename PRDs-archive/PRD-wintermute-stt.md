# PRD: wintermute-stt — speech to text

**Author:** /dream (Claude Opus 4.7), with jsy
**Status:** Draft v0.1
**Date:** 2026-05-24
**Vision:** `visions/wintermute.md`
**Builds on:** `PRD-wintermute-audio.md` (consumes `wm.audio.speech.*` events)
**Consumed by:** `PRD-wintermute-dialog.md` (which forwards finalized transcripts to the brain)
build_auto: true
build_target: rust-cli
build_priority: high
deferred_acs: [1, 4, 5, 6, 7, 8]
mock_unjustified_for: [1, 4, 5, 6, 7, 8]
mock_justifications:
  1: "AC1 measures warm whisper.cpp distil-small.en transcription latency on this laptop's real CPU; the inference time is hardware-dependent and a mock bypass of whisper-rs would assert nothing about actual inference speed."
  4: "AC4 requires a live network call to the Whisper API with real PCM and measures end-to-end round-trip latency; a mock HTTP stub validates only our wiring, not the actual cloud-path latency the AC demands."
  5: "AC5 tests that a mid-flight cloud request is abandoned gracefully on network drop and the local result emits without duplication; this involves real OS socket teardown behavior that cannot be faithfully reproduced by a stubbed reqwest client."
  6: "AC6 measures hot-swap of the whisper.cpp model (--reload-model small.en) completing in <5 s without dropping the mic.sock subscription; the timing bound is dominated by whisper.cpp's real model load time, which a mock cannot simulate."
  7: "AC7 verifies RSS growth <50 MB over a 60-minute real inference run; memory growth is a property of the live whisper-rs binding and allocator, not observable through any in-process fake."
  8: "AC8 confirms recovery from wm-audio restart by re-subscribing to mic.sock within 5 s; the recovery path exercises real Unix socket teardown and reconnect semantics that a loopback mock does not exercise in the same way."

---

## TL;DR

Speech chunks from `wm-audio` arrive as PCM; we turn them into text
using **whisper.cpp** via the `whisper-rs` Rust binding. Default
model is `distil-small.en` (the realistic CPU choice, per plan-agent
— `large-v3-turbo` is too optimistic on this laptop). Confidence
score is emitted with every final transcript; below threshold, the
brain asks for repeat. Optional cloud fast-path routes to the
Whisper API when network is OK and the user opted in during
bootstrap. Plan-agent cut Moonshine — the Rust bindings are
immature and the cloud fast-path achieves the same goal more reliably.

---

## 1. Why this exists

Three observations:

1. **whisper.cpp is the mature CPU choice.** Three years of
   production tooling, broad language support, well-understood
   memory/CPU profile. `whisper-rs` 0.13+ wraps it cleanly.

2. **distil-small.en hits the latency budget on CPU.** Plan-agent's
   correction: assuming `large-v3-turbo` warm transcription in 1.2 s
   on an older laptop is optimistic. `distil-small.en` is realistic
   at ≤2 s for a 5-second utterance and degrades gracefully on
   slower CPUs.

3. **Cloud fast-path is cleaner than a second local engine.** The
   bootstrap form exposes `WM_CLOUD_STT_FASTPATH`; when on AND
   network OK, route to Whisper API for sub-300 ms transcription.
   Falls back to local on any error. This replaces the Moonshine
   fast-path in the original sketch.

---

## 2. What this builds

### 2.1 Binary: `wm-stt`

A long-running Rust daemon. On startup:

1. Load the configured whisper.cpp model (`distil-small.en` default,
   path `/usr/share/wintermute/models/whisper-distil-small-en.bin`).
2. Open Unix socket `mic.sock` from `wm-audio` for PCM subscription.
3. Subscribe to agorabus for `wm.audio.speech.*` events.
4. On `speech.start`: open a transcription session, begin streaming
   PCM chunks into a buffer.
5. Emit `wm.stt.partial` events every ~500 ms during active speech
   with the current best-guess transcript.
6. On `speech.end`: finalize transcription, compute confidence, emit
   `wm.stt.final` or `wm.stt.uncertain`.

Events published:

| Topic | Payload |
|---|---|
| `wm.stt.partial` | `{text, ts}` |
| `wm.stt.final` | `{text, confidence, duration_ms, model, ts}` |
| `wm.stt.uncertain` | `{text, confidence, ts}` (below threshold) |
| `wm.stt.error` | `{kind, message, ts}` |

### 2.2 Cloud fast-path

When `WM_CLOUD_STT_FASTPATH=true` AND `wm-net` reports the network
healthy:
- Speech chunks are *also* streamed to the Whisper API endpoint
  (`POST /v1/audio/transcriptions`) as they arrive
- Whichever completes first (local or cloud) emits the final event;
  the other is cancelled
- On cloud error, local result is used; never blocks

API key is read from `WM_ANTHROPIC_API_KEY`'s peer field
`WM_OPENAI_API_KEY` (only if cloud STT enabled). Bootstrap collects
it conditionally — if cloud fast-path is checked, a third field
appears.

### 2.3 Model swap

`wm-stt --reload-model <name>` triggers a hot swap:
- Allowed names: `distil-small.en`, `small.en`, `medium.en`,
  `large-v3-turbo` (any name with a corresponding `.bin` in
  `/usr/share/wintermute/models/`)
- Daemon completes in-flight transcription before swap
- ~2 s warmup; emits `wm.stt.model_loaded` event on completion

### 2.4 Confidence threshold

Default 0.45 (whisper.cpp's `no_speech_prob` inverted into a rough
confidence score). Below threshold → `wm.stt.uncertain`. The brain's
default response to uncertain: "Sorry, could you say that again?"
(spoken via wm-tts).

---

## 3. Open-source dependencies

| Crate / tool | Version | Purpose | License |
|---|---|---|---|
| `whisper-rs` | ^0.13 | whisper.cpp bindings | MIT |
| `whisper.cpp` | bundled | inference engine | MIT |
| `tokio` | ^1.40 | async runtime | MIT |
| `serde` + `serde_json` | ^1 | event payloads | MIT |
| `reqwest` | ^0.12 (optional, for cloud fast-path) | HTTP client | MIT/Apache-2.0 |
| `agorabus` client | local | pub/sub | local |
| Whisper API (cloud) | OpenAI | optional fast-path | commercial |

---

## 4. Acceptance criteria

1. Warm `distil-small.en` transcription of a 5-second utterance
   completes in ≤2 s on this laptop's CPU.
2. `wm.stt.partial` events emit at ~500 ms cadence during active
   speech.
3. Confidence below 0.45 emits `wm.stt.uncertain` instead of
   `wm.stt.final`; threshold is configurable.
4. With `WM_CLOUD_STT_FASTPATH=true` and network up, end-to-end
   transcription round-trip ≤500 ms for a 5-second utterance.
5. Network drop during cloud fast-path falls back to local result
   without dropping the in-flight utterance (no double-firing of
   `wm.stt.final`).
6. `wm-stt --reload-model small.en` completes in <5 s without
   dropping `mic.sock` subscription.
7. 60-minute steady-state run shows RSS growth <50 MB (no leak).
8. Daemon recovers from `wm-audio` restart by re-subscribing to
   `mic.sock` within 5 s.

## 5. Out of scope (Fleet 2 / 3)

- Multi-language detection — Fleet 3 if needed; v1 assumes English.
- Custom vocabulary / hotword biasing for proper nouns
  (her name, family names, medications) — Fleet 3.
- Speaker-conditioned STT — Fleet 3.

## 6. Risks

- **whisper-rs version drift** — pin to a known-good revision; track
  upstream for whisper.cpp CUDA/Vulkan support that may help GPU
  variants in Fleet 2.
- **CPU saturation on long utterances** — if she speaks for >30 s
  uninterrupted, transcription can lag. Mitigation: chunk at speech
  pauses (Silero VAD already does), emit partials, and let dialog
  surface "still listening" if needed.
- **Cloud privacy** — `WM_CLOUD_STT_FASTPATH=false` by default;
  document clearly that turning it on sends speech to OpenAI.

## 7. Open questions

- Should we ship `medium.en` as default for users with a 4-core+
  CPU and let `distil-small.en` be the low-end? Probably no — pick
  one default, document the opt-up command.
- Should `wm-stt` emit `wm.stt.partial` with very low confidence
  during early frames, or wait for stability? Leaning: emit early
  with confidence; dialog can ignore them. Helps with debugging.
