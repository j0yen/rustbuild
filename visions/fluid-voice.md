# Vision: fluid-voice — wake-to-first-word under 800 ms

**Authored by:** /dream (Claude Sonnet 4.6), with jsy
**Created:** 2026-06-20
**Status:** active
**Seed:** user — live voice session revealed 4 concrete latency sources
(diagnosed 2026-06-20: brain 401 degradation, Whisper re-init per turn,
no pipeline streaming, VAD floor rejections).

## TL;DR

The voice pipeline works but every turn pays unnecessary latency tax at
four points. Whisper re-creates a 97 MB decode buffer on every utterance
instead of keeping the state warm. The brain waits for the LLM to finish
before speaking the first word. A broken API key causes silent nonsense
rather than a visible error and local fallback. Short utterances are
silently dropped instead of padded to Whisper's floor. Fix all four and
the turn latency drops from 3–5 s to under 800 ms for a typical short
command.

## Root causes (evidence-backed, observed 2026-06-20)

| # | Root cause | Evidence |
|---|-----------|----------|
| 1 | `whisper_engine.rs:finalise()` calls `ctx.create_state()` on every turn | `journalctl -u wm-stt` shows `whisper_init_state: compute buffer (decode) = 97.27 MB` on every utterance |
| 2 | `daemon.rs:1957` uses `collect_messages()` — full LLM response before one `REPLY` | Code read: `BufferingSink`/`DeltaSink` infra exists in ladder but `handle_turn_user` never calls the streaming path |
| 3 | `WM_ANTHROPIC_API_KEY` expired → 401 on every cloud turn → degraded fallback | `journalctl` shows `authentication_error: invalid x-api-key` then `turn degraded` on every request |
| 4 | VAD 300 ms floor → many utterances < 1000 ms → whisper rejects them | Logs: `input is too short - 760 ms < 1000 ms` repeated on every short command |
| 5 | `current_partial()` returns `None` (iter-7 stub) | `whisper_engine.rs:current_partial()`: comment "iter-6: no mid-stream partial decoding — lands in iter-7" |

## End-state

When this vision is fulfilled:

1. **Whisper state is kept warm.** `whisper_init_state` fires once at daemon
   start, not per utterance. The 97 MB decode buffer persists across turns.
2. **The brain speaks the first sentence while generating the rest.**
   `wm.brain.reply.partial` events arrive sentence-by-sentence; wm-dialog
   hands each to wm-tts immediately. First word audible < 1 s after
   transcript arrives.
3. **A broken API key surfaces as a visible error, not nonsense.**
   On startup and every 5 minutes, the brain pings the Anthropic API.
   On 401 it publishes `wm.brain.key_status { ok: false }` and routes
   to the local tier rather than emitting a degraded fallback.
4. **Short utterances are padded, not dropped.** Any PCM window < 1000 ms
   is zero-padded to exactly 1000 ms before Whisper inference. "Tell me a
   story" stops failing at the floor.
5. **Partial transcripts let the brain load context early.** Mid-stream
   `wm.stt.partial` events fire during speech; the brain can warm its
   recall query before `speech.end` arrives.

## Components (PRD-sized pieces)

1. **PRD-fluid-stt-warm-state** (draft) — persist `WhisperState` across turns in `WhisperEngine`. Biggest single latency win (~1–2 s). Independent.
2. **PRD-fluid-stt-audio-padding** (draft) — zero-pad sub-1000 ms PCM buffers before inference. Eliminates "input too short" drops. Independent.
3. **PRD-fluid-brain-key-health** (draft) — proactive API-key health check + local fallback on 401. Eliminates degraded nonsense. Independent.
4. **PRD-fluid-brain-streaming-tts** (draft) — sentence-by-sentence streaming from LLM → `wm.brain.reply.partial` → wm-tts. Depends on key-health (brain must be functional first).
5. **PRD-fluid-stt-partial-transcribe** (draft) — implement `current_partial()` via whisper.cpp encode/decode split; publish `wm.stt.partial` during speech. Depends on warm-state being stable.

## Order

```
PRD-fluid-stt-warm-state     (independent — ship first, biggest win)
PRD-fluid-stt-audio-padding  (independent — parallel with warm-state)
PRD-fluid-brain-key-health   (independent — parallel; fixes 401 silently)
        │
        ▼
PRD-fluid-brain-streaming-tts  (brain must be functional → key-health first)
        │
        ▼
PRD-fluid-stt-partial-transcribe  (needs warm-state stable; ship last)
```

## Open questions

1. **Sentence boundary heuristic.** Splitting LLM output on `.!?` works for
   most replies but breaks on decimals, URLs, ellipses. v1 ships a simple
   regex; a proper sentence segmenter is a follow-up.
2. **Local tier for streaming.** `wm-local-llm` may not support sentence
   streaming with the current ollama backend. The streaming PRD targets the
   cloud path first; local streaming is a follow-on.
3. **Whisper state thread safety.** `WhisperState` in whisper-rs may not be
   `Send`. The warm-state PRD must verify the type and wrap in `Mutex` if
   needed — the existing `ctx: Mutex<WhisperContext>` pattern is the model.
4. **Partial transcripts and false triggers.** `wm.stt.partial` may fire on
   noisy mid-utterance segments. The brain must treat partials as advisory
   (pre-warm recall only), not as a prompt to start answering.

## Notes for /build

- PRD-fluid-stt-warm-state and PRD-fluid-stt-audio-padding both extend
  `~/wintermute/wintermute-stt` — do not dispatch concurrently; they touch
  the same `whisper_engine.rs` and `processor.rs`.
- PRD-fluid-brain-key-health extends `~/wintermute/wintermute-brain`; it is
  a prerequisite for fluid-brain-streaming-tts but can run in parallel with
  both STT PRDs.
- Build order enforced: warm-state → audio-padding → key-health (parallel
  to STT pair) → streaming-tts → partial-transcribe.
