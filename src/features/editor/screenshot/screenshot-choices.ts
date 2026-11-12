// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ScreenshotTool } from "./screenshot-column";
import type { EditorMode } from "../toolbar/editor-modes";

/**
 * Taking up a screenshot's tools and modes, and putting them down: the twin
 * of the recording's `useRecordingColumn`. Select on a chosen layer is the
 * Screenshot mode, so putting a tool down lets the layer go as well.
 */
export function screenshotChoices({
  clearAnnotations,
  newestItemId,
  onSelectedItemChange,
  selectedItemId,
  setTool,
  tool,
}: {
  clearAnnotations: () => void;
  /** The layer a tool takes in hand where none is chosen. */
  newestItemId: number | null;
  selectedItemId: number | null;
  setTool: (tool: ScreenshotTool) => void;
  tool: ScreenshotTool;
  onSelectedItemChange?: (id: number | null) => void;
}) {
  // Select first, then nothing chosen: taken up, Select would bring back the
  // annotations it held when it was put down (`useSelectToolMemory`). The
  // layer goes too, whether chosen by hand or by the tool that took one up.
  const backToSelect = () => {
    setTool("select");
    clearAnnotations();
    onSelectedItemChange?.(null);
  };
  // A tool that draws on a layer takes one in hand, the newest where none
  // is chosen.
  const takeUp = (next: ScreenshotTool) => {
    if (next !== "select" && next !== "canvas" && selectedItemId === null)
      onSelectedItemChange?.(newestItemId);
    setTool(next);
  };
  // The tool in hand taken up again puts it down, as its button does: Select
  // with nothing chosen, not Select over the layer the tool took in hand,
  // which would be the Screenshot mode.
  const toggle = (next: ScreenshotTool) => {
    if (tool === next) backToSelect();
    else takeUp(next);
  };
  const chooseMode = (next: EditorMode | null) => {
    if (next === null) backToSelect();
    else if (next === "frame") takeUp("canvas");
    else {
      // The Screenshot mode is Select on a layer, the newest where none is
      // chosen, with the annotations let go as any mode lets them go.
      if (selectedItemId === null) onSelectedItemChange?.(newestItemId);
      setTool("select");
      clearAnnotations();
    }
  };
  return { backToSelect, chooseMode, takeUp, toggle };
}
