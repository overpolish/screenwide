// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCameraEffects } from "../../recording/camera/use-camera-effects";

import {
  EditorSelectionTarget,
  editorSelectionTarget,
  SelectionTargetInputs,
} from "./selection-target";

/**
 * The placed layer the selection panel acts on, and where it is the camera,
 * the camera's effects with it: they go with the camera wherever it is
 * drawn, as a pane of its own or baked into the screen's video.
 */
export function useLayerSelectionTarget(
  inputs: SelectionTargetInputs,
): EditorSelectionTarget | null {
  const cameraEffects = useCameraEffects(inputs.artifact);
  const target = editorSelectionTarget(inputs);
  // The camera's effects sit with the camera, or with the recording itself
  // where it is of the camera alone.
  const isCamera =
    target?.selection.kind === "camera" ||
    (target?.selection.kind === "primary" &&
      inputs.artifact?.kind === "recording" &&
      inputs.artifact.primaryKind === "camera");
  if (!target || !isCamera || !cameraEffects) return target;
  return {
    ...target,
    cameraEffects,
    selection: { ...target.selection, cameraEffects: cameraEffects.shown },
  };
}
