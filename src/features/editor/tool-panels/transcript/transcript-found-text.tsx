// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { TranscriptMatch } from "./transcript-search";
import { FOUND_TEXT_CLASS } from "./transcript-unit-styles";
import { TranscriptUnit } from "./transcript-units";

/** Where the matches fall in each unit's text, as `[from, to)` letter
 * ranges by unit, and which match each unit is part of (-1 for none), so
 * the space inside a phrase found is tinted with it and the one between two
 * matches is not. */
export function foundRanges(
  units: readonly TranscriptUnit[],
  matches: readonly TranscriptMatch[],
) {
  const ranges = new Map<number, [number, number][]>();
  const owners = new Int32Array(units.length).fill(-1);
  matches.forEach((match, index) => {
    for (let unit = match.first; unit <= match.last; unit++) {
      const from = unit === match.first ? match.startOffset : 0;
      const to =
        unit === match.last ? match.endOffset : units[unit].text.length;
      const list = ranges.get(unit) ?? [];
      list.push([from, to]);
      ranges.set(unit, list);
      owners[unit] = index;
    }
  });
  return { owners, ranges };
}

/** `text` with the letters a search found in it tinted, the rest as it is. */
export function foundText(
  text: string,
  ranges: readonly [number, number][] | undefined,
): ReactNode {
  if (!ranges) return text;
  const pieces: ReactNode[] = [];
  let at = 0;
  for (const [from, to] of ranges) {
    if (from > at) pieces.push(text.slice(at, from));
    pieces.push(
      <span className={FOUND_TEXT_CLASS} key={from}>
        {text.slice(from, to)}
      </span>,
    );
    at = to;
  }
  if (at < text.length) pieces.push(text.slice(at));
  return pieces;
}
