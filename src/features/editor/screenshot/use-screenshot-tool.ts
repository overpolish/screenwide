// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { toolDisagreesWithAnnotation } from "../annotations/use-tool-follows-annotation";
import { isAnnotationTool } from "../tool-panels/tool-registry";

import { ScreenshotTool } from "./screenshot-tools";

import type { AnnotationKind } from "../../../components/shared/annotation-style/types";

/** Picking a screenshot tool up, and what that clears on its way in. The twin
 * of `useRecordingPreviewCanvasTool`'s own `changeCanvasTool`. */
export function useScreenshotTool({
  clearSelection,
  selectedKind,
  setActiveTool,
  tool,
}: {
  clearSelection: () => void;
  selectedKind: AnnotationKind | null;
  setActiveTool: (tool: ScreenshotTool) => void;
  tool: ScreenshotTool;
}) {
  const toolRef = useRef(tool);
  toolRef.current = tool;
  return (
    next: ScreenshotTool | ((current: ScreenshotTool) => ScreenshotTool),
  ) => {
    const resolved = typeof next === "function" ? next(toolRef.current) : next;
    // A tool that picks no annotation up lets the annotation in hand go, and
    // so does a tool that draws a different one.
    if (
      !isAnnotationTool(resolved) ||
      toolDisagreesWithAnnotation(resolved, selectedKind)
    )
      clearSelection();
    setActiveTool(resolved);
  };
}
