// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { normalizeText } from "../../tool-panels/transcript/transcript-units";

/** Text typed over bytes `start` to `end` of a caption being typed into. */
export type CaptionRetype = { end: number; start: number; text: string };

const encoder = new TextEncoder();
const bytesOf = (text: string) => encoder.encode(text).length;

/** Which of `typed` and `heard` pair up as the same word: the longest run in
 * common, in order, as pairs of their places. */
function common(typed: readonly string[], heard: readonly string[]) {
  const longest = Array.from({ length: typed.length + 1 }, () =>
    new Array<number>(heard.length + 1).fill(0),
  );
  for (let i = typed.length - 1; i >= 0; i--)
    for (let j = heard.length - 1; j >= 0; j--)
      longest[i][j] =
        typed[i] === heard[j]
          ? longest[i + 1][j + 1] + 1
          : Math.max(longest[i + 1][j], longest[i][j + 1]);
  const pairs: [number, number][] = [];
  let [i, j] = [0, 0];
  while (i < typed.length && j < heard.length) {
    if (typed[i] === heard[j]) {
      pairs.push([i, j]);
      i++;
      j++;
    } else if (longest[i + 1][j] >= longest[i][j + 1]) i++;
    else j++;
  }
  return pairs;
}

/**
 * What putting back what was heard for the word typed at byte `wordStart`
 * types, where `run` is the words typed for the same words said, starting
 * at byte `run.start` of the caption's text, and `heard` those words as
 * heard. Null where the word reads as it was heard.
 *
 * The typed words are paired with the heard ones they kept; between two
 * pairs, a word typed for one heard word gets that word back, a word typed
 * where nothing was heard goes, and words typed for a different number of
 * heard words, which cannot be told apart, get those words back together.
 */
export function captionTypedRestore(
  run: { start: number; text: string },
  wordStart: number,
  heard: readonly string[],
): CaptionRetype | null {
  const typed = [...run.text.matchAll(/\S+/gu)].map((match) => {
    const start = run.start + bytesOf(run.text.slice(0, match.index));
    return { end: start + bytesOf(match[0]), start, text: match[0] };
  });
  const at = typed.findIndex((word) => word.start === wordStart);
  if (at < 0) return null;
  const pairs = common(
    typed.map((word) => normalizeText(word.text)),
    heard.map(normalizeText),
  );
  if (pairs.some(([pairedTyped]) => pairedTyped === at)) return null;
  const earlier = pairs.filter(([pairedTyped]) => pairedTyped < at);
  const before = earlier[earlier.length - 1] as [number, number] | undefined;
  const after = pairs.find(([pairedTyped]) => pairedTyped > at);
  const [typedFrom, heardFrom] = before
    ? [before[0] + 1, before[1] + 1]
    : [0, 0];
  const [typedTo, heardTo] = after ?? [typed.length, heard.length];
  if (typedTo - typedFrom === heardTo - heardFrom)
    return { ...typed[at], text: heard[heardFrom + at - typedFrom] };
  if (heardTo === heardFrom) return { ...typed[at], text: "" };
  return {
    end: typed[typedTo - 1].end,
    start: typed[typedFrom].start,
    text: heard.slice(heardFrom, heardTo).join(" "),
  };
}
