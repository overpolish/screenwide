// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { ScreenshotTool } from "./screenshot-tools";
import { toolDisagreesWithAnnotation } from "./use-tool-follows-annotation";

/** Picking a screenshot tool up, and what that clears on its way in. The twin
 * of `useRecordingPreviewCanvasTool`'s own `changeCanvasTool`. */
export function useScreenshotTool({
  clearSelection,
  selectedKind,
  setActiveTool,
  tool,
}: {
  clearSelection: () => void;
  selectedKind: "arrow" | "counter" | null;
  setActiveTool: (tool: ScreenshotTool) => void;
  tool: ScreenshotTool;
}) {
  const toolRef = useRef(tool);
  toolRef.current = tool;
  return (
    next: ScreenshotTool | ((current: ScreenshotTool) => ScreenshotTool),
  ) => {
    const resolved = typeof next === "function" ? next(toolRef.current) : next;
    // A tool that draws neither shape lets the annotation in hand go, and so
    // does the tool that draws the other one.
    if (
      (resolved !== "arrow" &&
        resolved !== "counter" &&
        resolved !== "select") ||
      toolDisagreesWithAnnotation(resolved, selectedKind)
    )
      clearSelection();
    setActiveTool(resolved);
  };
}
