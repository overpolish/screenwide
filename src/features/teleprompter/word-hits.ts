// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { type ScriptText } from "./script-words";

/** The script's drawn words, by index. */
export type DrawnWords = ReadonlyMap<number, HTMLElement>;

/** A point on screen, as a pointer event gives it. */
export type ScreenPoint = { clientX: number; clientY: number };

/** Where the word under the point starts in the text, if one is. The words
 * are drawn under the field, which hit testing passes through. */
export function startOfWordAt(
  words: DrawnWords,
  script: ScriptText,
  { clientX, clientY }: ScreenPoint,
) {
  const under = document.elementsFromPoint(clientX, clientY);
  for (const [index, element] of words)
    if (under.includes(element)) return script.starts[index];
  return undefined;
}

/** Where the word nearest a point that is on no word starts, on the line
 * nearest its height: the nearest along the line for a point between its
 * words, the line's first for a point beside the text. */
function startOfLineAt(
  words: DrawnWords,
  script: ScriptText,
  { clientX, clientY }: ScreenPoint,
) {
  let line: { index: number; left: number; right: number }[] = [];
  let distance = Infinity;
  for (const [index, element] of words) {
    const { bottom, left, right, top } = element.getBoundingClientRect();
    const away = Math.abs((top + bottom) / 2 - clientY);
    if (away < distance) {
      line = [];
      distance = away;
    }
    if (away === distance) line.push({ index, left, right });
  }
  if (line.length === 0) return undefined;
  const start = Math.min(...line.map(({ left }) => left));
  const end = Math.max(...line.map(({ right }) => right));
  let best = line.reduce((first, word) =>
    word.index < first.index ? word : first,
  );
  if (clientX > start && clientX < end) {
    const along = ({ left, right }: { left: number; right: number }) =>
      Math.max(left - clientX, clientX - right, 0);
    for (const word of line) if (along(word) < along(best)) best = word;
  }
  return script.starts[best.index];
}

/** The word under the point, or the nearest on the line beside it. */
export const wordStartNear = (
  words: DrawnWords,
  script: ScriptText,
  point: ScreenPoint,
) => startOfWordAt(words, script, point) ?? startOfLineAt(words, script, point);
