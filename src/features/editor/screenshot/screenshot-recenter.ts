// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { sourceRect, SourceRect } from "./screenshot-geometry";
import {
  screenshotLayout,
  ScreenshotOutputSettings,
} from "./screenshot-output";
import { withScreenshotSourceCrop } from "./screenshot-output-settings";

export type ScreenshotContentBounds = {
  height: number;
  width: number;
  x: number;
  y: number;
};

export type ScreenshotRecenterAnalysis = {
  backgroundColor: string;
  bounds: ScreenshotContentBounds | null;
};

export const getScreenshotRecenterAnalysis = (
  artifactId: number,
  itemId: number,
  sourceCrop: SourceRect,
) =>
  invoke<ScreenshotRecenterAnalysis | null>("get_screenshot_content_bounds", {
    artifactId,
    itemId,
    sourceCrop,
  });

export const getRecordingRecenterAnalysis = (
  artifactId: number,
  positionMs: number,
  sourceCrop: SourceRect,
) =>
  invoke<ScreenshotRecenterAnalysis | null>("get_recording_content_bounds", {
    artifactId,
    positionMs: Math.max(0, Math.round(positionMs)),
    sourceCrop,
  });

export const recenterScreenshotContent = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
  content: ScreenshotContentBounds,
): ScreenshotOutputSettings => {
  const layout = screenshotLayout(source, settings);
  const inset = Math.max(
    0,
    Math.min(
      (layout.crop.width - layout.sourceCrop.width) / 2,
      (layout.crop.height - layout.sourceCrop.height) / 2,
    ),
  );
  const sourceCrop = sourceRect({
    height: content.height / source.height,
    width: content.width / source.width,
    x: content.x / source.width,
    y: content.y / source.height,
  });
  const contentRect = {
    height: layout.image.height * sourceCrop.height,
    width: layout.image.width * sourceCrop.width,
    x: layout.image.x + layout.image.width * sourceCrop.x,
    y: layout.image.y + layout.image.height * sourceCrop.y,
  };
  const deltaX =
    layout.crop.x +
    layout.crop.width / 2 -
    (contentRect.x + contentRect.width / 2);
  const deltaY =
    layout.crop.y +
    layout.crop.height / 2 -
    (contentRect.y + contentRect.height / 2);
  return withScreenshotSourceCrop(
    {
      ...settings,
      cropHeight: contentRect.height + inset * 2,
      cropWidth: contentRect.width + inset * 2,
      cropX: contentRect.x + deltaX - inset,
      cropY: contentRect.y + deltaY - inset,
      imageX: settings.imageX + deltaX,
      imageY: settings.imageY + deltaY,
    },
    sourceCrop,
  );
};

/**
 * The layer's crop rect grown past its source crop by `inset` output pixels on
 * every side: the padded frame the Select tool treats as the layer.
 *
 * Nothing is clamped to the canvas. A padded layer is allowed to run past the
 * frame edges exactly as an unpadded one is, and the pad is filled with the
 * colour detected behind the content rather than cut to fit.
 */
export const insetScreenshotPadding = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
  inset: number,
): ScreenshotOutputSettings => {
  const { sourceCrop } = screenshotLayout(source, settings);
  const padding = Math.max(0, inset);
  return {
    ...settings,
    cropHeight: sourceCrop.height + padding * 2,
    cropWidth: sourceCrop.width + padding * 2,
    cropX: sourceCrop.x - padding,
    cropY: sourceCrop.y - padding,
  };
};

/** The padding a layer is carrying now, in output pixels: half the difference
 * between its padded frame and the source crop inside it. */
export const screenshotPaddingInset = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
) => {
  const { crop, sourceCrop } = screenshotLayout(source, settings);
  return Math.max(0, (crop.width - sourceCrop.width) / 2);
};
