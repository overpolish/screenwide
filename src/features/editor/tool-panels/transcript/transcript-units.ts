// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  correctionsWith as correctionsWithSpans,
  normalizeTranscriptText,
  TranscriptCorrectionSpan,
  TranscriptHeardSpan,
} from "../../recording/transcript/transcript-corrections";
import {
  TranscriptCorrection,
  TranscriptRange,
} from "../tool-panel-transcript";

import { transcriptPauses } from "./transcript-pauses";
import {
  isWordRemoved,
  middleOf,
  TranscriptWord,
  UNSURE_BELOW,
} from "./transcript-words";

/** What the transcript shows as one word: a word as it was heard, or the
 * words one correction covers, joined under its text. */
export type TranscriptUnit = TranscriptRange & {
  corrected: boolean;
  /** The words' own text, as the transcriber heard them; empty for a
   * pause. */
  heard: string;
  /** A pause is a gap between words the transcript shows in the flow; it is
   * never corrected, searched or edited, but is chosen and cut like a word. */
  kind: "pause" | "word";
  removed: boolean;
  text: string;
  /** Heard with little confidence, and not corrected since. */
  unsure: boolean;
};

/** The first and last unit chosen, by index. */
export type WordSelection = { first: number; last: number };

/** Units chosen as a range: the one it started from and the one it reaches,
 * which Shift moves. */
export type WordRange = { anchor: number; focus: number };

export const spanOf = ({ anchor, focus }: WordRange): WordSelection => ({
  first: Math.min(anchor, focus),
  last: Math.max(anchor, focus),
});

export const isWithin = (span: WordSelection | null, index: number) =>
  span !== null && span.first <= index && index <= span.last;

/** Text as a reader compares it: runs of spaces as one, none at the ends. */
export const normalizeText = normalizeTranscriptText;

const covers = (range: TranscriptRange, word: TranscriptRange) => {
  const middle = middleOf(word);
  return range.startMs <= middle && middle < range.endMs;
};

/** The words in transcript order, those a correction covers joined into one
 * unit with its text, interleaved with a pause unit wherever the transcript
 * holds a silence long enough to show one, its start and end included where
 * `durationMs` says how long the recording is. `corrections` and `removed`
 * are sorted by start and never overlap. */
export function transcriptUnits(
  words: readonly TranscriptWord[],
  corrections: readonly TranscriptCorrection[],
  {
    durationMs,
    removed,
  }: { removed: readonly TranscriptRange[]; durationMs?: number },
) {
  const units: TranscriptUnit[] = [];
  let next = 0;
  for (let first = 0; first < words.length;) {
    while (
      next < corrections.length &&
      corrections[next].endMs <= middleOf(words[first])
    )
      next++;
    const correction = covers(
      corrections[next] ?? { endMs: 0, startMs: 0 },
      words[first],
    )
      ? corrections[next]
      : undefined;
    let last = first;
    if (correction)
      while (last + 1 < words.length && covers(correction, words[last + 1]))
        last++;
    const covered = words.slice(first, last + 1);
    const heard = covered.map((word) => word.text).join(" ");
    const range = { endMs: words[last].endMs, startMs: words[first].startMs };
    const text = correction?.text ?? heard;
    const corrected = normalizeText(text) !== normalizeText(heard);
    units.push({
      ...range,
      corrected,
      heard,
      kind: "word",
      removed: isWordRemoved(range, removed),
      text,
      unsure:
        !corrected && covered.some((word) => word.probability < UNSURE_BELOW),
    });
    first = last + 1;
  }
  const pauses: TranscriptUnit[] = transcriptPauses(words, {
    durationMs,
  }).map((range) => ({
    ...range,
    corrected: false,
    heard: "",
    kind: "pause",
    removed: isWordRemoved(range, removed),
    text: "",
    unsure: false,
  }));
  // A pause sits exactly in the gap it was found in, so it always sorts
  // between the word before it and the word after.
  return [...units, ...pauses].sort((a, b) => a.startMs - b.startMs);
}

/**
 * `corrections` once `change` is made: it replaces every correction it
 * overlaps, and where its text is what the words it covers were heard as,
 * it leaves them uncorrected instead. The editor applies the same rule.
 */
export function correctionsWith(
  words: readonly TranscriptWord[],
  corrections: readonly TranscriptCorrection[],
  change: TranscriptCorrection,
) {
  return correctionsWithSpans(
    words.map((word): TranscriptHeardSpan => ({
      end: word.endMs,
      start: word.startMs,
      text: word.text,
    })),
    corrections.map((correction): TranscriptCorrectionSpan => ({
      end: correction.endMs,
      start: correction.startMs,
      text: correction.text,
    })),
    { end: change.endMs, start: change.startMs, text: change.text },
  ).map((correction): TranscriptCorrection => ({
    endMs: correction.end,
    startMs: correction.start,
    text: correction.text,
  }));
}

/** Whether `corrections` already hold what `change` makes of them. */
export function isCorrectionApplied(
  words: readonly TranscriptWord[],
  corrections: readonly TranscriptCorrection[],
  change: TranscriptCorrection,
) {
  const applied = correctionsWith(words, corrections, {
    ...change,
    text: normalizeText(change.text),
  });
  return (
    applied.length === corrections.length &&
    applied.every(
      (correction, index) =>
        correction.startMs === corrections[index].startMs &&
        correction.endMs === corrections[index].endMs &&
        normalizeText(correction.text) ===
          normalizeText(corrections[index].text),
    )
  );
}
