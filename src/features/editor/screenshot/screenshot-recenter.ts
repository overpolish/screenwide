// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { sourceRect, SourceRect } from "./screenshot-geometry";
import { screenshotInsetShare } from "./screenshot-inset";
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
  const share = screenshotInsetShare(settings, source);
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
  // The padding is the content's share, so it keeps its proportion to the
  // picture it now frames.
  const inset =
    share * Math.max(1, Math.min(contentRect.width, contentRect.height));
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
      inset: share,
    },
    sourceCrop,
  );
};
