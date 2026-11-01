// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  SceneRotation,
  UPRIGHT_ROTATION,
  withRotation,
} from "../../layer-rotation";
import {
  padWithInsetColor,
  recenterWorkspaceContent,
} from "../../recenter-inset-channel";
import { CameraEffectsTool } from "../../recording/camera/use-camera-effects";
import {
  insetScreenshotPadding,
  screenshotInsetShare,
} from "../../screenshot/screenshot-inset";
import { ScreenshotOutputSettings } from "../../screenshot/screenshot-output";
import { EditorKind } from "../../types";
import { ToolPanelLayerSelection } from "../tool-panel-store";

/**
 * The one layer the selection panel acts on: what to show for it, and the
 * edits the editor already owns for it.
 *
 * Each edit is an intent rather than a settings object, so a layer placed by
 * an output of its own and a camera placed by the baked overlay can both be
 * this, and the panel never has to know which it is looking at.
 */
export type EditorSelectionTarget = {
  /** Cast the layer's shadow onto the canvas, or take it away. */
  applyDropShadow: (dropShadow: boolean) => void;
  /** Pad the layer by this share of its picture's shorter side on each
   * side. */
  applyInset: (inset: number) => void;
  /** Round the layer's corners by this share of its shorter side. */
  applyRadius: (radius: number) => void;
  /** Put the layer's content in the middle of its padded frame. */
  recenter: () => void;
  selection: ToolPanelLayerSelection;
  /** Have the camera follow the face, or stop, where the layer is a camera
   * drawn into the screen's picture. */
  applyFollowFace?: (follow: boolean) => void;
  /** Turn the layer in depth, a field at a time, where it can be turned. */
  applyRotation?: (rotation: Partial<SceneRotation>) => void;
  /** The camera's effects, where the layer is the camera. */
  cameraEffects?: CameraEffectsTool;
};

/**
 * A layer placed by an output of its own: a screenshot layer, or a recording
 * track carried as its own picture.
 *
 * Every edit goes back out through the workspace's own output handler, so a
 * number typed in the panel takes the identical path through the editor's
 * state as the drag that would have produced it.
 */
export const placedSelectionTarget = ({
  apply,
  kind,
  label,
  settings,
  source,
  turns,
  workspace,
}: {
  apply: (next: ScreenshotOutputSettings) => void;
  kind: ToolPanelLayerSelection["kind"];
  label: string;
  settings: ScreenshotOutputSettings;
  source: { height: number; width: number };
  /** Whether the layer turns in depth: the screen of a recording and a
   * screenshot's layers do, the camera does not. */
  turns: boolean;
  /** The workspace whose padding analysis this layer's pad is filled from, or
   * null where a layer there carries no pad. */
  workspace: EditorKind | null;
}): EditorSelectionTarget => {
  const rotation = settings.rotation ?? UPRIGHT_ROTATION;
  return {
    // The shadow is the layer's own, cast onto whatever the canvas is wearing,
    // so it travels with the layer's output rather than with the canvas.
    applyDropShadow: (dropShadow) => {
      apply({ ...settings, dropShadow });
    },
    applyInset: (inset) => {
      const pad = () => {
        apply(insetScreenshotPadding(settings, source, inset));
      };
      // The pad is filled with the colour detected behind the content, so
      // the first pixel of padding asks for that colour if none was read
      // yet, after the padding is committed so the colour lands on the
      // padded layer rather than on the one before it.
      if (inset > 0 && !settings.recenterInsetColor && workspace)
        padWithInsetColor(workspace, pad);
      else pad();
    },
    applyRadius: (radiusPercent) => {
      apply({ ...settings, radiusPercent });
    },
    ...(turns
      ? {
          applyRotation: (turn: Partial<SceneRotation>) => {
            apply(withRotation(settings, { ...rotation, ...turn }));
          },
        }
      : {}),
    recenter: () => {
      if (workspace) recenterWorkspaceContent(workspace);
    },
    selection: {
      dropShadow: settings.dropShadow,
      // A share of the layer's own picture, its source crop, rather than of
      // its padded frame, so it does not move as the padding changes, and a
      // large look is large whatever the layer's size.
      inset: screenshotInsetShare(settings, source),
      kind,
      label,
      radius: settings.radiusPercent,
      ...(turns ? { rotation } : {}),
    },
  };
};
