// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { drawingToolKind } from "../tools/tool-registry";

import type { AnnotationKind } from "../../annotations/types";
import type { EditorIslandKind } from "../island/host/editor-island-kind";
import type { EditorMode } from "../toolbar/editor-modes";
import type { EditorTool } from "../toolbar/editor-tool-column";

/** What a press on the screenshot does: Select, a pen, the marquee, the
 * crop, or the frame's own drag (`canvas`). */
export type ScreenshotTool =
  AnnotationKind | "canvas" | "crop" | "marquee" | "select" | null;

// The screenshot's tools as the tool column and the title bar show them,
// and the island each brings out. The frame's drag keeps its name
// (`canvas`); the title bar names it by its mode, Frame. The Screenshot
// mode has no drag of its own: it is the chosen layer's island, open over
// Select. A spotlight is the focus pen's dimmed look, so the focus pen
// stands for it.

/** The pen, Crop or Select the column shows pressed for `tool`. */
export const columnTool = (tool: ScreenshotTool): EditorTool => {
  if (tool === "crop") return "crop";
  const pen = drawingToolKind(tool);
  if (pen === "spotlight") return "focus";
  return pen ?? "select";
};

/** The mode the title bar shows open for `tool`: the Screenshot mode is
 * Select on a chosen layer, however it was chosen, so its button is lit
 * whenever its island is out. */
export const columnMode = (
  tool: ScreenshotTool,
  hasSelectedLayer: boolean,
): EditorMode | null =>
  tool === "canvas"
    ? "frame"
    : tool === "select" && hasSelectedLayer
      ? "screen"
      : null;

/** Which island the tool in hand brings out: a pen's own, or the
 * annotations chosen with it or with the marquee; the crop's; the frame's;
 * and a layer chosen with Select, which the Screenshot island holds.
 * Nothing in hand with Select brings out nothing. */
export const screenshotIsland = ({
  annotationCount,
  hasSelectedLayer,
  selectedKind,
  tool,
}: {
  annotationCount: number;
  hasSelectedLayer: boolean;
  selectedKind: AnnotationKind | null;
  tool: ScreenshotTool;
}): EditorIslandKind | null => {
  if (
    drawingToolKind(tool) !== null ||
    annotationCount > 1 ||
    selectedKind !== null
  )
    return "annotation";
  if (tool === "canvas") return "frame";
  if (tool === "crop") return "crop";
  if (tool === "select" && hasSelectedLayer) return "screen";
  return null;
};
