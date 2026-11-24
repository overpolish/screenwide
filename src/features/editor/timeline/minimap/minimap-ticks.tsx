// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Span, type TimeMap, timelineScale } from "../timeline-scale";

const share = (part: number) => `${String(part * 100)}%`;

export type MinimapBeats = {
  /** The beats, drawn as ticks; a moment's in its kind's colour,
   * `#rrggbb`, so the moments worth keeping stand out of the whole. */
  beats: (Span & { color?: string })[];
  /** How the recording plays now, which places the ticks and sets how much
   * of the recording's length the overview spans. */
  map: TimeMap;
};

/** The line along the overview, and a tick for each beat on it. */
export function MinimapTicks({ beats, map }: MinimapBeats) {
  const scale = timelineScale(map);
  return (
    <>
      <span className="absolute inset-x-0 top-1/2 h-px -translate-y-1/2 bg-fill-strong" />
      {beats.map((beat) => (
        <span
          aria-hidden
          className="absolute inset-y-0.5 min-w-px rounded-full bg-fg-tertiary"
          key={beat.start}
          style={{
            backgroundColor: beat.color,
            insetInlineStart: share(scale.toX(beat.start)),
            width: share(scale.toX(beat.end, true) - scale.toX(beat.start)),
          }}
        />
      ))}
    </>
  );
}
