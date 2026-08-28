// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { RulerComponentBox } from "./api";
import { edgeMass } from "./edge-detection";
import { GradientField, gradientAt } from "./gradient-field";
import { PixelSize, Point, screenToPixel } from "./pixel-analysis";
import { Corner, RadiusMeasurement } from "./ruler-types";

export type RadiusEstimate = {
  box: RulerComponentBox;
  confidence: "high" | "low";
  corner: Corner;
  radius: number;
};

type LocalPoint = { u: number; v: number };
type FittedEstimate = RadiusEstimate & { fitScore: number };

const MAXIMUM_RADIUS = 128;
const MAXIMUM_CURSOR_DISTANCE = 96;
const MAXIMUM_ARC_DISTANCE = 24;
const MAXIMUM_CANDIDATES = 16;
const FIT_TOLERANCE = 2;
const ANGLE_BINS = 8;
/** Small raster curves land one physical pixel inside the analytic curve. */
const rasterEdgeCompensation = (radius: number) => (radius < 10 ? 1 : 0);

const pixelAt = (
  box: RulerComponentBox,
  corner: Corner,
  { u, v }: LocalPoint,
) => ({
  x: corner.endsWith("right") ? box.x + box.width - u : box.x + u,
  y: corner.startsWith("bottom") ? box.y + box.height - v : box.y + v,
});

const clearsThreshold = ({
  axis,
  field,
  point: { x, y },
  threshold,
}: {
  axis: "x" | "y";
  field: GradientField;
  point: Point;
  threshold: number;
}) => {
  const at =
    axis === "x"
      ? { across: y, axis, position: x }
      : { across: x, axis, position: y };
  return gradientAt(field, at) > 0 && edgeMass(field, at) >= threshold;
};

/** First inward edge on each scanline, excluding straight-edge continuations. */
const curvePoints = ({
  box,
  corner,
  field,
  limit,
  threshold,
}: {
  box: RulerComponentBox;
  corner: Corner;
  field: GradientField;
  limit: number;
  threshold: number;
}) => {
  const points = new Map<string, LocalPoint>();
  const add = (point: LocalPoint) => {
    if (point.u <= 0 || point.v <= 0) return;
    points.set(`${String(point.u)}:${String(point.v)}`, point);
  };
  for (let v = 0; v <= limit; v += 1) {
    for (let u = 0; u <= limit; u += 1) {
      if (
        !clearsThreshold({
          axis: "x",
          field,
          point: pixelAt(box, corner, { u, v }),
          threshold,
        })
      )
        continue;
      add({ u, v });
      break;
    }
  }
  for (let u = 0; u <= limit; u += 1) {
    for (let v = 0; v <= limit; v += 1) {
      if (
        !clearsThreshold({
          axis: "y",
          field,
          point: pixelAt(box, corner, { u, v }),
          threshold,
        })
      )
        continue;
      add({ u, v });
      break;
    }
  }
  return [...points.values()];
};

const median = (values: number[]) => {
  values.sort((a, b) => a - b);
  const middle = Math.floor(values.length / 2);
  return values.length % 2 === 0
    ? (values[middle - 1] + values[middle]) / 2
    : values[middle];
};

const residual = ({ u, v }: LocalPoint, radius: number) =>
  Math.abs(Math.hypot(u - radius, v - radius) - radius);

const fitScore = ({
  coverage,
  inlierShare,
  residual: fitResidual,
}: {
  coverage: number;
  inlierShare: number;
  residual: number;
}) => fitResidual * 3 + (1 - coverage) * 0.5 + (1 - inlierShare) * 0.5;

const fitCircle = (points: readonly LocalPoint[], limit: number) => {
  if (points.length < 4) return undefined;
  let best:
    | {
        coverage: number;
        inlierShare: number;
        radius: number;
        residual: number;
      }
    | undefined;
  for (let radius = 2; radius <= limit; radius += 1) {
    const residuals = points.map((point) => residual(point, radius));
    const inliers = points.filter(
      (point, index) =>
        point.u <= radius + FIT_TOLERANCE &&
        point.v <= radius + FIT_TOLERANCE &&
        residuals[index] <= FIT_TOLERANCE,
    );
    const bins = new Set(
      inliers.map((point) => {
        const angle = Math.atan2(radius - point.v, radius - point.u);
        return Math.max(
          0,
          Math.min(
            ANGLE_BINS - 1,
            Math.floor((angle / (Math.PI / 2)) * ANGLE_BINS),
          ),
        );
      }),
    );
    const candidate = {
      coverage: bins.size / ANGLE_BINS,
      inlierShare: inliers.length / points.length,
      radius,
      residual: median([...residuals]),
    };
    if (!best || fitScore(candidate) < fitScore(best)) best = candidate;
  }
  if (!best || best.coverage < 0.25 || best.inlierShare < 0.4) return undefined;
  return best;
};

