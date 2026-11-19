// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  CaptureSource,
  Measure,
  percent,
  snappedCorners,
  snappedInset,
} from "../layer-measure";

import {
  insetScreenshotPadding,
  screenshotInsetPoints,
} from "./screenshot-inset";
import {
  normalizedScreenshotOutput,
  ScreenshotOutputSettings,
} from "./screenshot-output-settings";

// Settings saved before a layer's padding and corners were kept in points
// hold them as percents. A project or a remembered look is read once as it
// opens, and each such value becomes the look in points it was nearest, so
// the island shows a look picked before as that look and nothing else has
// to read percents for a capture.

/** Corners in the unit the layer takes them in: a capture's in points, a
 * percent snapped to its look; a camera's as a percent, square where it
 * arrives in points from a capture's remembered look. */
export const cornersIn = (
  radius: Measure,
  unit: "percent" | "points",
): Measure =>
  unit === "points"
    ? snappedCorners(radius)
    : "percent" in radius
      ? radius
      : percent(0);

/** A remembered look as a template: its padding and corners in points,
 * nothing laid out by them, since a template is laid out again for the
 * capture it is given to. */
export const snappedTemplate = (
  saved: ScreenshotOutputSettings,
  corners: "percent" | "points",
): ScreenshotOutputSettings => {
  const settings = normalizedScreenshotOutput(saved);
  return {
    ...settings,
    ...(settings.inset ? { inset: snappedInset(settings.inset) } : {}),
    radius: cornersIn(settings.radius, corners),
  };
};

/** A layer as a project saved it, laid out for `source`: its corners in
 * the unit it takes them in, and a padding saved as a percent snapped to
 * the look it was nearest, the frame grown by that look again. A padding
 * saved before it was kept at all is still read from the frame
 * (`screenshotInsetPoints`), which is left as it was. */
export const withSnappedMeasures = (
  saved: ScreenshotOutputSettings,
  source: CaptureSource,
  corners: "percent" | "points",
): ScreenshotOutputSettings => {
  const settings = normalizedScreenshotOutput(saved);
  const radius = cornersIn(settings.radius, corners);
  if (!settings.inset || "points" in settings.inset)
    return { ...settings, radius };
  const withRadius = { ...settings, radius };
  const inset = screenshotInsetPoints(withRadius, source);
  return settings.inset.percent > 0 || inset > 0
    ? insetScreenshotPadding(withRadius, source, inset)
    : withRadius;
};
