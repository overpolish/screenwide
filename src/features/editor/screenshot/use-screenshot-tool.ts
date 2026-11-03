// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { useSelectToolMemory } from "../annotations/use-select-tool-memory";
import { toolDisagreesWithAnnotation } from "../annotations/use-tool-follows-annotation";
import { isAnnotationTool } from "../tools/tool-registry";

import { ScreenshotTool } from "./screenshot-column";

import type { AnnotationKind } from "../../annotations/types";

/** Picking a screenshot tool up, and what that clears on its way in. The twin
 * of `useRecordingPreviewCanvasTool`'s own `changeCanvasTool`. */
export function useScreenshotTool({
  chooseSelection,
  clearSelection,
  selectedIds,
  selectedKind,
  setActiveTool,
  tool,
}: {
  /** Chooses exactly these annotations. */
  chooseSelection: (ids: readonly string[]) => void;
  clearSelection: () => void;
  selectedIds: ReadonlySet<string>;
  selectedKind: AnnotationKind | null;
  setActiveTool: (tool: ScreenshotTool) => void;
  tool: ScreenshotTool;
}) {
  const toolRef = useRef(tool);
  toolRef.current = tool;
  const rememberSelection = useSelectToolMemory(selectedIds, chooseSelection);
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
    // A mode (crop, canvas) lets the annotations in hand go, and putting it
    // away does not bring them back.
    rememberSelection(
      toolRef.current,
      resolved,
      resolved === "canvas" || resolved === "crop",
    );
  };
}
