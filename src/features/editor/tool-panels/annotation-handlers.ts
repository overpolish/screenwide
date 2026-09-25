// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  withAnnotationColor,
  withoutAnnotationColor,
} from "../../../components/shared/annotation-style/palette";
import {
  applyAnnotationAngle,
  applyAnnotationAnimated,
  applyAnnotationReverse,
  applyAnnotationShuffle,
  applyAnnotationStyle,
} from "../annotation-channel";
import { EditorKind } from "../types";

import { ToolPanelHandlers } from "./tool-panel-bridge";

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
  | "onAnnotationReverse"
  | "onAnnotationShuffle"
  | "onAnnotationStyleChange"
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
  onAnnotationReverse: () => {
    applyAnnotationReverse(workspace);
  },
  onAnnotationShuffle: () => {
    applyAnnotationShuffle(workspace);
  },
  onAnnotationStyleChange: (style) => {
    applyAnnotationStyle(workspace, style);
  },
});
