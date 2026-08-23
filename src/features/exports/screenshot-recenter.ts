// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { sourceRect, SourceRect } from "./screenshot-geometry";
import {
  screenshotLayout,
  screenshotOutputDimensions,
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

export const recenterScreenshotContent = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
  content: ScreenshotContentBounds,
): ScreenshotOutputSettings => {
  const output = screenshotOutputDimensions(settings);
  const layout = screenshotLayout(source, output, settings);
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
      screenshotCropHeightPercent: (contentRect.height * 100) / output.height,
      screenshotCropWidthPercent: (contentRect.width * 100) / output.width,
      screenshotCropXPercent: ((contentRect.x + deltaX) * 100) / output.width,
      screenshotCropYPercent: ((contentRect.y + deltaY) * 100) / output.height,
      screenshotImageXPercent:
        settings.screenshotImageXPercent + (deltaX * 100) / output.width,
      screenshotImageYPercent:
        settings.screenshotImageYPercent + (deltaY * 100) / output.height,
    },
    sourceCrop,
  );
};

/** Remove inset while retaining the current clip/screenshot bounds. */
export const resetScreenshotRecenter = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
): ScreenshotOutputSettings => {
  const output = screenshotOutputDimensions(settings);
  const { sourceCrop } = screenshotLayout(source, output, settings);
  return {
    ...settings,
    recenterInsetColor: null,
    screenshotCropHeightPercent: (sourceCrop.height * 100) / output.height,
    screenshotCropWidthPercent: (sourceCrop.width * 100) / output.width,
    screenshotCropXPercent: (sourceCrop.x * 100) / output.width,
    screenshotCropYPercent: (sourceCrop.y * 100) / output.height,
  };
};

const resizeRecenteredScreenshot = ({
  edges,
  scale: requestedScale,
  settings,
  source,
}: {
  edges: number;
  scale: number;
  settings: ScreenshotOutputSettings;
  source?: { height: number; width: number };
}): ScreenshotOutputSettings => {
  const output = screenshotOutputDimensions(settings);
  if (!source) return settings;
  const { crop, sourceCrop } = screenshotLayout(source, output, settings);
  const scale = Math.max(0, requestedScale);
  const verticalOnly = (edges & (4 | 8)) !== 0 && (edges & (1 | 2)) === 0;
  const sourceSize = verticalOnly ? sourceCrop.height : sourceCrop.width;
  const cropSize = verticalOnly ? crop.height : crop.width;
  const requestedInset = (cropSize * scale - sourceSize) / 2;
  const maximumInset = Math.max(
    0,
    Math.min(
      sourceCrop.x,
      sourceCrop.y,
      output.width - sourceCrop.x - sourceCrop.width,
      output.height - sourceCrop.y - sourceCrop.height,
    ),
  );
  const inset = Math.min(maximumInset, Math.max(0, requestedInset));
  return {
    ...settings,
    screenshotCropHeightPercent:
      ((sourceCrop.height + inset * 2) * 100) / output.height,
    screenshotCropWidthPercent:
      ((sourceCrop.width + inset * 2) * 100) / output.width,
    screenshotCropXPercent: ((sourceCrop.x - inset) * 100) / output.width,
    screenshotCropYPercent: ((sourceCrop.y - inset) * 100) / output.height,
  };
};

export const applyScreenshotRecenterGesture = ({
  deltaX,
  deltaY,
  edges,
  operation,
  scale,
  settings,
  source,
}: {
  deltaX: number;
  deltaY: number;
  edges: number;
  operation: string;
  scale: number;
  settings: ScreenshotOutputSettings;
  source?: { height: number; width: number };
}): ScreenshotOutputSettings | null =>
  operation === "move"
    ? {
        ...settings,
        screenshotCropXPercent: settings.screenshotCropXPercent + deltaX * 100,
        screenshotCropYPercent: settings.screenshotCropYPercent + deltaY * 100,
        screenshotImageXPercent:
          settings.screenshotImageXPercent + deltaX * 100,
        screenshotImageYPercent:
          settings.screenshotImageYPercent + deltaY * 100,
      }
    : operation === "resize"
      ? resizeRecenteredScreenshot({
          edges,
          scale,
          settings,
          source,
        })
      : null;
