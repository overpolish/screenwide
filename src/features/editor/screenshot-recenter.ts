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

/** Remove inset while retaining the current clip/screenshot bounds. */
export const resetScreenshotRecenter = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
): ScreenshotOutputSettings => {
  const { sourceCrop } = screenshotLayout(source, settings);
  return {
    ...settings,
    cropHeight: sourceCrop.height,
    cropWidth: sourceCrop.width,
    cropX: sourceCrop.x,
    cropY: sourceCrop.y,
    recenterInsetColor: null,
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
  const { crop, sourceCrop } = screenshotLayout(source, settings);
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
    cropHeight: sourceCrop.height + inset * 2,
    cropWidth: sourceCrop.width + inset * 2,
    cropX: sourceCrop.x - inset,
    cropY: sourceCrop.y - inset,
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
}): ScreenshotOutputSettings | null => {
  // Native gesture deltas arrive as a share of the canvas.
  const output = screenshotOutputDimensions(settings);
  const moveX = deltaX * output.width;
  const moveY = deltaY * output.height;
  return operation === "move"
    ? {
        ...settings,
        cropX: settings.cropX + moveX,
        cropY: settings.cropY + moveY,
        imageX: settings.imageX + moveX,
        imageY: settings.imageY + moveY,
      }
    : operation === "resize"
      ? resizeRecenteredScreenshot({
          edges,
          scale,
          settings,
          source,
        })
      : null;
};
