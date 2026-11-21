// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  canvasValue,
  CaptureSource,
  isSameMeasure,
  snappedCorners,
  snappedInset,
} from "../layer-measure";

import { insetScreenshotPadding } from "./screenshot-inset";
import {
  normalizedScreenshotOutput,
  ScreenshotOutputSettings,
} from "./screenshot-output-settings";

// Settings saved before a layer's padding and corners were shares of the
// canvas hold them as a share of the frame or as points of the capture. A
// project or a remembered look is read once as it opens, and each such
// value becomes the look it was nearest, so the island shows a look picked
// before as that look and nothing else has to read the old units.

/** Which looks a layer's corners are offered: a camera's include a
 * circle. */
type CornerLooks = { circle: boolean };

/** A remembered look as a template: its padding and corners snapped to
 * looks, nothing laid out by them, since a template is laid out again for
 * the capture it is given to. */
export const snappedTemplate = (
  saved: ScreenshotOutputSettings,
  looks: CornerLooks,
): ScreenshotOutputSettings => {
  const settings = normalizedScreenshotOutput(saved);
  return {
    ...settings,
    ...(settings.inset ? { inset: snappedInset(settings.inset) } : {}),
    radius: snappedCorners(settings.radius, looks),
  };
};

/** A layer as a project saved it, laid out for `source`: its corners and
 * padding snapped to the looks they were nearest, the frame grown by the
 * padding's look again. A padding saved before it was kept at all is still
 * read from the frame (`screenshotInset`), which is left as it was. */
export const withSnappedMeasures = (
  saved: ScreenshotOutputSettings,
  source: CaptureSource,
  looks: CornerLooks,
): ScreenshotOutputSettings => {
  const settings = normalizedScreenshotOutput(saved);
  const withRadius = {
    ...settings,
    radius: snappedCorners(settings.radius, looks),
  };
  if (!settings.inset) return withRadius;
  const inset = snappedInset(settings.inset);
  return isSameMeasure(inset, settings.inset)
    ? withRadius
    : insetScreenshotPadding(withRadius, source, canvasValue(inset) ?? 0);
};