const estimateCorner = ({
  box,
  corner,
  field,
  threshold,
}: {
  box: RulerComponentBox;
  corner: Corner;
  field: GradientField;
  threshold: number;
}): FittedEstimate | undefined => {
  const limit = Math.min(
    MAXIMUM_RADIUS,
    Math.floor(Math.min(box.width, box.height) / 2),
  );
  if (limit < 2) return undefined;
  const points = curvePoints({ box, corner, field, limit, threshold });
  const fit = fitCircle(points, limit);
  if (!fit) return undefined;
  const radius = Math.min(
    limit,
    fit.radius + rasterEdgeCompensation(fit.radius),
  );
  return {
    box,
    confidence:
      fit.residual <= 1.25 &&
      fit.coverage >= (radius < 10 ? 0.375 : 0.5) &&
      fit.inlierShare >= 0.6
        ? "high"
        : "low",
    corner,
    fitScore: fitScore(fit),
    radius,
  };
};

const corners = (box: RulerComponentBox) =>
  [
    { corner: "top-left", x: box.x, y: box.y },
    { corner: "top-right", x: box.x + box.width, y: box.y },
    { corner: "bottom-left", x: box.x, y: box.y + box.height },
    {
      corner: "bottom-right",
      x: box.x + box.width,
      y: box.y + box.height,
    },
  ] as const;

const cursorArcDistance = ({
  box,
  corner,
  cursor,
  radius,
}: {
  box: RulerComponentBox;
  corner: Corner;
  cursor: Point;
  radius: number;
}) => {
  const origin = pixelAt(box, corner, { u: 0, v: 0 });
  const u = corner.endsWith("right")
    ? origin.x - cursor.x
    : cursor.x - origin.x;
  const v = corner.startsWith("bottom")
    ? origin.y - cursor.y
    : cursor.y - origin.y;
  const radial = Math.abs(Math.hypot(u - radius, v - radius) - radius);
  return radial + Math.max(0, -u, -v, u - radius, v - radius);
};

/** Best continuous fitted arc close to the world-space cursor. */
export const cornerRadiusAt = ({
  boxes,
  cursor,
  field,
  threshold,
  viewport,
}: {
  boxes: readonly RulerComponentBox[];
  cursor: Point;
  field: GradientField;
  threshold: number;
  viewport: PixelSize;
}): RadiusEstimate | undefined => {
  const pixel = screenToPixel(cursor, field, viewport);
  const scaleX = viewport.width / field.width;
  const scaleY = viewport.height / field.height;
  const scale = (scaleX + scaleY) / 2;
  let best: { estimate: FittedEstimate; score: number } | undefined;
  const candidates = boxes
    .flatMap((box) =>
      corners(box).map((candidate) => ({
        ...candidate,
        box,
        cornerDistance: Math.hypot(
          (candidate.x - pixel.x) * scaleX,
          (candidate.y - pixel.y) * scaleY,
        ),
      })),
    )
    .filter(({ cornerDistance }) => cornerDistance <= MAXIMUM_CURSOR_DISTANCE)
    .sort((a, b) => a.cornerDistance - b.cornerDistance)
    .slice(0, MAXIMUM_CANDIDATES);
  for (const candidate of candidates) {
    const { box } = candidate;
    const estimate = estimateCorner({
      box,
      corner: candidate.corner,
      field,
      threshold,
    });
    if (!estimate) continue;
    const arcDistance =
      cursorArcDistance({
        box,
        corner: candidate.corner,
        cursor: pixel,
        radius: estimate.radius,
      }) * scale;
    if (arcDistance > MAXIMUM_ARC_DISTANCE) continue;
    const score =
      arcDistance +
      estimate.fitScore * scale * 2 +
      (estimate.confidence === "low" ? 4 : 0);
    if (!best || score < best.score) best = { estimate, score };
  }
  if (!best) return undefined;
  const { fitScore: _fitScore, ...estimate } = best.estimate;
  return estimate;
};

/** Converts detector pixels to the ruler's zoom-invariant world coordinates. */
export const radiusEstimateToWorld = (
  estimate: RadiusEstimate,
  field: GradientField,
  viewport: PixelSize,
): RadiusMeasurement => {
  const scaleX = viewport.width / field.width;
  const scaleY = viewport.height / field.height;
  return {
    confidence: estimate.confidence,
    corner: estimate.corner,
    height: estimate.box.height * scaleY,
    radius: Math.round(estimate.radius * (scaleX + scaleY) * 0.5),
    width: estimate.box.width * scaleX,
    x: estimate.box.x * scaleX,
    y: estimate.box.y * scaleY,
  };
};
