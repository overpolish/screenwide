// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Point } from "./pixel-analysis";
import { DistanceProbeSvg } from "./ruler-probe-svg";
import { DistanceProbe } from "./ruler-types";

/**
 * The sibling's 1px stroke plus 1px of air each side - plain screen pixels,
 * since this layer never scales.
 */
const INTERSECT_GAP = 3;

/**
 * Transient probe previews, rendered OUTSIDE the zoomed world layer: they are
 * cursor furniture, and screen space keeps their 1px strokes and the cut-out
 * at their intersection crisp at any zoom - the scaled world layer's capped
 * rasterisation would swallow both. Stamped probes stay in the world layer,
 * anchored to the content they measure.
 */
export function PreviewProbeLayer({
  probes,
  showLabels = false,
  showLines = true,
  toScreen,
}: {
  probes: readonly DistanceProbe[];
  toScreen: (point: Point) => Point;
  showLabels?: boolean;
  showLines?: boolean;
}) {
  if (probes.length === 0) return null;
  const converted = probes.map((probe) => {
    const vertical = probe.axis === "y";
    const from = toScreen(
      vertical
        ? { x: probe.position, y: probe.start }
        : { x: probe.start, y: probe.position },
    );
    const to = toScreen(
      vertical
        ? { x: probe.position, y: probe.end }
        : { x: probe.end, y: probe.position },
    );
    const labelDistance = Math.round(Math.abs(probe.end - probe.start));
    return vertical
      ? {
          ...probe,
          end: to.y,
          labelDistance,
          position: from.x,
          start: from.y,
        }
      : {
          ...probe,
          end: to.x,
          labelDistance,
          position: from.y,
          start: from.x,
        };
  });
  return (
    <svg className="pointer-events-none absolute inset-0 size-full overflow-visible">
      {converted.map((probe) => (
        <DistanceProbeSvg
          gapAt={converted.find((other) => other.axis !== probe.axis)?.position}
          gapSize={INTERSECT_GAP}
          key={probe.axis}
          labelDistance={probe.labelDistance}
          probe={probe}
          showLabel={showLabels}
          showLine={showLines}
        />
      ))}
    </svg>
  );
}
