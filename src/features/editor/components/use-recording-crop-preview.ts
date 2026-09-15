// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { uncroppedCameraPreviewOverlay } from "../camera-overlay-geometry";
import { uncroppedScreenshotPreviewOutput } from "../screenshot-crop";
import { RecordingOutputSettings } from "../screenshot-output";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../types";

import { RecordingCanvasTool } from "./recording-crop-toggle";

import type { ScrubPreviewProps } from "./scrub-preview";

/** What the player draws while a crop is in hand. The framed track is shown
 * whole - and a baked camera in the place its overlay holds - so the handles
 * sit on the picture the crop is taken out of. */
export function useRecordingCropPreview({
  activeVideoTrack,
  bakeCamera,
  cameraOverlay,
  canvasTool,
  effectiveRecordingOutput,
  previewSourceDimensions,
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  canvasTool: RecordingCanvasTool;
  effectiveRecordingOutput: RecordingOutputSettings;
  previewSourceDimensions: ScrubPreviewProps["previewSourceDimensions"];
}) {
  const cropSource =
    activeVideoTrack === "primary"
      ? previewSourceDimensions.primary
      : previewSourceDimensions.camera;
  const previewRecordingOutput = useMemo(() => {
    if (canvasTool !== "crop" || !activeVideoTrack || !cropSource)
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
    canvasTool,
    cropSource,
    effectiveRecordingOutput,
  ]);
  const previewCameraOverlay = useMemo(() => {
    if (
      canvasTool !== "crop" ||
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
    canvasTool,
    effectiveRecordingOutput.primary,
    previewSourceDimensions.camera,
  ]);
  return { previewCameraOverlay, previewRecordingOutput };
}
