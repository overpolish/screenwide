// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span } from "../timeline-scale";

// Everything taken out of the recording, as the band past the strip's end
// accounts for it: what the cuts deleted, and what playing faster saved,
// which is the squeeze and any lull retimed to play faster. Together they
// are the recording's length less what plays now, so the strip and the band
// are always as long as the recording.

/** What is taken out of the recording, in seconds. */
export type TakenOut = {
  /** What the cuts left out. */
  deleted: number;
  /** What playing faster saves. */
  squeezed: number;
};

/** What `cuts`, in recorded seconds, and playing faster take out of a
 * recording `recorded` seconds long that plays in `played`. */
export function takenOut(
  cuts: readonly Span[],
  played: number,
  recorded: number,
): TakenOut {
  // Cuts that overlap are counted once.
  let deleted = 0;
  let reached = 0;
  for (const cut of [...cuts].sort((one, other) => one.start - other.start)) {
    const start = Math.max(cut.start, reached, 0);
    const end = Math.min(cut.end, recorded);
    if (end > start) deleted += end - start;
    reached = Math.max(reached, end);
  }
  return { deleted, squeezed: Math.max(recorded - played - deleted, 0) };
}

/** The whole of what is taken out, in seconds. */
export const takenTotal = ({ deleted, squeezed }: TakenOut) =>
  deleted + squeezed;
