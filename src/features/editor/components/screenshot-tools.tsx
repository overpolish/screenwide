// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CircleDotDashed, Crop, MousePointer2, ScanSquare } from "lucide-react";
import { ReactNode, useCallback, useMemo, useRef } from "react";

import { ButtonGroup } from "../../../components/base/button-group/button-group";

import { PreviewToolToggle } from "./preview-tool-toggle";

export type ScreenshotTool = "canvas" | "crop" | "recenter" | "select" | null;

type ScreenshotToolActions = {
  /** The layer a crop falls back to when nothing is selected. */
  newestItemId: number | null;
  selectedItemId: number | null;
  setRecenterSelected: (selected: boolean) => void;
  setTool: (tool: ScreenshotTool) => void;
  tool: ScreenshotTool;
  onSelectedItemChange?: (itemId: number | null) => void;
};

/**
 * The screenshot workspace's view tools, as one element for the title bar to
 * carry.
 *
 * The actions are held behind a ref so the element survives a zoom, a nudge or
 * a resize draft: only choosing a tool rebuilds it, and the title bar above
 * re-renders no more often than the tools actually change.
 */
export function useScreenshotTools({
  newestItemId,
  onSelectedItemChange,
  selectedItemId,
  setRecenterSelected,
  setTool,
  tool,
}: ScreenshotToolActions): ReactNode {
  const actionsRef = useRef({
    newestItemId,
    onSelectedItemChange,
    selectedItemId,
    setRecenterSelected,
    setTool,
  });
  actionsRef.current = {
    newestItemId,
    onSelectedItemChange,
    selectedItemId,
    setRecenterSelected,
    setTool,
  };
  const chooseSelectTool = useCallback((selected: boolean) => {
    actionsRef.current.setTool(selected ? "select" : null);
  }, []);
  const chooseCanvasTool = useCallback((selected: boolean) => {
    actionsRef.current.setTool(selected ? "canvas" : null);
  }, []);
  const chooseCropTool = useCallback((selected: boolean) => {
    const actions = actionsRef.current;
    if (actions.selectedItemId === null)
      actions.onSelectedItemChange?.(actions.newestItemId);
    actions.setTool(selected ? "crop" : null);
  }, []);
  const chooseRecenterTool = useCallback((selected: boolean) => {
    actionsRef.current.setRecenterSelected(selected);
  }, []);

  return useMemo(
    () => (
      <ButtonGroup aria-label="View tools" className="gap-control">
        {/* The marker is the anchor Select's panel hangs from: taking the
            tool up opens it, however the tool was taken up. */}
        <span className="inline-flex" data-editor-tool="select">
          <PreviewToolToggle
            isSelected={tool === "select"}
            label="Select"
            name="Select screenshot"
            onSelectedChange={chooseSelectTool}
            shortcut="V"
          >
            <MousePointer2 />
          </PreviewToolToggle>
        </span>
        <PreviewToolToggle
          isSelected={tool === "canvas"}
          label="Resize canvas"
          name="Resize canvas"
          onSelectedChange={chooseCanvasTool}
          shortcut="F"
        >
          <ScanSquare />
        </PreviewToolToggle>
        <PreviewToolToggle
          isSelected={tool === "crop"}
          label="Crop"
          name="Crop screenshot"
          onSelectedChange={chooseCropTool}
          shortcut="C"
        >
          <Crop />
        </PreviewToolToggle>
        <PreviewToolToggle
          isSelected={tool === "recenter"}
          label="Recenter"
          name="Recenter screenshot"
          onSelectedChange={chooseRecenterTool}
          shortcut="R"
        >
          <CircleDotDashed />
        </PreviewToolToggle>
      </ButtonGroup>
    ),
    [
      chooseCanvasTool,
      chooseCropTool,
      chooseRecenterTool,
      chooseSelectTool,
      tool,
    ],
  );
}
