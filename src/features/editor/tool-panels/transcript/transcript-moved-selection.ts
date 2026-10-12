// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TranscriptUnit, WordRange } from "./transcript-units";

const sameUnit = (a: TranscriptUnit, b: TranscriptUnit) =>
  a.kind === b.kind && a.text === b.text;

/**
 * Where words a drag carried landed, once the transcript is laid out again:
 * the run of `units` reading as `carried` does, the occurrence nearest
 * `near` where the same words appear more than once. A pause at either end
 * of what was carried can join the silence it lands beside, so only the
 * words from the first to the last are looked for. Null where they are not
 * found.
 */
export function movedSelection(
  units: readonly TranscriptUnit[],
  carried: readonly TranscriptUnit[],
  near: number,
): WordRange | null {
  const words = carried.flatMap((unit, index) =>
    unit.kind === "word" ? [index] : [],
  );
  if (words.length === 0) return null;
  const run = carried.slice(words[0], words[words.length - 1] + 1);
  let found: number | null = null;
  for (let start = 0; start + run.length <= units.length; start++) {
    if (!run.every((unit, offset) => sameUnit(unit, units[start + offset])))
      continue;
    if (found === null || Math.abs(start - near) < Math.abs(found - near))
      found = start;
  }
  return found === null
    ? null
    : { anchor: found, focus: found + run.length - 1 };
}
