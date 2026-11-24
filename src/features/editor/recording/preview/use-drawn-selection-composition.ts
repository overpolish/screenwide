// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useMemo } from "react";

import {
  recordingBetween,
  useTweenedOutput,
} from "../../preview/use-tweened-output";

import type { RecordingOutputSettings } from "../../screenshot/screenshot-output";
import type { CameraOverlaySettings } from "../../types";
import type { RecordingSceneArrangement } from "../scenes/recording-scene-arrangement";

/**
 * The screen and the camera as the selection's outline and handles are
 * placed by: where the scene at the playhead puts them, else the recording's
 * own, and the camera following the face. A change an island asked to be
 * moved to (`picture-tween.ts`) moves them with the picture, rather than
 * leaving the outline at the new place while the picture is still on its
 * way there.
 */
export function useDrawnSelectionComposition({
  arrangement,
  followedCameraOverlay,
  output,
}: {
  arrangement: RecordingSceneArrangement | null;
  followedCameraOverlay: CameraOverlaySettings;
  output: RecordingOutputSettings;
}) {
  const target = useMemo(
    () => ({
      cameraOverlay: arrangement?.cameraOverlay ?? followedCameraOverlay,
      recordingOutput: arrangement
        ? { ...output, primary: arrangement.output }
        : output,
    }),
    [arrangement, followedCameraOverlay, output],
  );
  return useTweenedOutput(
    target,
    { part: "chrome", workspace: "recording" },
    recordingBetween,
  );
}
