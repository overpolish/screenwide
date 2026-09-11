// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { activePopupPanel, usePopupPanelStore } from "../../popup-panel/store";
import { usePreviewFit } from "../components/preview-fit-context";
import { EditorKind } from "../types";

import { toolPanelLabel } from "./tool-panel-window";
import { EditorToolId, toolPanel, toolResetsView } from "./tool-registry";
import { previewViewport, useToolPanel } from "./use-tool-panel";

/** The toolbar button a tool's panel hangs from, for the tools that mark one.
 * Without it the panel is anchored on the picture itself. */
const toolTrigger = (tool: EditorToolId) =>
  document
    .querySelector<HTMLElement>(`[data-editor-tool="${tool}"]`)
    ?.getBoundingClientRect();

/**
 * The panel of the tool in hand, and no other.
 *
 * A tool's panel is open exactly while that tool is active: taking a tool up
 * shows its controls, swapping tools swaps the panel in place, and a tool
 * without controls leaves the picture clear. The workspaces state which tool
 * is in hand and nothing else; every open and close is decided here, so a
 * toolbar button, a shortcut and the tool a session starts in all behave the
 * same way.
 *
 * The exception is the tools that are only a panel - Cursor today. Opening one
 * retires the canvas tool, and that null must not take the panel it just
 * opened away again: a cleared tool only ever closes its own panel.
 */
export function useToolPanelFollowsTool(
  workspace: EditorKind,
  tool: EditorToolId | null,
) {
  const { close, openPanel } = useToolPanel(workspace);
  const { fitPreview } = usePreviewFit();
  // The tool the panel was last settled against. `undefined` until the first
  // pass, so the tool a session starts in opens its panel like any other.
  const settledRef = useRef<EditorToolId | null | undefined>(undefined);

  useEffect(() => {
    if (settledRef.current === tool) return;
    let disposed = false;
    let frame = 0;

    const settle = async (previous: EditorToolId | null | undefined) => {
      if (tool === null) {
        const previousPanel =
          previous === null || previous === undefined
            ? undefined
            : toolPanel(previous);
        const open = activePopupPanel(
          usePopupPanelStore.getState(),
          toolPanelLabel(workspace),
        )?.content;
        if (
          previousPanel !== undefined &&
          open?.kind === "tool" &&
          open.tool === previousPanel &&
          open.workspace === workspace
        )
          await close();
        return;
      }
      const panel = toolPanel(tool);
      if (panel === undefined) {
        await close();
        if (toolResetsView(tool)) fitPreview();
        return;
      }
      const anchor =
        toolTrigger(tool) ?? previewViewport()?.getBoundingClientRect();
      if (!anchor) return;
      await openPanel(panel, anchor, toolResetsView(tool));
    };

    // A panel is placed against the picture, so there is nowhere to put one
    // until the viewport has been laid out. The tool a session starts in is in
    // hand before that happens, and its panel opens as soon as there is a
    // corner to hang it from.
    const whenPlaceable = () => {
      if (disposed) return;
      if (!previewViewport()) {
        frame = requestAnimationFrame(whenPlaceable);
        return;
      }
      const previous = settledRef.current;
      settledRef.current = tool;
      void settle(previous);
    };
    whenPlaceable();

    return () => {
      disposed = true;
      cancelAnimationFrame(frame);
    };
  }, [close, fitPreview, openPanel, tool, workspace]);
}
