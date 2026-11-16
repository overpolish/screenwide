<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Dictation's cleanup evaluation

English dictations for choosing the model that applies the corrections a speaker makes as they speak (#47), and the scores of the models tried.

## The dictations

`cases.json` holds 64 dictations, each with the text it should become: 16 corrections marked by a word ("at 2, actually 3"), 6 without one ("on Monday, on Tuesday"), 12 where the marking word is meant ("I actually enjoyed the film"), 8 with names to keep or correct, 6 with spoken commands round corrections, and 16 of clean text that needs no change. Two corrections and two meant words are said after a pause and without a comma, as people say them; dictation marks such a pause with a comma (`src/commands/render.rs`), and the meant ones check the comma does no harm. `audio/` holds each said by a macOS voice, its pieces a pause apart; `node src-tauri/crates/dictation/evaluation/record.mjs` records any that are missing. The expected text is written as the English Parakeet model hears each voice, misheard words included, so a score measures the cleanup model rather than the recognizer.

## Running it

Run `pnpm transcriber:prepare`, then, with FFmpeg on the PATH and the models downloaded:

```sh
cargo run -p screenwide-dictation --example evaluate_cleanup -- \
  --recognizer parakeet-unified-en-0.6b-Q8_0.gguf \
  --model speakoflow-mini-q8=SpeakoFlow-Mini-0.8B-Q8_0.gguf \
  --model speakoflow-mini-q4=SpeakoFlow-Mini-0.8B-Q4_K_M.gguf \
  --model s1-mini=s1-mini-q4_k_m.gguf \
  --model gemma-4-e2b=gemma-4-E2B-it-Q4_0.gguf
```

Each dictation is heard as dictation hears it, its commands carried out, then corrected by each model with the prompt its authors give (`src/correct/prompts.rs`). The answers are written to `results.md`, dictation by dictation.

## Results

On an Apple M4 Pro, each model on the GPU through llama.cpp, the model loaded before timing:

| Model | Applied | Unchanged | Exactly | Losing words | Words lost | Checked: applied | Checked: unchanged | Checked: losing words | Median ms | Slowest ms |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| none (as heard) | 0/27 | 37/37 | 37/37 | 0 | 0 | 0/27 | 37/37 | 0 | 0 | 0 |
| SpeakoFlow Mini, 8 bits | 27/27 | 35/37 | 30/37 | 2 | 4 | 26/27 | 37/37 | 0 | 142 | 224 |
| SpeakoFlow Mini, 4 bits | 25/27 | 35/37 | 31/37 | 3 | 6 | 24/27 | 37/37 | 1 | 139 | 205 |
| Superwhisper S1-mini, 4 bits | 22/27 | 36/37 | 0/37 | 3 | 4 | 21/27 | 37/37 | 0 | 96 | 148 |
| Gemma 4 E2B, 4 bits | 18/27 | 35/37 | 33/37 | 4 | 13 | 18/27 | 37/37 | 0 | 204 | 341 |

Each answer is scored with the line breaks and quotation marks of the text as heard and no others (`src/correct/said.rs`), as dictation takes it. Applied counts the dictations needing a correction that came out as expected, ignoring case and punctuation; Unchanged, those needing none whose words came back as heard, and Exactly, character for character. Words lost counts words both heard and expected that the answer left out. Checked is what dictation pastes: the answer when it passes the lost-word check (`src/correct/kept.rs`), and the text as heard when it does not.

- S1-mini changed one clean word ("third" to "3rd") but punctuates every answer, so none comes back exactly; it missed corrections without a marking word.
- Gemma 4 E2B, a general model, left most corrections without a marking word undone and answered a dictated question ("Paris").
- SpeakoFlow Mini applied every correction at 8 bits. Its lost words were a contrast it took for a correction ("It's not Tuesday, it's Wednesday") and a number it wrote as a word ("5th" as "fifth"); the check turns both down. The check also turns down one right answer, a restart with no marking word ("I think we should, we need to"), which is pasted as heard.

## Choice

SpeakoFlow Mini at 8 bits: the most corrections applied, no words lost once checked, and about 150 ms a correction. Its weights are Apache-2.0, as are those of Qwen3.5 0.8B it is trained from. S1-mini's Apache-2.0 licence adds a term on how it must be named, and it scored lower; Gemma 4 E2B is Apache-2.0 but scored lowest and is five times the download.

# Recognition

A cleanup model can only correct what was heard: "coffee at free, actually at two" stays as it is. `evaluate_recognition` scores the ways English dictation could be recognised, on the macOS voices above (but the spoken commands) and on a person's own recordings of `voice-cases.json`, sentences heavy on the sounds speech models confuse. `node src-tauri/crates/dictation/evaluation/record-voice.mjs` records them into `voice/`, which stays on that person's computer. Run it as:

```sh
cargo run -p screenwide-dictation --example evaluate_recognition -- \
  --stream parakeet-en=parakeet-unified-en-0.6b-Q8_0.gguf \
  --vad silero-vad.bin \
  --at-pauses whisper-turbo=ggml-large-v3-turbo-q8_0.bin \
  --at-pauses parakeet-ultra=parakeet-ultra-Q8_0.gguf \
  --rewritten parakeet-en=parakeet-unified-en-0.6b-Q8_0.gguf,parakeet-ultra-Q8_0.gguf \
  --cleanup SpeakoFlow-Mini-0.8B-Q8_0.gguf
```

Words are compared against what was said, with numbers spelled out so "3:30" and "three thirty" score alike. Finish is from the speaker stopping to the last of the text, which for a recogniser writing at pauses is about the time it takes to write a stretch. Each recording, as each recogniser heard it, is in `recognition.md`.

| Recogniser | macOS voices: word errors | One speaker's recordings: word errors | Median finish ms | One speaker's recordings as expected once corrected |
| --- | --- | --- | --- | --- |
| Parakeet Unified EN, streaming | 1.6% (8/503) | 7.7% (4/52) | 44 | 4/6 |
| Whisper large-v3 turbo, at pauses | 1.0% (5/503) | 3.8% (2/52) | 493 | 4/6 |
| Parakeet Ultra, at pauses | 0.8% (4/503) | 3.8% (2/52) | 42 | 4/6 |
| Parakeet Unified EN streaming, Parakeet Ultra again at pauses | 0.8% (4/503) | 3.8% (2/52) | 44 | 4/6 |

Six recordings of one speaker are too few to choose on alone, but both sets point the same way: a model hearing the whole stretch after a pause, with the words on both sides of each, makes about half the mistakes of the streaming model, which hears a word with little after it. The streaming model heard "coffee at free" and "the invoice from the third. Nice, the fourth"; both models writing at pauses heard "coffee at three" and "on the third, no, the fourth". None heard "three free tickets" as said.

English dictation takes the last: Parakeet Unified streams the words as they are spoken, and Parakeet Ultra writes each stretch again once a pause ends it (`transcriber/src/dictation/pauses/rewritten.rs`), its text taking the streamed words' place. It scores as Parakeet Ultra does alone, keeps the words appearing as they are said, and adds about 40 ms to each pause; Whisper scored no better and takes half a second.
