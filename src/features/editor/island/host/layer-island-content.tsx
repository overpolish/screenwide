// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { cameraIsland } from "../camera/camera-island";
import { cropIsland } from "../crop/crop-island";
import { screenIsland } from "../screen/screen-island";

import type { ToolPanelPatch } from "../../tool-panels/tool-panel-patch";
import type { ToolPanelSnapshot } from "../../tool-panels/tool-panel-store";
import type { EditorIslandContent } from "../editor-island";

// The islands of the layers a recording or screenshot is made of: the
// screen (or a screenshot's layer) and the camera, each for the whole
// recording, and the Crop tool's, for the layer in hand. Each is the layer
// in hand: the Screen and Camera modes choose their track, and the Crop
// tool crops the one chosen. Over a scene that shows the layer, its
// corners, the screen's inset, turn and Recenter, the camera following the
// face, and the crop are that scene's own, which the scene's patches reach
// (the crop through the crop target, which is the scene's there); the
// scene's corners are the selected pane's, which is this layer's.

type Inputs = {
  change: (values: ToolPanelPatch) => void;
  snapshot: ToolPanelSnapshot;
};

export function screenIslandContent({
  capture,
  change,
  snapshot,
}: Inputs & {
  capture: "recording" | "screenshot";
}): EditorIslandContent | null {
  const { scene, selection } = snapshot;
  if (!selection || selection.kind === "audio") return null;
  // Until the screen's track is in hand the selection is another layer's.
  if (capture === "recording" && selection.kind !== "primary") return null;
  const scenePane = scene?.hasSceneAtPlayhead ? scene.panes.screen : null;
  const sceneRotation = scenePane ? scene?.rotation : null;
  const rotation = sceneRotation ?? selection.rotation;
  // A scene's inset is a share of its box's shorter side, offered as the
  // same looks as the recording's, which are shares of the screen's.
  return screenIsland({
    capture,
    hasShadow: selection.dropShadow,
    inset: scenePane ? scenePane.inset : selection.inset,
    isAtDefaultPlacement: scenePane
      ? scenePane.isAtLayout
      : selection.isAtDefaultPlacement,
    isInScene: scenePane !== null,
    onInsetChange: (inset) => {
      change(scenePane ? { sceneInset: inset } : { selectionInset: inset });
    },
    onRadiusChange: (radius) => {
      change(scenePane ? { sceneRadius: radius } : { selectionRadius: radius });
    },
    onRecenter: () => {
      change(scenePane ? { sceneRecenter: true } : { recenterSelection: true });
    },
    onResetPlacement: () => {
      change(
        scenePane
          ? { scenePlacementReset: "screen" }
          : { resetPlacement: true },
      );
    },
    onShadowChange: (selectionDropShadow) => {
      change({ selectionDropShadow });
    },
    radius: scenePane?.radius ?? selection.radius,
    ...(rotation
      ? {
          onRotationChange: (turn) => {
            change(
              sceneRotation
                ? { sceneRotation: turn }
                : { selectionRotation: turn },
            );
          },
          rotation,
        }
      : {}),
  });
}

export function cameraIslandContent({
  change,
  isFindingFace,
  snapshot,
}: Inputs & {
  /** Whether the face is being looked for, which following it starts. */
  isFindingFace: boolean;
}): EditorIslandContent | null {
  const { scene, selection } = snapshot;
  // Until the camera's track is in hand the selection is another layer's.
  // The camera's effects come with the recording's own track only where
  // the recording is of the camera alone, whose own picture it then is.
  if (
    !selection ||
    selection.kind === "audio" ||
    selection.kind === "layer" ||
    !selection.cameraEffects
  )
    return null;
  const isOwnPicture = selection.kind === "primary";
  const paneName = isOwnPicture ? "screen" : "camera";
  const scenePane = scene?.hasSceneAtPlayhead ? scene.panes[paneName] : null;
  const followFace =
    scene && scenePane ? scene.followsFace : (selection.followsFace ?? false);
  const sceneRotation = scenePane ? scene?.rotation : null;
  const rotation = sceneRotation ?? selection.rotation;
  return cameraIsland({
    effects: selection.cameraEffects,
    followFace,
    hasShadow: selection.dropShadow,
    isAtDefaultPlacement: scenePane
      ? scenePane.isAtLayout
      : selection.isAtDefaultPlacement,
    isFindingFace: followFace && isFindingFace,
    isInScene: scenePane !== null,
    onBackgroundChange: (background) => {
      change({ cameraEffects: { background } });
    },
    onChoosePicture: () => {
      change({ chooseCameraBackground: true });
    },
    onDownloadBackgroundModel: () => {
      change({ downloadCameraBackground: true });
    },
    onDownloadObjectsModel: () => {
      change({ downloadCameraObjects: true });
    },
    onEnhanceChange: (enhance) => {
      change({ cameraEffects: { enhance } });
    },
    onFollowFaceChange: (follow) => {
      change(
        scenePane
          ? { sceneFollowFace: follow }
          : { selectionFollowFace: follow },
      );
    },
    onPickObjects: (isPicking) => {
      change({ pickCameraObjects: isPicking });
    },
    onRadiusChange: (radius) => {
      change(scenePane ? { sceneRadius: radius } : { selectionRadius: radius });
    },
    onRemoveObject: (id) => {
      change({ removeCameraObject: id });
    },
    onResetPlacement: () => {
      change(
        scenePane
          ? { scenePlacementReset: paneName }
          : { resetPlacement: true },
      );
    },
    onShadowChange: (selectionDropShadow) => {
      change({ selectionDropShadow });
    },
    onShowObject: (id) => {
      change({ showCameraObject: id });
    },
    // The recording's own picture is padded, turned and recentred as the
    // screen is, the scene's screen pane over a scene.
    ...(isOwnPicture
      ? {
          picture: {
            inset: scenePane ? scenePane.inset : selection.inset,
            onInsetChange: (inset) => {
              change(
                scenePane ? { sceneInset: inset } : { selectionInset: inset },
              );
            },
            onRecenter: () => {
              change(
                scenePane
                  ? { sceneRecenter: true }
                  : { recenterSelection: true },
              );
            },
            ...(rotation
              ? {
                  onRotationChange: (turn) => {
                    change(
                      sceneRotation
                        ? { sceneRotation: turn }
                        : { selectionRotation: turn },
                    );
                  },
                  rotation,
                }
              : {}),
          },
        }
      : {}),
    radius: scenePane?.radius ?? selection.radius,
  });
}

/** The Crop tool's island, for the layer in hand: the screen's or the
 * camera's in a recording, its own crop or, over a scene that shows it,
 * the scene's. */
export function cropIslandContent({
  change,
  snapshot,
}: Inputs): EditorIslandContent | null {
  const { crop, scene, selection } = snapshot;
  if (!crop || !selection || selection.kind === "audio") return null;
  const pane =
    scene?.hasSceneAtPlayhead && selection.kind === "camera"
      ? scene.panes.camera
      : scene?.hasSceneAtPlayhead
        ? scene.panes.screen
        : null;
  return cropIsland({
    aspect: crop.aspect,
    canLock: crop.canLock,
    isInScene: pane !== null,
    onAspectChange: (cropAspect) => {
      change({ cropAspect });
    },
    onReset: () => {
      change({ resetCrop: true });
    },
  });
}
