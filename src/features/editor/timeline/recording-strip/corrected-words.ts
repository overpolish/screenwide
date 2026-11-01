// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type TranscriptCorrection } from "../../tool-panels/tool-panel-transcript";
import { transcriptUnits } from "../../tool-panels/transcript/transcript-units";

import { linesOf, type RecordingReadings } from "./recording-beats";

// A word's spelling is the transcript's, corrected in the Script island, so
// the strip reads what was said as the captions and the script do. The words
// are read as heard and the corrections kept apart in the edit; they meet
// here.

type Words = RecordingReadings["words"];

/** `words` as the transcript reads them now: the words one correction covers
 * become one word under its text, at the times they were heard. A word
 * corrected to nothing is hidden from the captions, and is left out. */
function correctedWords(
  words: Words,
  corrections: readonly TranscriptCorrection[],
): Words {
  const heard = [...words]
    .sort((one, other) => one.start - other.start)
    .map(({ end, start, text }) => ({
      endMs: end * 1000,
      probability: 1,
      startMs: start * 1000,
      text,
    }));
  const sorted = [...corrections].sort(
    (one, other) => one.startMs - other.startMs,
  );
  return transcriptUnits(heard, sorted, { removed: [] }).flatMap((unit) =>
    unit.kind === "word" && unit.text.trim()
      ? [
          {
            end: unit.endMs / 1000,
            start: unit.startMs / 1000,
            text: unit.text,
          },
        ]
      : [],
  );
}

/** The lines `words` were said in (`linesOf`), each reading as corrected.
 * The lines are where they were heard, so they keep to the speech beats
 * found in the same words; a line whose every word is hidden is left out. */
export function correctedLines(
  words: Words,
  corrections: readonly TranscriptCorrection[],
) {
  const lines = linesOf(words);
  if (corrections.length === 0) return lines;
  const corrected = correctedWords(words, corrections);
  return lines.flatMap((line) => {
    const said = corrected.filter(
      (word) => word.start >= line.start && word.start <= line.end,
    );
    if (said.length === 0) return [];
    return [
      {
        ...line,
        text: said.map((word) => word.text.trim()).join(" "),
        words: said.map((word) => ({ at: word.start, word: word.text.trim() })),
      },
    ];
  });
}
