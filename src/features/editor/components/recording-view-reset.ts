// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  cameraOverlayForDimensions,
  defaultCameraOverlay,
} from "../recording-export-settings";
import {
  RecordingOutputSettings,
  resetScreenshotCrop,
  resetScreenshotLayout,
  resetScreenshotTransform,
} from "../screenshot-output";
import {
  CameraOverlaySettings,
  RecordingPreviewPane,
  RecordingVideoTrackId,
} from "../types";

import type { RecordingCanvasTool } from "./recording-crop-toggle";

type RecordingViewResetInputs = {
  activeTrack: RecordingVideoTrackId | null;
  bakeCamera: boolean;
  isEnabled: boolean;
  tool: RecordingCanvasTool;
  cameraPane?: RecordingPreviewPane;
  isFrameEnabled?: boolean;
  isSelectEnabled?: boolean;
  onCameraOverlayReset?: (settings: CameraOverlaySettings) => void;
  onChange?: (
    track: RecordingVideoTrackId,
    next: RecordingOutputSettings[RecordingVideoTrackId],
  ) => void;
  outputs?: RecordingOutputSettings;
  screenPane?: RecordingPreviewPane;
};

/**
 * What resetting the view tool in hand means for a recording: the frame back
 * to the source size, the selection's transform or crop cleared, or a baked
 * camera overlay re-fitted. The toolbar no longer carries a reset; each tool's
 * panel will, and this is what it calls.
 */
export function recordingViewReset({
  activeTrack,
  bakeCamera,
  cameraPane,
  isEnabled,
  isFrameEnabled = isEnabled,
  isSelectEnabled = isEnabled,
  onCameraOverlayReset,
  onChange,
  outputs,
  screenPane,
  tool,
}: RecordingViewResetInputs) {
  const resetEnabled =
    tool === "canvas"
      ? isFrameEnabled
      : tool === "select"
        ? isSelectEnabled
        : tool === "crop" && isEnabled;
  const targetTrack = bakeCamera && tool === "canvas" ? "primary" : activeTrack;
  const canReset =
    activeTrack === "camera" && bakeCamera && tool !== "canvas"
      ? Boolean(onCameraOverlayReset)
      : Boolean(
          onChange &&
          outputs &&
          targetTrack &&
          (targetTrack === "primary" ? screenPane : cameraPane),
        );
  const reset = () => {
    if (!tool || !resetEnabled || !canReset) return;
    if (activeTrack === "camera" && bakeCamera && tool !== "canvas") {
      // The reset must be computed from the real output and camera geometry:
      // generic 16:9 defaults land the crop frame outside the camera image
      // for other aspect ratios, and the compositor's clamped rendering then
      // no longer matches the on-screen controls.
      onCameraOverlayReset?.(
        outputs && cameraPane
          ? cameraOverlayForDimensions({
              cameraHeight: cameraPane.sourceHeight,
              cameraWidth: cameraPane.sourceWidth,
              screenHeight: outputs.primary.height,
              screenWidth: outputs.primary.width,
            })
          : defaultCameraOverlay(),
      );
      return;
    }
    if (!activeTrack || !outputs) return;
    const targetTrack =
      bakeCamera && tool === "canvas" ? "primary" : activeTrack;
    const pane = targetTrack === "primary" ? screenPane : cameraPane;
    if (!pane) return;
    const source = { height: pane.sourceHeight, width: pane.sourceWidth };
    const current = outputs[targetTrack];
    const next =
      tool === "canvas"
        ? resetScreenshotLayout(
            { ...current, height: source.height, width: source.width },
            source,
          )
        : tool === "select"
          ? resetScreenshotTransform(current, source)
          : resetScreenshotCrop(current, source);
    onChange?.(targetTrack, next);
  };
  return { canReset: resetEnabled && canReset, reset };
}
