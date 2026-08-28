// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Bounds, PixelSize, Point } from "./pixel-analysis";
import { RadiusMeasurement } from "./ruler-types";

export type RadiusGeometry = {
  arcEnd: Point;
  arcMidpoint: Point;
  arcStart: Point;
  center: Point;
  path: string;
};

export const radiusGeometry = (
  measurement: RadiusMeasurement,
): RadiusGeometry => {
  const { corner, radius: radiusValue, x, y } = measurement;
  const radius = Math.max(1, radiusValue);
  const x1 = x + measurement.width;
  const y1 = y + measurement.height;
  const right = corner.endsWith("right");
  const bottom = corner.startsWith("bottom");
  const signX = right ? 1 : -1;
  const signY = bottom ? 1 : -1;
  const center = {
    x: right ? x1 - radius : x + radius,
    y: bottom ? y1 - radius : y + radius,
  };
  const arcStart = bottom
    ? right
      ? { x: center.x + radius, y: center.y }
      : { x: center.x, y: center.y + radius }
    : right
      ? { x: center.x, y: center.y - radius }
      : { x: center.x - radius, y: center.y };
  const arcEnd = bottom
    ? right
      ? { x: center.x, y: center.y + radius }
      : { x: center.x - radius, y: center.y }
    : right
      ? { x: center.x + radius, y: center.y }
      : { x: center.x, y: center.y - radius };
  const diagonal = radius / Math.SQRT2;
  const arcMidpoint = {
    x: center.x + signX * diagonal,
    y: center.y + signY * diagonal,
  };
  return {
    arcEnd,
    arcMidpoint,
    arcStart,
    center,
    path: `M ${String(arcStart.x)} ${String(arcStart.y)} A ${String(radius)} ${String(radius)} 0 0 1 ${String(arcEnd.x)} ${String(arcEnd.y)}`,
  };
};

const LABEL_GAP = 8;
const VIEW_MARGIN = 4;

const clamp = (value: number, minimum: number, maximum: number) =>
  Math.max(minimum, Math.min(maximum, value));

const overlapArea = (a: Bounds, b: Bounds) =>
  Math.max(0, Math.min(a.x + a.width, b.x + b.width) - Math.max(a.x, b.x)) *
  Math.max(0, Math.min(a.y + a.height, b.y + b.height) - Math.max(a.y, b.y));

export type RadiusLabelPlacement = Point & { leaderEnd: Point };

/**
 * Parks a radius label outside its measured component where possible. The
 * chosen candidate is clamped into the visible world and scored primarily by
 * overlap, then by how far clamping and the leader have to move it.
 */
export const radiusLabelPlacement = ({
  geometry,
  labelSize,
  measurement,
  visibleBounds,
}: {
  geometry: RadiusGeometry;
  labelSize: PixelSize;
  measurement: RadiusMeasurement;
  visibleBounds?: Bounds;
}): RadiusLabelPlacement => {
  const halfWidth = labelSize.width / 2;
  const halfHeight = labelSize.height / 2;
  const right = measurement.corner.endsWith("right");
  const bottom = measurement.corner.startsWith("bottom");
  const signX = right ? 1 : -1;
  const signY = bottom ? 1 : -1;
  const corner = {
    x: right ? measurement.x + measurement.width : measurement.x,
    y: bottom ? measurement.y + measurement.height : measurement.y,
  };
  const oppositeX = right
    ? measurement.x - halfWidth - LABEL_GAP
    : measurement.x + measurement.width + halfWidth + LABEL_GAP;
  const oppositeY = bottom
    ? measurement.y - halfHeight - LABEL_GAP
    : measurement.y + measurement.height + halfHeight + LABEL_GAP;
  const candidates = [
    {
      x: corner.x + signX * (halfWidth + LABEL_GAP),
      y: corner.y + signY * (halfHeight + LABEL_GAP),
    },
    {
      x: corner.x + signX * (halfWidth + LABEL_GAP),
      y: geometry.arcMidpoint.y,
    },
    {
      x: geometry.arcMidpoint.x,
      y: corner.y + signY * (halfHeight + LABEL_GAP),
    },
    { x: oppositeX, y: geometry.arcMidpoint.y },
    { x: geometry.arcMidpoint.x, y: oppositeY },
  ];
  let best: { center: Point; leaderEnd: Point; score: number } | undefined;
  for (const [index, preferred] of candidates.entries()) {
    const center = visibleBounds
      ? {
          x: clamp(
            preferred.x,
            visibleBounds.x + halfWidth + VIEW_MARGIN,
            visibleBounds.x + visibleBounds.width - halfWidth - VIEW_MARGIN,
          ),
          y: clamp(
            preferred.y,
            visibleBounds.y + halfHeight + VIEW_MARGIN,
            visibleBounds.y + visibleBounds.height - halfHeight - VIEW_MARGIN,
          ),
        }
      : preferred;
    const labelBounds = {
      height: labelSize.height,
      width: labelSize.width,
      x: center.x - halfWidth,
      y: center.y - halfHeight,
    };
    const leaderEnd = {
      x: clamp(
        geometry.arcMidpoint.x,
        labelBounds.x,
        labelBounds.x + labelBounds.width,
      ),
      y: clamp(
        geometry.arcMidpoint.y,
        labelBounds.y,
        labelBounds.y + labelBounds.height,
      ),
    };
    const clampDistance = Math.hypot(
      center.x - preferred.x,
      center.y - preferred.y,
    );
    const leaderDistance = Math.hypot(
      leaderEnd.x - geometry.arcMidpoint.x,
      leaderEnd.y - geometry.arcMidpoint.y,
    );
    const score =
      overlapArea(labelBounds, measurement) * 10_000 +
      clampDistance * 100 +
      leaderDistance +
      index * 0.1;
    if (!best || score < best.score) best = { center, leaderEnd, score };
  }
  return best
    ? { ...best.center, leaderEnd: best.leaderEnd }
    : { ...geometry.arcMidpoint, leaderEnd: geometry.arcMidpoint };
};
