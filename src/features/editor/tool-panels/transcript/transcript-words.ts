// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { TranscriptRange } from "../tool-panel-transcript";

/** One word as the transcriber heard it, timed in source milliseconds. */
export type TranscriptWord = TranscriptRange & {
  /** How sure the transcriber was of it, 0 to 1. */
  probability: number;
  text: string;
};

/** Words heard with less confidence than this are marked for a second look. */
export const UNSURE_BELOW = 0.5;

/** A pause this long after the end of a sentence starts a new paragraph. */
const PARAGRAPH_PAUSE_MS = 1_500;
/** A pause this long starts a new paragraph wherever it falls. */
const LONG_PAUSE_MS = 4_000;
const SENTENCE_END = /[.!?。！？]["')\]]*$/u;

/** The first index in `[0, count)` where `test` holds, for a `test` that
 * fails up to some index and holds from there on; `count` where it never
 * holds. */
export function firstIndex(count: number, test: (index: number) => boolean) {
  let low = 0;
  let high = count;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (test(middle)) high = middle;
    else low = middle + 1;
  }
  return low;
}

/** The middle of a stretch, which decides what covers it: a cut or a
 * correction that only clips a word's edge does not take it. */
export const middleOf = ({ endMs, startMs }: TranscriptRange) =>
  (startMs + endMs) / 2;

/** The items of each paragraph, as `[start, end)` indices. A pause's own
 * duration decides the break, since it always sits in the gap it covers;
 * two plain words are never apart long enough to need one, so a break only
 * ever falls right after a pause, keeping it with the paragraph it ends. */
export function transcriptParagraphs(
  items: readonly (TranscriptRange & {
    heard: string;
    kind: "pause" | "word";
  })[],
) {
  const paragraphs: { end: number; start: number }[] = [];
  let start = 0;
  let lastWordHeard = "";
  for (let index = 0; index < items.length; index++) {
    const item = items[index];
    if (item.kind !== "pause") {
      lastWordHeard = item.heard;
      continue;
    }
    const pause = item.endMs - item.startMs;
    if (
      pause >= LONG_PAUSE_MS ||
      (pause >= PARAGRAPH_PAUSE_MS && SENTENCE_END.test(lastWordHeard))
    ) {
      paragraphs.push({ end: index + 1, start });
      start = index + 1;
    }
  }
  if (items.length > start) paragraphs.push({ end: items.length, start });
  return paragraphs;
}

/** Whether the edit leaves `item` out: its middle falls inside one of
 * `removed`, which is sorted by start and never overlapping. */
export function isWordRemoved(
  item: TranscriptRange,
  removed: readonly TranscriptRange[],
) {
  const middle = middleOf(item);
  const after = firstIndex(removed.length, (i) => removed[i].startMs > middle);
  return after > 0 && middle < removed[after - 1].endMs;
}

/** The item being said at `ms`, or -1 between items. */
export function wordAt(items: readonly TranscriptRange[], ms: number) {
  const index = firstIndex(items.length, (i) => items[i].startMs > ms) - 1;
  return index >= 0 && ms < items[index].endMs ? index : -1;
}

/** The nearest item from `index` on, stepping by `step`, that the edit still
 * keeps; -1 where there is none. */
export function nearestKeptWord(
  removedFlags: readonly boolean[],
  index: number,
  step: -1 | 1,
) {
  for (let i = index; i >= 0 && i < removedFlags.length; i += step)
    if (!removedFlags[i]) return i;
  return -1;
}

/** A run of items `first` to `last` cover where they are `removed` (or
 * kept), by index into `items`. */
export type WordRunSpan = { first: number; last: number };

/** The runs the items `first` to `last` cover where they are `removed` (or
 * kept), one per unbroken run, by index. */
export function wordRunSpans(
  removedFlags: readonly boolean[],
  { first, last, removed }: { first: number; last: number; removed: boolean },
): WordRunSpan[] {
  const runs: WordRunSpan[] = [];
  let open: WordRunSpan | null = null;
  for (let i = Math.max(first, 0); i <= last && i < removedFlags.length; i++) {
    if (removedFlags[i] !== removed) {
      open = null;
      continue;
    }
    if (open) open.last = i;
    else {
      open = { first: i, last: i };
      runs.push(open);
    }
  }
  return runs;
}

/** The stretches the items `first` to `last` cover where they are `removed`
 * (or kept), one per unbroken run, each from its first item's start to its
 * last item's end. */
export function wordRuns(
  items: readonly TranscriptRange[],
  removedFlags: readonly boolean[],
  options: { first: number; last: number; removed: boolean },
) {
  return wordRunSpans(removedFlags, options).map((span): TranscriptRange => ({
    endMs: items[span.last].endMs,
    startMs: items[span.first].startMs,
  }));
}
