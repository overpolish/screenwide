// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { scaledCameraOverlay } from "../camera-overlay-placement";
import {
  applyScreenshotCropGesture,
  commitScreenshotCrop,
} from "../screenshot-crop";
import {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
  resizeScreenshotWorkspaceCanvasEdges,
  screenshotOutputDimensions,
  screenshotWorkspaceItemOutput,
} from "../screenshot-output";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../types";

import type { RecordingSelectionGestureEvent } from "../use-recording-preview-surface";

/**
 * Where one gesture frame leaves the baked camera overlay, measured against
 * the snapshot the gesture began from.
 */
export function gesturedCameraOverlay({
  event,
  primaryOutput,
  start,
}: {
  event: RecordingSelectionGestureEvent;
  primaryOutput: ScreenshotOutputSettings;
  start: CameraOverlaySettings;
}) {
  // A baked overlay is placed in the screen output's own pixels, and the
  // native gesture reports its deltas as a share of that canvas.
  const canvas = screenshotOutputDimensions(primaryOutput);
  const moveX = event.deltaX * canvas.width;
  const moveY = event.deltaY * canvas.height;
  const frameX = start.frameX + moveX;
  const frameY = start.frameY + moveY;
  let next: CameraOverlaySettings;
  if (event.operation === "cropMove") {
    next = {
      ...start,
      frameX,
      frameY,
    };
  } else if (event.operation === "cropResize") {
    let left = start.frameX;
    let top = start.frameY;
    let right = left + start.frameWidth;
    let bottom = top + start.frameHeight;
    if ((event.edges & 1) !== 0) left += moveX;
    if ((event.edges & 2) !== 0) right += moveX;
    if ((event.edges & 4) !== 0) top += moveY;
    if ((event.edges & 8) !== 0) bottom += moveY;
    next = {
      ...start,
      frameHeight: bottom - top,
      frameWidth: right - left,
      frameX: left,
      frameY: top,
    };
  } else if (event.operation === "radius") {
    next = {
      ...start,
      radiusPercent: Math.min(50, Math.max(0, event.scale)),
    };
  } else if (event.operation === "resize") {
    next = scaledCameraOverlay(start, Math.min(8, Math.max(0, event.scale)), {
      x: frameX,
      y: frameY,
    });
  } else {
    next = {
      ...start,
      cameraX: start.cameraX + moveX,
      cameraY: start.cameraY + moveY,
      frameX,
      frameY,
    };
  }
  return next;
}

/**
 * Where one gesture frame leaves an ordinary pane's output, measured against
 * the snapshot the gesture began from. Null means the gesture has no source to
 * crop against and the frame is dropped.
 */
export function gesturedTrackOutput({
  event,
  previewSourceDimensions,
  snapshot,
  trackId,
}: {
  event: RecordingSelectionGestureEvent;
  previewSourceDimensions: Partial<
    Record<RecordingVideoTrackId, { height: number; width: number }>
  >;
  snapshot: RecordingOutputSettings[RecordingVideoTrackId];
  trackId: RecordingVideoTrackId;
}) {
  // Native gesture deltas arrive as a share of the layer's own canvas.
  const canvas = screenshotOutputDimensions(snapshot);
  const moveX = event.deltaX * canvas.width;
  const moveY = event.deltaY * canvas.height;
  const cropX = snapshot.cropX + moveX;
  const cropY = snapshot.cropY + moveY;
  let next: RecordingOutputSettings[RecordingVideoTrackId];
  if (event.operation === "cropMove" || event.operation === "cropResize") {
    const source =
      trackId === "primary"
        ? previewSourceDimensions.primary
        : previewSourceDimensions.camera;
    if (!source) return null;
    next = applyScreenshotCropGesture({
      ...event,
      operation: event.operation,
      output: screenshotOutputDimensions(snapshot),
      settings: snapshot,
      source,
    });
    if (event.phase === "end")
      next = commitScreenshotCrop(snapshot, next, source);
  } else if (event.operation === "frameResize") {
    const workspace = {
      ...snapshot,
      items: [{ id: 0, output: snapshot }],
    };
    const resized = resizeScreenshotWorkspaceCanvasEdges({
      deltaX: event.deltaX,
      deltaY: event.deltaY,
      edges: event.edges,
      settings: workspace,
    });
    next = screenshotWorkspaceItemOutput(resized, 0);
  } else if (event.operation === "radius") {
    next = {
      ...snapshot,
      radiusPercent: Math.min(50, Math.max(0, event.scale)),
    };
  } else if (event.operation === "resize") {
    const scale = Math.min(8, Math.max(0, event.scale));
    const transform = (
      value: number,
      startFrame: number,
      nextFrame: number,
    ) => {
      if (Math.abs(scale - 1) < 1e-9) return value;
      const anchor = (nextFrame - startFrame * scale) / (1 - scale);
      return anchor + (value - anchor) * scale;
    };
    next = {
      ...snapshot,
      cropHeight: snapshot.cropHeight * scale,
      cropWidth: snapshot.cropWidth * scale,
      cropX,
      cropY,
      imageWidth: snapshot.imageWidth * scale,
      imageX: transform(snapshot.imageX, snapshot.cropX, cropX),
      imageY: transform(snapshot.imageY, snapshot.cropY, cropY),
    };
  } else {
    next = {
      ...snapshot,
      cropX,
      cropY,
      imageX: snapshot.imageX + moveX,
      imageY: snapshot.imageY + moveY,
    };
  }
  return next;
}
