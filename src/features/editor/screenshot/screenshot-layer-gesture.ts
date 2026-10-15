// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { withRotation } from "../layer-rotation";

import {
  applyScreenshotCropGesture,
  commitScreenshotCrop,
  isCropOperation,
} from "./screenshot-crop";
import {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
  screenshotGestureShift,
} from "./screenshot-output";

import type { ScreenshotSelectionGestureEvent } from "./screenshot-preview-events";

/**
 * Where one gesture frame leaves a screenshot layer, measured against the
 * `snapshot` the gesture began from: a crop, a corner radius, a turn, a
 * resize or a move. `anchor` is the Begin deltas, which a crop draw starts
 * from; `output` is the canvas's size, the crop's frame of reference; the
 * move travels the `workspace` canvas the gesture began on.
 */
export function gesturedScreenshotLayer({
  anchor,
  event,
  output,
  snapshot,
  source,
  workspace,
}: {
  anchor: { x: number; y: number };
  event: ScreenshotSelectionGestureEvent;
  output: { height: number; width: number };
  snapshot: ScreenshotOutputSettings;
  source: { height: number; width: number };
  workspace: ScreenshotWorkspaceOutputSettings;
}): ScreenshotOutputSettings {
  if (isCropOperation(event.operation)) {
    const next = applyScreenshotCropGesture({
      ...event,
      anchor,
      operation: event.operation,
      output,
      settings: snapshot,
      source,
    });
    return event.phase === "end"
      ? commitScreenshotCrop(snapshot, next, source)
      : next;
  }
  if (event.operation === "radius")
    return {
      ...snapshot,
      radiusPercent: Math.min(50, Math.max(0, event.scale)),
    };
  // The gizmo reports the turn it reaches: about X and Y as its travel, about
  // Z as its scale.
  if (event.operation === "rotate")
    return withRotation(snapshot, {
      x: event.deltaX,
      y: event.deltaY,
      z: event.scale,
    });
  const move = screenshotGestureShift(event, workspace);
  const cropX = snapshot.cropX + move.x;
  const cropY = snapshot.cropY + move.y;
  if (event.operation === "resize") {
    const scale = Math.min(8, Math.max(0, event.scale));
    const transform = (
      value: number,
      startFrame: number,
      nextFrame: number,
    ) => {
      if (Math.abs(scale - 1) < 1e-9) return value;
      const anchorAt = (nextFrame - startFrame * scale) / (1 - scale);
      return anchorAt + (value - anchorAt) * scale;
    };
    return {
      ...snapshot,
      cropHeight: snapshot.cropHeight * scale,
      cropWidth: snapshot.cropWidth * scale,
      cropX,
      cropY,
      imageWidth: snapshot.imageWidth * scale,
      imageX: transform(snapshot.imageX, snapshot.cropX, cropX),
      imageY: transform(snapshot.imageY, snapshot.cropY, cropY),
    };
  }
  return {
    ...snapshot,
    cropX,
    cropY,
    imageX: snapshot.imageX + move.x,
    imageY: snapshot.imageY + move.y,
  };
}
