// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  withAnnotationColor,
  withoutAnnotationColor,
} from "../../../../composites/color-grid/palette";
import {
  applyAnnotationAngle,
  applyAnnotationAnimated,
  applyAnnotationReverse,
  applyAnnotationRotation,
  applyAnnotationShuffle,
  applyAnnotationKind,
  applyAnnotationsDelete,
  applyAnnotationEmphasis,
  applyAnnotationImage,
  applyAnnotationLook,
  applyAnnotationPin,
  applyAnnotationSceneReset,
  applyAnnotationImagePlay,
  applyAnnotationStyle,
  applyAnnotationSway,
  applyAnnotationZoom,
} from "../../annotations/annotation-apply";
import { EditorKind } from "../../types";
import { ToolPanelHandlers } from "../tool-panel-handlers";

/**
 * The annotation panel's asks, answered against the annotation `workspace`
 * has in hand, and the colours of your own kept beside the palette:
 * `annotationColors` is the list kept now and `saveColors` keeps a new one.
 */
export const annotationPanelHandlers = (
  workspace: EditorKind,
  annotationColors: string[],
  saveColors: (colors: string[]) => void,
): Pick<
  ToolPanelHandlers,
  | "onAnnotationAngleChange"
  | "onAnnotationAnimatedChange"
  | "onAnnotationColorRemove"
  | "onAnnotationColorSave"
  | "onAnnotationEmphasisChange"
  | "onAnnotationImageChange"
  | "onAnnotationImagePlayChange"
  | "onAnnotationImageSwayChange"
  | "onAnnotationKindChange"
  | "onAnnotationLookChange"
  | "onAnnotationPin"
  | "onAnnotationSceneReset"
  | "onAnnotationReverse"
  | "onAnnotationRotationChange"
  | "onAnnotationShuffle"
  | "onAnnotationsDelete"
  | "onAnnotationStyleChange"
  | "onAnnotationZoomChange"
> => ({
  onAnnotationAngleChange: (angle) => {
    applyAnnotationAngle(workspace, angle);
  },
  onAnnotationAnimatedChange: (animated) => {
    applyAnnotationAnimated(workspace, animated);
  },
  onAnnotationColorRemove: (color) => {
    saveColors(withoutAnnotationColor(annotationColors, color));
  },
  onAnnotationColorSave: (color) => {
    saveColors(withAnnotationColor(annotationColors, color));
  },
  onAnnotationEmphasisChange: (emphasis) => {
    applyAnnotationEmphasis(workspace, emphasis);
  },
  onAnnotationImageChange: (art) => {
    applyAnnotationImage(workspace, art);
  },
  onAnnotationImagePlayChange: (play) => {
    applyAnnotationImagePlay(workspace, play);
  },
  onAnnotationImageSwayChange: (sway) => {
    applyAnnotationSway(workspace, sway);
  },
  onAnnotationKindChange: (kind) => {
    applyAnnotationKind(workspace, kind);
  },
  onAnnotationLookChange: (look) => {
    applyAnnotationLook(workspace, look);
  },
  onAnnotationPin: (action) => {
    applyAnnotationPin(workspace, action);
  },
  onAnnotationReverse: () => {
    applyAnnotationReverse(workspace);
  },
  onAnnotationRotationChange: (rotation) => {
    applyAnnotationRotation(workspace, rotation);
  },
  onAnnotationSceneReset: () => {
    applyAnnotationSceneReset(workspace);
  },
  onAnnotationShuffle: () => {
    applyAnnotationShuffle(workspace);
  },
  onAnnotationStyleChange: (style) => {
    applyAnnotationStyle(workspace, style);
  },
  onAnnotationZoomChange: (zoom) => {
    applyAnnotationZoom(workspace, zoom);
  },
  onAnnotationsDelete: () => {
    applyAnnotationsDelete(workspace);
  },
});
