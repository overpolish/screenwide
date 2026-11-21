// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import { uncroppedCameraPreviewOverlay } from "../../preview/camera-overlay-geometry";
import { cameraOverlayBetween } from "../../preview/picture-between";
import { croppingScreenshotPreviewOutput } from "../../screenshot/screenshot-crop";
import { RecordingOutputSettings } from "../../screenshot/screenshot-output";
import { CameraOverlaySettings, RecordingVideoTrackId } from "../../types";

import type { ScrubPreviewProps } from "./scrub-preview";
import type { CropTurn } from "../../preview/use-crop-turn";

/** What the player draws as a crop comes and goes. The track in hand is
 * opened out around its crop - a baked camera to the place its overlay
 * holds, from where the face puts it when it follows the face - and turned
 * upright as the selection's `cropTurn` eases, so the handles sit on the
 * picture the crop is taken out of, and closes again as the tool is put
 * down. Inside a scene the native preview opens the pane in place instead
 * (`scene_reframe.rs`): the scene would arrange the whole picture as if it
 * were the crop. */
export function useRecordingCropPreview({
  activeVideoTrack,
  bakeCamera,
  cameraOverlay,
  effectiveRecordingOutput,
  previewSourceDimensions,
  selection: { cropTurn, cropTurnTrack, followedCameraOverlay, isFramed },
}: {
  activeVideoTrack: RecordingVideoTrackId | null;
  bakeCamera: boolean;
  cameraOverlay: CameraOverlaySettings;
  effectiveRecordingOutput: RecordingOutputSettings;
  previewSourceDimensions: ScrubPreviewProps["previewSourceDimensions"];
  selection: {
    cropTurn: CropTurn;
    /** The track the crop opens and closes on, kept while it closes after
     * the track is let go. */
    cropTurnTrack: RecordingVideoTrackId | null;
    /** The camera's overlay with its picture where the preview draws it,
     * which a camera following the face opens from. */
    followedCameraOverlay: CameraOverlaySettings;
    isFramed: boolean;
  };
}) {
  // The layer the crop is open on, kept while it closes, whether the track
  // was let go or another layer was pressed (`useRecordingCropTurn`).
  const track = cropTurnTrack ?? activeVideoTrack;
  const cropSource =
    track === "primary"
      ? previewSourceDimensions.primary
      : previewSourceDimensions.camera;
  const previewRecordingOutput = useMemo(() => {
    if (!track || !cropSource) return effectiveRecordingOutput;
    if (bakeCamera && track === "camera") return effectiveRecordingOutput;
    if (isFramed && track === "primary") return effectiveRecordingOutput;
    const output = effectiveRecordingOutput[track];
    // Opened out while the tool is in hand and closing as it goes.
    const drawn = croppingScreenshotPreviewOutput(cropSource, output, cropTurn);
    return drawn === output
      ? effectiveRecordingOutput
      : { ...effectiveRecordingOutput, [track]: drawn };
  }, [
    bakeCamera,
    cropSource,
    cropTurn,
    effectiveRecordingOutput,
    isFramed,
    track,
  ]);
  const previewCameraOverlay = useMemo(() => {
    if (
      cropTurn.open === 0 ||
      track !== "camera" ||
      !bakeCamera ||
      isFramed ||
      !previewSourceDimensions.camera
    )
      return cameraOverlay;
    const output = effectiveRecordingOutput.primary;
    const whole = uncroppedCameraPreviewOverlay(
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
      followedCameraOverlay,
    );
    return cropTurn.isFlat
      ? whole
      : cameraOverlayBetween(followedCameraOverlay, whole, cropTurn.open);
  }, [
    bakeCamera,
    cameraOverlay,
    cropTurn,
    effectiveRecordingOutput.primary,
    followedCameraOverlay,
    isFramed,
    previewSourceDimensions.camera,
    track,
  ]);
  return { previewCameraOverlay, previewRecordingOutput };
}
