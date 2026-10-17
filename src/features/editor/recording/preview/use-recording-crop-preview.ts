// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { withShareOfTurn } from "../../layer-rotation";
import { uncroppedCameraPreviewOverlay } from "../../preview/camera-overlay-geometry";
import { croppingScreenshotPreviewOutput } from "../../screenshot/screenshot-crop";
import { RecordingOutputSettings } from "../../screenshot/screenshot-output";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";

import type { ScrubPreviewProps } from "./scrub-preview";
import type { CropTurn } from "../../preview/use-crop-turn";

/** What the player draws while a crop is in hand. The framed track is shown
 * whole - and a baked camera in the place its overlay holds - so the handles
 * sit on the picture the crop is taken out of. A turned screen is shown so
 * once the selection's `cropTurn` stands it upright, its turn eased on the
 * way there and back. Inside a scene the crop tool pans and zooms the picture
 * within its box instead, so nothing is shown whole: the scene would arrange
 * the whole picture as if it were the crop. The scene's own turn of the
 * screen eases in the native preview. */
export function useRecordingCropPreview({
  activeVideoTrack,
  bakeCamera,
  cameraOverlay,
  canvasTool,
  effectiveRecordingOutput,
  previewSourceDimensions,
  selection: {
    cropTurn: { isFlat, share },
    isFramed,
  },
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  canvasTool: RecordingCanvasTool;
  effectiveRecordingOutput: RecordingOutputSettings;
  previewSourceDimensions: ScrubPreviewProps["previewSourceDimensions"];
  selection: { cropTurn: CropTurn; isFramed: boolean };
}) {
  const cropSource =
    activeVideoTrack === "primary"
      ? previewSourceDimensions.primary
      : previewSourceDimensions.camera;
  // With the camera a pane of its own, a scene only places the screen, so the
  // camera's crop still shows it whole.
  const isCropping =
    canvasTool === "crop" &&
    !(isFramed && (activeVideoTrack === "primary" || bakeCamera));
  const previewRecordingOutput = useMemo(() => {
    if (!activeVideoTrack || !cropSource) return effectiveRecordingOutput;
    if (bakeCamera && activeVideoTrack === "camera")
      return effectiveRecordingOutput;
    if (isFramed && activeVideoTrack === "primary")
      return effectiveRecordingOutput;
    const output = effectiveRecordingOutput[activeVideoTrack];
    const drawn = isCropping
      ? croppingScreenshotPreviewOutput(cropSource, output, { isFlat, share })
      : withShareOfTurn(output, share);
    return drawn === output
      ? effectiveRecordingOutput
      : { ...effectiveRecordingOutput, [activeVideoTrack]: drawn };
  }, [
    activeVideoTrack,
    bakeCamera,
    cropSource,
    effectiveRecordingOutput,
    isCropping,
    isFlat,
    isFramed,
    share,
  ]);
  const previewCameraOverlay = useMemo(() => {
    if (
      !isCropping ||
      activeVideoTrack !== "camera" ||
      !bakeCamera ||
      !previewSourceDimensions.camera
    )
      return cameraOverlay;
    const output = effectiveRecordingOutput.primary;
    return uncroppedCameraPreviewOverlay(
      {
        height: output.height,
        kind: "screen",
        sourceHeight: output.height,
        sourceWidth: output.width,
        width: output.width,
        x: 0,
        y: 0,
      },
      {
        height: previewSourceDimensions.camera.height,
        kind: "camera",
        sourceHeight: previewSourceDimensions.camera.height,
        sourceWidth: previewSourceDimensions.camera.width,
        width: previewSourceDimensions.camera.width,
        x: 0,
        y: 0,
      },
      cameraOverlay,
    );
  }, [
    activeVideoTrack,
    bakeCamera,
    cameraOverlay,
    effectiveRecordingOutput.primary,
    isCropping,
    previewSourceDimensions.camera,
  ]);
  return { previewCameraOverlay, previewRecordingOutput };
}
