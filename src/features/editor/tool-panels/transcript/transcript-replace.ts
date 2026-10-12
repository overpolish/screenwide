// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TranscriptMatch } from "./transcript-search";
import { TranscriptUnit } from "./transcript-units";

import type { TranscriptCorrection } from "../tool-panel-transcript";

/**
 * `replacement` in place of every match, one correction per match: only the
 * matched text is replaced, so whatever of the first and last unit sits
 * outside it is kept. A match that spans several units becomes one phrase
 * correction over all of them. A match already reading as `replacement`
 * needs no correction at all.
 */
export function replacementCorrections(
  units: readonly TranscriptUnit[],
  matches: readonly TranscriptMatch[],
  replacement: string,
): TranscriptCorrection[] {
  const corrections: TranscriptCorrection[] = [];
  for (const match of matches) {
    const firstUnit = units[match.first];
    const lastUnit = units[match.last];
    const prefix = firstUnit.text.slice(0, match.startOffset);
    const suffix = lastUnit.text.slice(match.endOffset);
    const matched =
      match.first === match.last
        ? firstUnit.text.slice(match.startOffset, match.endOffset)
        : [
            firstUnit.text.slice(match.startOffset),
            ...units
              .slice(match.first + 1, match.last)
              .map((unit) => unit.text),
            lastUnit.text.slice(0, match.endOffset),
          ].join(" ");
    if (matched === replacement) continue;
    corrections.push({
      endMs: lastUnit.endMs,
      startMs: firstUnit.startMs,
      text: prefix + replacement + suffix,
    });
  }
  return corrections;
}
