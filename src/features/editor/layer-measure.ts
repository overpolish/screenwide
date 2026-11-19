// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Measure } from "../../bindings/Measure";

/** A capture's size in pixels and how many of them span a point: what a
 * length in points is drawn against. */
export type CaptureSource = {
  height: number;
  scaleFactor: number;
  width: number;
};

export type { Measure };

// A layer's padding and corners are points of its capture, as its
// annotations are, so a look pads and rounds a short strip as much as a
// full screen and matches the corners of the windows in it. A camera's
// corners are a share of its frame instead, so one drawn as a circle stays
// one whatever its crop. Settings saved before lengths had a unit held a
// bare share; they are snapped to the look in points they stood for when a
// project opens (`withSnappedMeasures`).

/** The padding looks, None, Tight, Medium and Large, in points. */
export const INSET_LOOKS = [0, 16, 32, 64] as const;

/** The corner looks a capture is offered, square, soft and round, in
 * points. */
export const CORNER_LOOKS = [0, 10, 24] as const;

/** The same looks as they were kept before, in percent of the picture's or
 * frame's shorter side, in the order of the looks they became. */
const LEGACY_INSET_PERCENTS = [0, 3, 6, 10] as const;
const LEGACY_CORNER_PERCENTS = [0, 4, 12] as const;

export const points = (value: number): Measure => ({ points: value });
export const percent = (value: number): Measure => ({ percent: value });

export const unitOf = (measure: Measure): "percent" | "points" =>
  "points" in measure ? "points" : "percent";

/** `value` in the unit `reference` is kept in, a percent held to half the
 * frame. */
export const measureLike = (reference: Measure, value: number): Measure =>
  "points" in reference
    ? points(Math.max(0, value))
    : percent(Math.min(50, Math.max(0, value)));

/** The length in `unit`, or null for one kept in the other. Nothing is
 * nothing in either. */
export const measureIn = (
  measure: Measure | undefined,
  unit: "percent" | "points",
): number | null => {
  if (!measure) return null;
  const value = "points" in measure ? measure.points : measure.percent;
  if (value <= 0) return 0;
  return unit in measure ? value : null;
};

/** Canvas pixels per point of `source` drawn `imageWidth` canvas pixels
 * wide. */
export const pixelsPerPoint = (imageWidth: number, source: CaptureSource) =>
  (Math.max(0, imageWidth) * Math.max(source.scaleFactor, 1e-6)) /
  Math.max(1, source.width);

/** `measure` in canvas pixels on a frame whose shorter side is `shortest`
 * canvas pixels. */
export const measurePixels = (
  measure: Measure,
  shortest: number,
  perPoint: number,
) =>
  Math.max(
    0,
    "points" in measure
      ? measure.points * perPoint
      : (Math.max(0, shortest) * measure.percent) / 100,
  );

/** As a corner radius: never more than half the shorter side. */
const radiusPixels = (measure: Measure, shortest: number, perPoint: number) =>
  Math.min(
    measurePixels(measure, shortest, perPoint),
    Math.max(0, shortest) / 2,
  );

export const isZeroMeasure = (measure: Measure) =>
  ("points" in measure ? measure.points : measure.percent) <= 0;

/** `share` of the way from one length to another. Lengths in different
 * units cannot be mixed, so the change happens halfway. */
export const measureBetween = (
  from: Measure,
  to: Measure,
  share: number,
): Measure => {
  if ("points" in from && "points" in to)
    return points(from.points + (to.points - from.points) * share);
  if ("percent" in from && "percent" in to)
    return percent(from.percent + (to.percent - from.percent) * share);
  return share < 0.5 ? from : to;
};

/** A layer's corner radius as a percent of its frame's shorter side, as
 * the selection's outline is rounded in the view. */
export const cornerRadiusFramePercent = (
  settings: {
    cropHeight: number;
    cropWidth: number;
    imageWidth: number;
    radius: Measure;
  },
  source: CaptureSource,
) => {
  const shortest = Math.min(settings.cropWidth, settings.cropHeight);
  if (shortest <= 0) return 0;
  const radius = radiusPixels(
    settings.radius,
    shortest,
    pixelsPerPoint(settings.imageWidth, source),
  );
  return (radius / shortest) * 100;
};

const nearestLook = (
  value: number,
  legacy: readonly number[],
  looks: readonly number[],
) => {
  let best = 0;
  legacy.forEach((candidate, index) => {
    if (Math.abs(candidate - value) < Math.abs(legacy[best] - value))
      best = index;
  });
  return looks[best];
};

/** A padding kept as a percent, as the look in points it was nearest. */
export const snappedInset = (measure: Measure): Measure =>
  "points" in measure
    ? measure
    : points(nearestLook(measure.percent, LEGACY_INSET_PERCENTS, INSET_LOOKS));

/** A capture's corners kept as a percent, as the look in points they were
 * nearest. */
export const snappedCorners = (measure: Measure): Measure =>
  "points" in measure
    ? measure
    : points(
        nearestLook(measure.percent, LEGACY_CORNER_PERCENTS, CORNER_LOOKS),
      );

/** A length as saved before lengths had a unit, a bare share or percent,
 * read as a percent; a length with a unit as it is. */
export const savedMeasure = (
  saved: unknown,
  bareIs: "percent" | "share",
): Measure | undefined => {
  if (typeof saved === "number" && Number.isFinite(saved))
    return percent(bareIs === "share" ? saved * 100 : saved);
  if (typeof saved !== "object" || saved === null) return undefined;
  const finiteNumber = (value: unknown): value is number =>
    typeof value === "number" && Number.isFinite(value);
  if ("points" in saved && finiteNumber(saved.points))
    return points(saved.points);
  if ("percent" in saved && finiteNumber(saved.percent))
    return percent(saved.percent);
  return undefined;
};
