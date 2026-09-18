// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop, MousePointer2, ScanSquare } from "lucide-react";
import { ReactNode, useCallback, useMemo, useRef } from "react";

import { ButtonGroup } from "../../../components/base/button-group/button-group";
import {
  ArrowToolIcon,
  CounterToolIcon,
} from "../../../components/shared/annotation-style/annotation-tool-icons";
import { ToolToggle } from "../../../components/shared/tool-toggle/tool-toggle";

export type ScreenshotTool =
  "arrow" | "canvas" | "counter" | "crop" | "select" | null;

type ScreenshotToolActions = {
  /** The layer a crop falls back to when nothing is selected. */
  newestItemId: number | null;
  selectedItemId: number | null;
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
  setTool,
  tool,
}: ScreenshotToolActions): ReactNode {
  const actionsRef = useRef({
    newestItemId,
    onSelectedItemChange,
    selectedItemId,
    setTool,
  });
  actionsRef.current = {
    newestItemId,
    onSelectedItemChange,
    selectedItemId,
    setTool,
  };
  const chooseSelectTool = useCallback((selected: boolean) => {
    actionsRef.current.setTool(selected ? "select" : null);
  }, []);
  const chooseCanvasTool = useCallback((selected: boolean) => {
    actionsRef.current.setTool(selected ? "canvas" : null);
  }, []);
  // The arrow is drawn on a layer, so the tool needs one in hand the way the
  // crop does.
  const chooseArrowTool = useCallback((selected: boolean) => {
    const actions = actionsRef.current;
    if (actions.selectedItemId === null)
      actions.onSelectedItemChange?.(actions.newestItemId);
    actions.setTool(selected ? "arrow" : null);
  }, []);
  const chooseCounterTool = useCallback((selected: boolean) => {
    const actions = actionsRef.current;
    if (actions.selectedItemId === null)
      actions.onSelectedItemChange?.(actions.newestItemId);
    actions.setTool(selected ? "counter" : null);
  }, []);
  const chooseCropTool = useCallback((selected: boolean) => {
    const actions = actionsRef.current;
    if (actions.selectedItemId === null)
      actions.onSelectedItemChange?.(actions.newestItemId);
    actions.setTool(selected ? "crop" : null);
  }, []);

  return useMemo(
    () => (
      <ButtonGroup aria-label="View tools" className="gap-control">
        {/* The marker is the anchor Select's panel hangs from: taking the
            tool up opens it, however the tool was taken up. */}
        <span className="inline-flex" data-editor-tool="select">
          <ToolToggle
            isSelected={tool === "select"}
            label="Select"
            name="Select screenshot"
            onSelectedChange={chooseSelectTool}
            shortcut="V"
          >
            <MousePointer2 />
          </ToolToggle>
        </span>
        <ToolToggle
          isSelected={tool === "canvas"}
          label="Resize canvas"
          name="Resize canvas"
          onSelectedChange={chooseCanvasTool}
          shortcut="F"
        >
          <ScanSquare />
        </ToolToggle>
        <ToolToggle
          isSelected={tool === "crop"}
          label="Crop"
          name="Crop screenshot"
          onSelectedChange={chooseCropTool}
          shortcut="C"
        >
          <Crop />
        </ToolToggle>
        <ToolToggle
          isSelected={tool === "arrow"}
          label="Arrow"
          name="Draw an arrow"
          onSelectedChange={chooseArrowTool}
          shortcut="A"
        >
          <ArrowToolIcon />
        </ToolToggle>
        <ToolToggle
          isSelected={tool === "counter"}
          label="Counter"
          name="Drop a counter"
          onSelectedChange={chooseCounterTool}
          shortcut="N"
        >
          <CounterToolIcon />
        </ToolToggle>
      </ButtonGroup>
    ),
    [
      chooseArrowTool,
      chooseCanvasTool,
      chooseCounterTool,
      chooseCropTool,
      chooseSelectTool,
      tool,
    ],
  );
}
