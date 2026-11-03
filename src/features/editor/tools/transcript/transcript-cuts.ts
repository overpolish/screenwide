// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { TranscriptUnit, WordSelection } from "./transcript-units";

/**
 * The run of struck units the next (`direction` 1) or previous (-1) cut
 * covers, from `from`: if `from` itself sits in a run, that run is skipped
 * first, so the keys always move to another cut. Null where there is none.
 */
export function nextCutRun(
  units: readonly Pick<TranscriptUnit, "removed">[],
  from: number,
  direction: 1 | -1,
): WordSelection | null {
  let edge = from;
  if (units[from]?.removed)
    while (units[edge + direction]?.removed) edge += direction;
  let at = edge + direction;
  while (at >= 0 && at < units.length && !units[at].removed) at += direction;
  if (at < 0 || at >= units.length) return null;
  let first = at;
  let last = at;
  while (units[first - 1]?.removed) first--;
  while (units[last + 1]?.removed) last++;
  return { first, last };
}

/** The index of the next (1) or previous (-1) unsure word from `from`, kept
 * rather than struck; -1 where there is none. */
export function nextUnsureWord(
  units: readonly Pick<TranscriptUnit, "kind" | "removed" | "unsure">[],
  from: number,
  direction: 1 | -1,
): number {
  let at = from + direction;
  while (at >= 0 && at < units.length) {
    const unit = units[at];
    if (unit.kind === "word" && unit.unsure && !unit.removed) return at;
    at += direction;
  }
  return -1;
}
