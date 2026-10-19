// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  SceneRotation,
  UPRIGHT_ROTATION,
  withRotation,
} from "../../layer-rotation";
import {
  prepareRecenterInset,
  recenterWorkspaceContent,
} from "../../recenter-inset-channel";
import { CameraEffectsTool } from "../../recording/camera/use-camera-effects";
import {
  resetScreenshotTransform,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import {
  insetScreenshotPadding,
  screenshotPaddingInset,
} from "../../screenshot/screenshot-recenter";
import { EditorKind } from "../../types";
import {
  SelectionPlacementPatch,
  selectionPlacement,
  withSelectionPlacement,
} from "../selection-placement";
import { ToolPanelLayerSelection } from "../tool-panel-store";

const insetSliderMaximum = 1_000;

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
  /** Pad the layer by this many output pixels on each side. */
  applyInset: (inset: number) => void;
  /** Place the layer at the size and position a field asked for. */
  applyPlacement: (placement: SelectionPlacementPatch) => void;
  /** Round the layer's corners by this share of its shorter side. */
  applyRadius: (radius: number) => void;
  /** Put the layer's content in the middle of its padded frame. */
  recenter: () => void;
  /** Put the layer back to the framing its source arrived in. */
  reset: () => void;
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
      apply(insetScreenshotPadding(settings, source, inset));
      // The pad is filled with the colour detected behind the content, so the
      // first pixel of padding asks for that colour if none was read yet. It
      // is asked for after the padding is committed, so the colour lands on
      // the padded layer rather than on the one before it.
      if (inset > 0 && !settings.recenterInsetColor && workspace)
        prepareRecenterInset(workspace);
    },
    applyPlacement: (placement) => {
      apply(withSelectionPlacement(settings, source, placement));
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
    // The reset the Select tool has always had: the layer back to the framing
    // its source arrived in, its crop kept.
    reset: () => {
      apply(resetScreenshotTransform(settings, source));
    },
    selection: {
      ...selectionPlacement(settings),
      dropShadow: settings.dropShadow,
      inset: Math.round(screenshotPaddingInset(settings, source)),
      // The padding is measured against the layer's own picture rather than
      // its padded frame, so the range does not move as the knob is dragged.
      // Keep the slider useful on small captures without making typed values
      // depend on the source dimensions. Number entry remains unbounded by
      // this soft slider range.
      insetMaximum: insetSliderMaximum,
      kind,
      label,
      radius: settings.radiusPercent,
      ...(turns ? { rotation } : {}),
      sourceHeight: source.height,
      sourceWidth: source.width,
    },
  };
};
