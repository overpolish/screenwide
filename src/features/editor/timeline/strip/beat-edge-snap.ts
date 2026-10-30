// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type Snap } from "../annotation-lane/use-annotation-bar-drag";
import { type Span, type TimelineScale } from "../timeline-scale";

/** How close, in px, an annotation's end comes to a beat's edge to snap. */
const SNAP_PX = 8;

/** Where an annotation's end settles on the strip: the nearest edge of a
 * beat in reach, so an arrow can start as a click does. */
export const beatEdgeSnap =
  ({
    beats,
    scale,
    width,
  }: {
    beats: Span[];
    scale: TimelineScale;
    /** The track's width, in px. */
    width: number;
  }): Snap =>
  (seconds) => {
    let best = { isSnapped: false, seconds };
    let nearest = SNAP_PX;
    for (const span of beats) {
      for (const edge of [span.start, span.end]) {
        const offset = Math.abs(scale.toX(edge) - scale.toX(seconds)) * width;
        if (offset > nearest) continue;
        best = { isSnapped: true, seconds: edge };
        nearest = offset;
      }
    }
    return best;
  };
