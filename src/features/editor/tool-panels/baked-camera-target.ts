// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { withCameraOverlaySize } from "../camera-overlay-placement";
import { defaultCameraOverlay } from "../recording-export-settings";
import { ScreenshotOutputSettings } from "../screenshot-output";
import { CameraOverlaySettings, EditorArtifact } from "../types";

import { EditorSelectionTarget } from "./placed-selection-target";

type BakedCameraTargetInputs = {
  artifact: Extract<EditorArtifact, { kind: "recording" }>;
  /** The camera track's own output, which carries the shadow the compositor
   * casts for the baked camera. */
  cameraOutput: ScreenshotOutputSettings;
  /** Where the camera is drawn in the screen's picture, in its output pixels. */
  cameraOverlay: CameraOverlaySettings;
  onBakeCameraChange?: (bake: boolean) => void;
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onRecordingOutputChange?: (
    track: "camera",
    next: ScreenshotOutputSettings,
  ) => void;
};

const whole = (value: number) => Math.round(value);

/**
 * The camera as the compositor draws it: one picture, with the camera in it.
 *
 * A baked camera has no output of its own to be placed by, so everything the
 * panel shows for it is read out of the overlay and written back to it, by the
 * same calls the drag on the overlay in the preview makes. Its shadow is the
 * one exception: the compositor takes that from the camera track's own output
 * in both states, so the switch between them cannot lose it.
 */
export const bakedCameraSelectionTarget = ({
  artifact,
  cameraOutput,
  cameraOverlay,
  onBakeCameraChange,
  onCameraOverlayChange,
  onRecordingOutputChange,
}: BakedCameraTargetInputs): EditorSelectionTarget | null => {
  const camera = artifact.camera;
  if (!camera) return null;
  return {
    applyBake: (bake) => {
      onBakeCameraChange?.(bake);
    },
    applyDropShadow: (dropShadow) => {
      onRecordingOutputChange?.("camera", { ...cameraOutput, dropShadow });
    },
    applyInset: () => {
      // A baked camera is placed in the screen's picture rather than padded
      // against a colour of its own, so it carries no pad.
    },
    applyPlacement: (placement) => {
      onCameraOverlayChange?.(withCameraOverlaySize(cameraOverlay, placement));
    },
    applyRadius: (radiusPercent) => {
      onCameraOverlayChange?.({ ...cameraOverlay, radiusPercent });
    },
    recenter: () => {
      // Nothing to recentre: there is no pad to put the camera back inside of.
    },
    // Back to where the camera is put when a recording is opened, which is
    // what the canvas resets it to as well.
    reset: () => {
      onCameraOverlayChange?.(defaultCameraOverlay(artifact));
    },
    selection: {
      canBake: true,
      dropShadow: cameraOutput.dropShadow,
      height: whole(cameraOverlay.frameHeight),
      inset: 0,
      insetMaximum: 0,
      isBaked: true,
      kind: "camera",
      label: "Camera",
      radius: cameraOverlay.radiusPercent,
      sourceHeight: camera.height,
      sourceWidth: camera.width,
      width: whole(cameraOverlay.frameWidth),
      x: whole(cameraOverlay.frameX),
      y: whole(cameraOverlay.frameY),
    },
  };
};
