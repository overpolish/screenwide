// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { sampleTranscriptWords } from "../transcript/transcript-panel-fixtures";

import type { Caption } from "../../../../bindings/Caption";

/** Fillers the sample captions leave out, as Hide filler words would. */
const HIDDEN = new Set(["um,", "Uh,"]);
/** Profanity the sample captions star out, as Partly would. */
const STARRED: Record<string, string> = { "damn,": "d***," };

/** The sample transcript as captions of up to six words, a sentence's end
 * closing one early, with its fillers hidden and its profanity partly
 * starred: what the editor would make of it, nothing cut. */
export const sampleCaptions: Caption[] = (() => {
  const captions: Caption[] = [];
  let said: typeof sampleTranscriptWords = [];
  const close = () => {
    if (said.length === 0) return;
    captions.push({
      endMs: said[said.length - 1].endMs,
      id: said[0].startMs,
      lines: [
        said.map((word) => ({
          endMs: word.endMs,
          sourceEndMs: word.endMs,
          sourceStartMs: word.startMs,
          startMs: word.startMs,
          text: STARRED[word.text] ?? word.text,
        })),
      ],
      spaced: true,
      startMs: said[0].startMs,
      text: said.map((word) => word.text).join(" "),
    });
    said = [];
  };
  for (const word of sampleTranscriptWords) {
    if (HIDDEN.has(word.text)) continue;
    said.push(word);
    if (said.length === 6 || /[.!?…]$/u.test(word.text)) close();
  }
  close();
  return captions;
})();
