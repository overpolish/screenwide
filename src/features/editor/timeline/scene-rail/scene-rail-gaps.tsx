// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cn } from "../../../../lib/styling";
import { SQUEEZED_HATCH } from "../strip/taken-out-band";
import { placed, type Span, type TimelineScale } from "../timeline-scale";

// Above each gap the rail is out of bounds: no scene plays over a gap
// (`recording-scene-gaps.ts`), so none is drawn there, and a scene drawn or
// moved towards one stops at its edge. It is hatched with a dashed edge, as
// what the squeeze took is (`TakenOutBand`), drawn where a scene would sit,
// a little inside the rail's height; the pointer anywhere over it, the
// rail's edges above and below included, says nothing can be done there.

export function SceneRailGaps({
  gaps,
  scale,
}: {
  /** In the rail's seconds, as the strip lays its lanes out. */
  gaps: readonly Span[];
  scale: TimelineScale;
}) {
  return gaps.map((gap) => (
    <span
      aria-hidden
      className="absolute inset-y-0 cursor-not-allowed"
      key={gap.start}
      style={placed(scale, gap.start, gap.end)}
    >
      <span
        className={cn(
          "absolute inset-x-0 inset-y-0.5 rounded-part border border-dashed border-fill-strong",
          SQUEEZED_HATCH,
        )}
      />
    </span>
  ));
}
