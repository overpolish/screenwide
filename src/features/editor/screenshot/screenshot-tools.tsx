// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop, MousePointer2, SquareDashedMousePointer } from "lucide-react";
import { ReactNode, useCallback, useMemo, useRef } from "react";

import { t } from "../../../i18n/i18n";
import { Toolbar } from "../../../primitives/toolbar/toolbar";
import { ToolbarTool } from "../../../primitives/toolbar/toolbar-tool";
import { FrameToolIcon } from "../glyphs/frame-tool-icon";
import { AnnotationToolStrip } from "../toolbar/annotation-tool-strip";

import type { AnnotationKind } from "../../annotations/types";

export type ScreenshotTool =
  AnnotationKind | "canvas" | "crop" | "marquee" | "select" | null;

type ScreenshotToolActions = {
  /** The layer a crop falls back to when nothing is selected. */
  newestItemId: number | null;
  selectedItemId: number | null;
  setTool: (tool: ScreenshotTool) => void;
  tool: ScreenshotTool;
  onSelectedItemChange?: (itemId: number | null) => void;
};

/**
 * The screenshot workspace's tools - pointer, canvas and annotation groups -
 * as one element for the title bar to carry.
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
  // The marquee chooses annotations on a layer, so it takes the newest layer
  // when nothing is selected, as a drawing tool does.
  const chooseMarqueeTool = useCallback((selected: boolean) => {
    const actions = actionsRef.current;
    if (actions.selectedItemId === null)
      actions.onSelectedItemChange?.(actions.newestItemId);
    actions.setTool(selected ? "marquee" : null);
  }, []);
  const chooseCanvasTool = useCallback((selected: boolean) => {
    actionsRef.current.setTool(selected ? "canvas" : null);
  }, []);
  // A drawing tool needs a layer in hand the way the crop does, so each one
  // takes the newest layer when nothing is selected.
  const chooseAnnotationTool = useCallback(
    (id: AnnotationKind, selected: boolean) => {
      const actions = actionsRef.current;
      if (actions.selectedItemId === null)
        actions.onSelectedItemChange?.(actions.newestItemId);
      actions.setTool(selected ? id : null);
    },
    [],
  );
  const chooseCropTool = useCallback((selected: boolean) => {
    const actions = actionsRef.current;
    if (actions.selectedItemId === null)
      actions.onSelectedItemChange?.(actions.newestItemId);
    actions.setTool(selected ? "crop" : null);
  }, []);

  return useMemo(
    () => (
      <>
        <Toolbar aria-label={t("editor-pointer-tools")}>
          {/* The marker is the anchor Select's panel hangs from: taking the
            tool up opens it, however the tool was taken up. */}
          <span className="inline-flex" data-editor-tool="select">
            <ToolbarTool
              aria-label={t("editor-tool-select-name")}
              isSelected={tool === "select"}
              label={t("editor-tool-select")}
              onChange={chooseSelectTool}
              shortcut="V"
            >
              <MousePointer2 />
            </ToolbarTool>
          </span>
          <ToolbarTool
            aria-label={t("editor-tool-marquee-name")}
            isSelected={tool === "marquee"}
            label={t("editor-tool-marquee")}
            onChange={chooseMarqueeTool}
            shortcut="G"
          >
            <SquareDashedMousePointer />
          </ToolbarTool>
        </Toolbar>
        <Toolbar aria-label={t("editor-canvas-tools")}>
          <ToolbarTool
            aria-label={t("editor-tool-canvas")}
            isSelected={tool === "canvas"}
            label={t("editor-tool-canvas")}
            onChange={chooseCanvasTool}
            shortcut="F"
          >
            <FrameToolIcon />
          </ToolbarTool>
          <ToolbarTool
            aria-label={t("editor-tool-crop-name")}
            isSelected={tool === "crop"}
            label={t("editor-tool-crop")}
            onChange={chooseCropTool}
            shortcut="C"
          >
            <Crop />
          </ToolbarTool>
        </Toolbar>
        <AnnotationToolStrip onChoose={chooseAnnotationTool} tool={tool} />
      </>
    ),
    [
      chooseAnnotationTool,
      chooseCanvasTool,
      chooseCropTool,
      chooseMarqueeTool,
      chooseSelectTool,
      tool,
    ],
  );
}
