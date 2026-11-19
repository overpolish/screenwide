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

/** Where the first word of the line nearest the height `y` starts, on
 * screen: for a point beside the text rather than on a word. */
export function startOfLineAt(
  words: DrawnWords,
  script: ScriptText,
  y: number,
) {
  let best = -1;
  let distance = Infinity;
  for (const [index, element] of words) {
    const { bottom, top } = element.getBoundingClientRect();
    const away = Math.abs((top + bottom) / 2 - y);
    if (away < distance || (away === distance && index < best)) {
      best = index;
      distance = away;
    }
  }
  return best < 0 ? undefined : script.starts[best];
}

/** The word under the point, or the first of the line beside it. */
export const wordStartNear = (
  words: DrawnWords,
  script: ScriptText,
  point: ScreenPoint,
) =>
  startOfWordAt(words, script, point) ??
  startOfLineAt(words, script, point.clientY);
