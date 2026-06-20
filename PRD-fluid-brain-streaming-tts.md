# PRD-fluid-brain-streaming-tts

**Status:** Draft v0.1
**Vision:** visions/fluid-voice.md
**build_target:** rust-extend
**build_into:** /home/jsy/wintermute/wintermute-brain

## TL;DR

The brain waits for the entire LLM response before speaking the first word.
Stream sentence-by-sentence from the LLM to wm-tts so the first word is audible
before generation finishes.

## Why this exists

`daemon.rs:1957` in `handle_turn_user`:

```rust
match llm.collect_messages(&req).await {
    Ok(events) => {
        let text = extract_assistant_text(&events);
        // ... later:
        publish.publish(outgoing::REPLY, ...).await
    }
```

`collect_messages` is a gather: it waits for all streaming tokens to arrive,
then emits one `wm.brain.reply` event. wm-dialog receives the reply and hands
the entire text to wm-tts at once. For a 3-sentence answer at haiku speed
(~50 tok/s on cloud), that is 1–2 s before the first word is spoken.

The ladder already has `DeltaSink` / `on_delta` infrastructure (code-read
2026-06-20: `ladder.rs:100–107`, `BufferingSink:619–622`). The daemon doesn't
use it. This PRD threads the delta stream through to agorabus.

## What this builds

Extend `~/wintermute/wintermute-brain` — changes to `daemon.rs` + new
`src/sentence_splitter.rs`:

**`src/sentence_splitter.rs`** — `SentenceSplitter`:
- Consumes text deltas incrementally.
- Emits complete sentences: splits on `[.!?]` followed by whitespace or end-of-
  stream, excluding decimals (`3.14`), initials (`J. S.`), ellipses (`...`).
- `fn push(&mut self, delta: &str) -> Vec<String>` — returns completed
  sentences from the accumulated buffer.
- `fn flush(&mut self) -> Option<String>` — drains any remaining text at turn
  end (handles responses that don't end in punctuation).

**`daemon.rs`** — replace the gather path with streaming:
- Add `wm.brain.reply.partial` outbound topic constant (alongside existing
  `outgoing::REPLY`).
- In `handle_turn_user`, replace `llm.collect_messages` with a streaming call
  that invokes `on_delta` per token. (Wire the existing `LadderSink` path or
  add a streaming collect variant.)
- On each completed sentence from `SentenceSplitter`, publish
  `wm.brain.reply.partial { text: sentence, ts: u64, turn_id: Option<String> }`.
- After the stream ends, flush the splitter and publish any remainder as a
  final `wm.brain.reply.partial`.
- Still publish one `wm.brain.reply { text: full_text, ... }` at end for
  history writeback and wm-dialog's existing REPLY subscriber (no breaking
  change to downstream).

**No changes to wm-dialog or wm-tts in this PRD** — wm-dialog already subscribes
to `wm.brain.reply` and hands full text to TTS. The `reply.partial` topic is
additive; wm-dialog can subscribe in a follow-on PRD. The immediate latency win
comes from wm-dialog subscribing to `reply.partial` — that's the follow-on.
This PRD ships the publisher side so it can be deployed and verified on the bus
before the consumer side changes.

## Acceptance criteria

1. **AC1 — partial events fire before reply.** Integration test with a mock LLM
   that emits 20 tokens over 200 ms: assert at least one `wm.brain.reply.partial`
   event is published on the bus before `wm.brain.reply` is published.
2. **AC2 — sentence splitter emits on punctuation.** Unit tests for
   `SentenceSplitter`:
   - `"Hello world. How are you?"` pushed as two deltas → two sentences emitted.
   - `"Pi is 3.14. Next."` → two sentences, decimal not split.
   - `"..."` → not split mid-ellipsis.
   - `flush()` on `"no punct"` → returns `Some("no punct")`.
3. **AC3 — full text in final reply.** Integration test: collect all
   `reply.partial` texts + final `reply` text; concatenated partials must equal
   `reply.text` (modulo whitespace normalization).
4. **AC4 — history unaffected.** After a streaming turn, assert one entry in
   `state.history` with the complete assistant text (not just the last partial).
5. **AC5 — local tier still works.** Integration test with stub local backend:
   streaming path must not break when the local tier emits deltas synchronously
   in a tight loop.
6. **AC6 — cargo test green.** `cargo test` on `wintermute-brain` passes with
   no regressions.
