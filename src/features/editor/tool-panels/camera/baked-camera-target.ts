// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { defaultCameraOverlay } from "../../export/recording-export-settings";
import {
  isOverlayAtDefaultPlacement,
  overlayAtDefaultPlacement,
} from "../../layer-placement";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { CameraOverlaySettings, EditorArtifact } from "../../types";
import { EditorSelectionTarget } from "../selection/placed-selection-target";

type BakedCameraTargetInputs = {
  artifact: Extract<EditorArtifact, { kind: "recording" }>;
  /** The camera track's own output, which carries the shadow the compositor
   * casts for the baked camera. */
  cameraOutput: ScreenshotOutputSettings;
  /** Where the camera is drawn in the screen's picture, in its output pixels. */
  cameraOverlay: CameraOverlaySettings;
  /** The screen's canvas the camera is drawn into, which a fresh project
   * places it on. */
  canvas: { height: number; width: number };
  onCameraOverlayChange?: (settings: CameraOverlaySettings) => void;
  onRecordingOutputChange?: (
    track: "camera",
    next: ScreenshotOutputSettings,
  ) => void;
};

/**
 * The camera as the compositor draws it: one picture, with the camera in it.
 *
 * A baked camera has no output of its own to be placed by, so everything the
 * panel shows for it is read out of the overlay and written back to it, by the
 * same calls the drag on the overlay in the preview makes. Its shadow is the
 * one exception: the compositor takes that from the camera track's own output
 * in both output modes, so switching between them cannot lose it.
 */
export const bakedCameraSelectionTarget = ({
  artifact,
  cameraOutput,
  cameraOverlay,
  canvas,
  onCameraOverlayChange,
  onRecordingOutputChange,
}: BakedCameraTargetInputs): EditorSelectionTarget | null => {
  const camera = artifact.camera;
  if (!camera) return null;
  const fresh = defaultCameraOverlay(artifact, canvas);
  return {
    applyDropShadow: (dropShadow) => {
      onRecordingOutputChange?.("camera", { ...cameraOutput, dropShadow });
    },
    // Following holds the face in the middle of the crop, wherever the
    // picture sat behind it, so the crop keeps its size, shape and zoom.
    applyFollowFace: (follow) => {
      const { followFace: _followFace, ...still } = cameraOverlay;
      onCameraOverlayChange?.(follow ? { ...still, followFace: true } : still);
    },
    applyInset: () => {
      // A baked camera is placed in the screen's picture rather than padded
      // against a colour of its own, so it carries no pad.
    },
    applyRadius: (radiusPercent) => {
      onCameraOverlayChange?.({ ...cameraOverlay, radiusPercent });
    },
    recenter: () => {
      // Nothing to recentre: there is no pad to put the camera back inside of.
    },
    resetPlacement: () => {
      onCameraOverlayChange?.(overlayAtDefaultPlacement(cameraOverlay, fresh));
    },
    selection: {
      dropShadow: cameraOutput.dropShadow,
      followsFace: cameraOverlay.followFace ?? false,
      inset: 0,
      isAtDefaultPlacement: isOverlayAtDefaultPlacement(cameraOverlay, fresh),
      kind: "camera",
      label: t("editor-island-camera"),
      radius: cameraOverlay.radiusPercent,
    },
  };
};
