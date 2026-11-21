// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { Measure } from "../../bindings/Measure";

/** A capture's size in pixels and how many of them span a point. */
export type CaptureSource = {
  height: number;
  scaleFactor: number;
  width: number;
};

export type { Measure };

// A layer's padding and corners are shares of the canvas's shorter side, as
// `screenwide_scenes::Measure` draws them, so a look pads and rounds a full
// display and a small region alike, and a screen and a camera given the
// same look have the same corners however large each is drawn. A camera
// drawn as a circle is half its own frame's shorter side, so it stays one
// whatever its crop. Settings saved before hold a share of the frame, or
// points of the capture read as pixels of a 1080 high canvas; a project
// opening snaps them to the look they were nearest (`withSnappedMeasures`).

/** The padding looks, None, Tight, Medium and Large, in percent of the
 * canvas's shorter side. */
export const INSET_LOOKS = [0, 1.5, 3, 6] as const;

/** The corner looks every layer is offered, square, rounded and round, in
 * percent of the canvas's shorter side. */
export const CORNER_LOOKS = [0, 1, 3] as const;

/** A camera's circle, in percent of its frame's shorter side. */
export const CIRCLE_PERCENT = 50;

/** A fresh project's camera corners: round, as the roundest of the looks
 * the screen is offered. */
export const DEFAULT_CAMERA_RADIUS: Measure = { canvas: CORNER_LOOKS[2] };

/** The looks as they were kept as a share of the frame, in percent of its
 * shorter side, in the order of the looks they became. */
const LEGACY_INSET_PERCENTS = [0, 3, 6, 10] as const;
const LEGACY_CORNER_PERCENTS = [0, 4, 12] as const;

/** The canvas height a saved point was drawn on, near enough: a full
 * display drawn on a 1080 high canvas is about a pixel per point. */
const LEGACY_POINTS_CANVAS = 1080;

export const canvasShare = (value: number): Measure => ({ canvas: value });
export const percent = (value: number): Measure => ({ percent: value });

const valueOf = (measure: Measure) =>
  "canvas" in measure ? measure.canvas : measure.percent;

export const isSameMeasure = (a: Measure, b: Measure) =>
  "canvas" in a === "canvas" in b && Math.abs(valueOf(a) - valueOf(b)) < 1e-9;

/** The length's share of the canvas, or null for one kept as a share of
 * the frame. */
export const canvasValue = (measure: Measure): number | null =>
  "canvas" in measure ? measure.canvas : null;

/** The canvas's shorter side, in its pixels. */
export const canvasShortest = (canvas: { height: number; width: number }) =>
  Math.max(0, Math.min(canvas.width, canvas.height));

/** `measure` in canvas pixels on a frame whose shorter side is `frame`
 * canvas pixels, on a canvas whose shorter side is `canvas`. */
export const measurePixels = (
  measure: Measure,
  frame: number,
  canvas: number,
) =>
  Math.max(
    0,
    "canvas" in measure
      ? (Math.max(0, canvas) * measure.canvas) / 100
      : (Math.max(0, frame) * measure.percent) / 100,
  );

/** As a corner radius: never more than half the frame's shorter side. */
const radiusPixels = (measure: Measure, frame: number, canvas: number) =>
  Math.min(measurePixels(measure, frame, canvas), Math.max(0, frame) / 2);

/** As a corner radius in percent of the frame's shorter side, as a
 * selection's outline and a layout's drawing are rounded. */
export const radiusFramePercent = (
  measure: Measure,
  frame: number,
  canvas: number,
) => (frame > 0 ? (radiusPixels(measure, frame, canvas) / frame) * 100 : 0);

export const isZeroMeasure = (measure: Measure) => valueOf(measure) <= 0;

/** `share` of the way from one length to another. Lengths in different
 * units cannot be mixed, so the change happens halfway. */
export const measureBetween = (
  from: Measure,
  to: Measure,
  share: number,
): Measure => {
  if ("canvas" in from && "canvas" in to)
    return canvasShare(from.canvas + (to.canvas - from.canvas) * share);
  if ("percent" in from && "percent" in to)
    return percent(from.percent + (to.percent - from.percent) * share);
  return share < 0.5 ? from : to;
};

/** `share` of the way from one corner radius to another, `from` drawn on a
 * frame whose shorter side is `frame` canvas pixels, on a canvas whose
 * shorter side is `canvas`. A radius in the other unit is first measured in
 * `to`'s as it is drawn, so a change to or from a circle eases as the looks
 * do. */
export const radiusBetween = (
  from: Measure,
  to: Measure,
  { canvas, frame, share }: { canvas: number; frame: number; share: number },
): Measure => {
  if ("canvas" in from === "canvas" in to)
    return measureBetween(from, to, share);
  const drawn = radiusPixels(from, frame, canvas);
  const start =
    "canvas" in to
      ? canvasShare(canvas > 0 ? (drawn / canvas) * 100 : 0)
      : percent(frame > 0 ? (drawn / frame) * 100 : 0);
  return measureBetween(start, to, share);
};

/** A layer's corner radius as a percent of its frame's shorter side, as
 * the selection's outline is rounded in the view. */
export const cornerRadiusFramePercent = (settings: {
  cropHeight: number;
  cropWidth: number;
  height: number;
  radius: Measure;
  width: number;
}) =>
  radiusFramePercent(
    settings.radius,
    Math.min(settings.cropWidth, settings.cropHeight),
    canvasShortest(settings),
  );

const nearest = (value: number, candidates: readonly number[]) =>
  candidates.reduce(
    (best, candidate, index) =>
      Math.abs(candidate - value) < Math.abs(candidates[best] - value)
        ? index
        : best,
    0,
  );

/** A padding as the look it was nearest. */
export const snappedInset = (measure: Measure): Measure =>
  canvasShare(
    INSET_LOOKS[
      "canvas" in measure
        ? nearest(measure.canvas, INSET_LOOKS)
        : nearest(measure.percent, LEGACY_INSET_PERCENTS)
    ],
  );

/** Corners as the look they were nearest; a camera's circle stays one. */
export const snappedCorners = (
  measure: Measure,
  { circle }: { circle: boolean },
): Measure => {
  if (circle && "percent" in measure && measure.percent >= CIRCLE_PERCENT)
    return percent(CIRCLE_PERCENT);
  return canvasShare(
    CORNER_LOOKS[
      "canvas" in measure
        ? nearest(measure.canvas, CORNER_LOOKS)
        : nearest(measure.percent, LEGACY_CORNER_PERCENTS)
    ],
  );
};

/** A length as saved before lengths had a unit, a bare share or percent of
 * the frame, read as a percent; one saved in points as a share of a 1080
 * high canvas; a length with a unit as it is. */
export const savedMeasure = (
  saved: unknown,
  bareIs: "percent" | "share",
): Measure | undefined => {
  if (typeof saved === "number" && Number.isFinite(saved))
    return percent(bareIs === "share" ? saved * 100 : saved);
  if (typeof saved !== "object" || saved === null) return undefined;
  const finiteNumber = (value: unknown): value is number =>
    typeof value === "number" && Number.isFinite(value);
  if ("canvas" in saved && finiteNumber(saved.canvas))
    return canvasShare(saved.canvas);
  if ("percent" in saved && finiteNumber(saved.percent))
    return percent(saved.percent);
  if ("points" in saved && finiteNumber(saved.points))
    return canvasShare((saved.points / LEGACY_POINTS_CANVAS) * 100);
  return undefined;
};
