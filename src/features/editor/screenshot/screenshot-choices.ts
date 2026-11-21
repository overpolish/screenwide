// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ScreenshotTool } from "./screenshot-column";
import type { EditorMode } from "../toolbar/editor-modes";
import type { EditorTool } from "../toolbar/editor-tool-column";

/**
 * Taking up a screenshot's tools and modes, and putting them down: the twin
 * of the recording's `useRecordingColumn`. Select on a chosen layer is the
 * Screenshot mode. Select taken up keeps the layer chosen, as the crop put
 * away does; Escape, a press on nothing and ⌘D let it go.
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
  // Select taken up, by its button, V, or the crop put away by Done,
  // Return, Escape or a press past its picture: the layer stays chosen, so
  // the Screenshot mode opens on it.
  const selectTool = () => {
    setTool("select");
    clearAnnotations();
  };
  // A tool that draws on a layer takes one in hand, the newest where none
  // is chosen.
  const takeUp = (next: ScreenshotTool) => {
    if (next !== "select" && next !== "canvas" && selectedItemId === null)
      onSelectedItemChange?.(newestItemId);
    setTool(next);
  };
  // The tool in hand taken up again puts it down, as its button does: the
  // crop with its layer still chosen, a pen with nothing chosen, not Select
  // over the layer the pen took in hand, which would be the Screenshot mode.
  const toggle = (next: ScreenshotTool) => {
    if (tool !== next) takeUp(next);
    else if (next === "crop") selectTool();
    else backToSelect();
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
  return {
    backToSelect,
    chooseMode,
    /** The crop of `itemId`, double-clicked on the picture, taken up, or
     * for none, a press past the picture being cropped, put away. */
    cropLayer: (itemId: number | null) => {
      if (itemId === null) {
        if (tool === "crop") selectTool();
        return;
      }
      if (itemId !== selectedItemId) onSelectedItemChange?.(itemId);
      if (tool !== "crop") setTool("crop");
    },
    /** Escape: the crop put away with its layer chosen, anything else put
     * down with nothing chosen. */
    deselect: () => {
      if (tool === "crop") selectTool();
      else backToSelect();
    },
    /** A tool taken up from the column: Select keeping the layer, or a
     * pen. */
    fromColumn: (next: EditorTool) => {
      if (next === "select") selectTool();
      else takeUp(next);
    },
    selectTool,
    takeUp,
    toggle,
  };
}
