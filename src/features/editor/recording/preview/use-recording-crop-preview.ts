// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { uncroppedCameraPreviewOverlay } from "../../preview/camera-overlay-geometry";
import { uncroppedScreenshotPreviewOutput } from "../../screenshot/screenshot-crop";
import { RecordingOutputSettings } from "../../screenshot/screenshot-output";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";

import type { ScrubPreviewProps } from "./scrub-preview";

/** What the player draws while a crop is in hand. The framed track is shown
 * whole - and a baked camera in the place its overlay holds - so the handles
 * sit on the picture the crop is taken out of. Inside a scene the crop tool
 * pans and zooms the picture within its box instead, so nothing is shown
 * whole: the scene would arrange the whole picture as if it were the crop. */
export function useRecordingCropPreview({
  activeVideoTrack,
  bakeCamera,
  cameraOverlay,
  canvasTool,
  effectiveRecordingOutput,
  isFramed,
  previewSourceDimensions,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  canvasTool: RecordingCanvasTool;
  effectiveRecordingOutput: RecordingOutputSettings;
  isFramed: boolean;
  previewSourceDimensions: ScrubPreviewProps["previewSourceDimensions"];
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
    if (!isCropping || !activeVideoTrack || !cropSource)
      return effectiveRecordingOutput;
    if (bakeCamera && activeVideoTrack === "camera")
      return effectiveRecordingOutput;
    return {
      ...effectiveRecordingOutput,
      [activeVideoTrack]: uncroppedScreenshotPreviewOutput(
        cropSource,
        effectiveRecordingOutput[activeVideoTrack],
      ),
    };
  }, [
    activeVideoTrack,
    bakeCamera,
    cropSource,
    effectiveRecordingOutput,
    isCropping,
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
