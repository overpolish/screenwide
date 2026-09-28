// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  fullSourceRect,
  sourceRect,
  SourceRect,
  translateSourceRect,
} from "./screenshot-geometry";
import {
  ScreenshotOutputSettings,
  screenshotLayout,
} from "./screenshot-output";
import {
  screenshotSourceCrop,
  withScreenshotSourceCrop,
} from "./screenshot-output-settings";

export type CropOperation = "cropDraw" | "cropMove" | "cropResize";

export const isCropOperation = (
  operation: string,
): operation is CropOperation =>
  operation === "cropDraw" ||
  operation === "cropMove" ||
  operation === "cropResize";

const CENTERED_EDGE = 1 << 16;

/**
 * The window a crop draw sample describes, in the units of its anchor and
 * deltas. The anchor is a corner of the window, or its centre when the
 * sample is centered; the deltas reach the far corner or the half extents.
 */
export const drawnCropRect = (
  anchor: { x: number; y: number },
  sample: { deltaX: number; deltaY: number; edges: number },
) => {
  const centered = (sample.edges & CENTERED_EDGE) !== 0;
  const span = (start: number, delta: number) =>
    centered
      ? { length: 2 * Math.abs(delta), start: start - Math.abs(delta) }
      : { length: Math.abs(delta), start: Math.min(start, start + delta) };
  const x = span(anchor.x, sample.deltaX);
  const y = span(anchor.y, sample.deltaY);
  return { height: y.length, width: x.length, x: x.start, y: y.start };
};

type CropGesture = {
  deltaX: number;
  deltaY: number;
  edges: number;
  operation: CropOperation;
  output: { height: number; width: number };
  settings: ScreenshotOutputSettings;
  source: { height: number; width: number };
  /** A crop draw's anchor: its Begin deltas, a share of the canvas from the
   * crop's origin. */
  anchor?: { x: number; y: number };
};

export const applyScreenshotCropGesture = ({
  anchor = { x: 0, y: 0 },
  deltaX,
  deltaY,
  edges,
  operation,
  output,
  settings,
  source,
}: CropGesture): ScreenshotOutputSettings => {
  const image = screenshotLayout(source, settings).image;
  const toSourceX = output.width / image.width;
  const toSourceY = output.height / image.height;
  const sourceDeltaX = deltaX * toSourceX;
  const sourceDeltaY = deltaY * toSourceY;
  const current = screenshotSourceCrop(settings);
  let next: SourceRect;
  if (operation === "cropDraw") {
    const drawn = drawnCropRect(
      {
        x: current.x + anchor.x * toSourceX,
        y: current.y + anchor.y * toSourceY,
      },
      { deltaX: sourceDeltaX, deltaY: sourceDeltaY, edges },
    );
    return withScreenshotSourceCrop(settings, sourceRect(drawn));
  }
  if (operation === "cropMove") {
    next = translateSourceRect(current, { x: sourceDeltaX, y: sourceDeltaY });
    return withScreenshotSourceCrop(settings, next);
  }
  let left = current.x;
  let top = current.y;
  let right = left + current.width;
  let bottom = top + current.height;
  if ((edges & 1) !== 0) left += sourceDeltaX;
  if ((edges & 2) !== 0) right += sourceDeltaX;
  if ((edges & 4) !== 0) top += sourceDeltaY;
  if ((edges & 8) !== 0) bottom += sourceDeltaY;
  next = sourceRect({
    height: bottom - top,
    width: right - left,
    x: left,
    y: top,
  });
  return withScreenshotSourceCrop(settings, next);
};

/** Rebase the outer inset frame by the crop committed on each source edge. */
export const commitScreenshotCrop = (
  before: ScreenshotOutputSettings,
  after: ScreenshotOutputSettings,
  source: { height: number; width: number },
): ScreenshotOutputSettings => {
  const previous = screenshotLayout(source, before);
  const next = screenshotLayout(source, after);
  const left = previous.crop.x + next.sourceCrop.x - previous.sourceCrop.x;
  const top = previous.crop.y + next.sourceCrop.y - previous.sourceCrop.y;
  const right =
    previous.crop.x +
    previous.crop.width -
    (previous.sourceCrop.x +
      previous.sourceCrop.width -
      next.sourceCrop.x -
      next.sourceCrop.width);
  const bottom =
    previous.crop.y +
    previous.crop.height -
    (previous.sourceCrop.y +
      previous.sourceCrop.height -
      next.sourceCrop.y -
      next.sourceCrop.height);
  return {
    ...after,
    cropHeight: bottom - top,
    cropWidth: right - left,
    cropX: left,
    cropY: top,
  };
};

export const resetCommittedScreenshotCrop = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
): ScreenshotOutputSettings =>
  commitScreenshotCrop(
    settings,
    withScreenshotSourceCrop(
      { ...settings, radiusPercent: 0 },
      fullSourceRect(),
    ),
    source,
  );

/**
 * Show the full source while the OSC marks the committed screenshot crop.
 *
 * The whole source becomes the visible rectangle so nothing being cropped away
 * disappears while the tool is open, and `cropPreview` keeps the committed crop
 * so the compositor can draw the layer the crop actually produces over that
 * ghost, rounded and shadowed. Both rectangles are in the same output pixels:
 * the source crop only moves the window, never the image behind it.
 */
export const uncroppedScreenshotPreviewOutput = (
  source: { height: number; width: number },
  settings: ScreenshotOutputSettings,
): ScreenshotOutputSettings => {
  const cropPreview = screenshotLayout(source, settings).sourceCrop;
  const previewSettings = withScreenshotSourceCrop(settings, fullSourceRect());
  const { image } = screenshotLayout(source, previewSettings);
  return {
    ...previewSettings,
    cropHeight: image.height,
    cropPreview,
    cropWidth: image.width,
    cropX: image.x,
    cropY: image.y,
  };
};
