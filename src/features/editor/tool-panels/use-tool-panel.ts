// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import { useCallback } from "react";

import { hidePopupPanel, showPopupPanel } from "../../popup-panel/api";
import {
  initialToolPanelHeight,
  popupPanelSpacing,
  toolPanelWidth,
} from "../../popup-panel/layout";
import { ToolPanelKind, usePopupPanelStore } from "../../popup-panel/store";
import { usePreviewFit } from "../components/preview-fit-context";
import { EditorKind } from "../types";

import {
  openToolPanelSpace,
  panelGrowthDelta,
  toolPanelGutter,
} from "./tool-panel-space";
import { toolPanelTitles } from "./tool-panel-titles";
import { EditorToolId, toolPanel, toolResetsView } from "./tool-registry";

/** Both workspaces mark their preview area with this, and a tool panel is
 * placed against it rather than against the button that opened it. */
const PREVIEW_VIEWPORT_SELECTOR = "[data-recording-preview-viewport]";

const toolPanelId = (tool: ToolPanelKind) => `tool:${tool}`;

export const previewViewport = () =>
  document.querySelector<HTMLElement>(PREVIEW_VIEWPORT_SELECTOR);

/** The panel's top left in the parent window's content coordinates: the top
 * right of the picture, where it covers the least of it. */
export const panelOffset = (bounds: DOMRect) =>
  new LogicalPosition(
    Math.max(
      bounds.right - toolPanelWidth - popupPanelSpacing,
      popupPanelSpacing,
    ),
    bounds.top + popupPanelSpacing,
  );

/**
 * Opening and closing the editor's tool panels, and what the preview does
 * about it.
 *
 * The panel is the one panel window, so opening a second tool replaces the
 * first in place. Which tool is up is read back out of the panel store, so it
 * survives the panel being closed from anywhere else: Escape, the editor
 * being put away, or another window opening a list in the same panel.
 *
 * Opted-in tools grow and fit once on opening. Panel visibility and later
 * resizes never reserve layout space or reset the user's transform.
 */
export function useToolPanel(workspace: EditorKind) {
  const active = usePopupPanelStore((state) => state.active);
  const openTool = active?.content.kind === "tool" ? active.content.tool : null;
  const { fitPreview, setFitBasis } = usePreviewFit();

  /** Puts the open panel away without resetting the current zoom or pan. The
   * view stays where it is, but the basis a double-click resets to goes back
   * to the full viewport the panel is no longer taking a bite out of. */
  const close = useCallback(async () => {
    const current = usePopupPanelStore.getState().active;
    if (current?.content.kind !== "tool") return;
    usePopupPanelStore.getState().close();
    setFitBasis();
    await hidePopupPanel();
  }, [setFitBasis]);

  const openPanel = useCallback(
    async (tool: ToolPanelKind, anchor: DOMRect, fitsView: boolean) => {
      const id = toolPanelId(tool);
      if (fitsView) {
        const viewport = previewViewport();
        await openToolPanelSpace(
          panelGrowthDelta(
            viewport?.getBoundingClientRect().width ?? anchor.width,
            Number(viewport?.dataset.previewFitWidth),
            toolPanelGutter,
          ),
        );
      }
      const bounds = previewViewport()?.getBoundingClientRect() ?? anchor;
      if (fitsView) fitPreview(Math.max(1, bounds.width - toolPanelGutter));
      usePopupPanelStore.getState().open({
        content: { kind: "tool", tool, workspace },
        focusContents: false,
        id,
        label: toolPanelTitles[tool],
      });
      await showPopupPanel({
        anchor: {
          height: anchor.height,
          width: anchor.width,
          x: anchor.left,
          y: anchor.top,
        },
        focusContents: false,
        offset: panelOffset(bounds),
        parentWindowLabel: getCurrentWindow().label,
        size: new LogicalSize(toolPanelWidth, initialToolPanelHeight),
        // The editor keeps working while a tool panel is up, so a press
        // outside it must not take it away.
        sticky: true,
        triggerId: id,
      });
    },
    [fitPreview, workspace],
  );

  /**
   * A press on the button of a tool that owns a panel. A second press on the
   * same button puts the panel away; a press on another tool's swaps the
   * contents in place, applying only the destination tool's fit policy.
   */
  const toggle = useCallback(
    async (tool: EditorToolId, anchor: DOMRect) => {
      const panel = toolPanel(tool);
      const current = usePopupPanelStore.getState().active;
      const openContent =
        current?.content.kind === "tool" ? current.content : null;
      const isOpen =
        panel !== undefined &&
        current?.id === toolPanelId(panel) &&
        openContent;
      if (isOpen || panel === undefined) {
        await close();
      } else {
        await openPanel(panel, anchor, toolResetsView(tool));
      }
      if (!isOpen && panel === undefined && toolResetsView(tool)) fitPreview();
    },
    [close, fitPreview, openPanel],
  );

  /**
   * Activating a canvas tool replaces the panel. Panel activation already
   * clears the canvas tool; that null update must not close the new panel.
   * Only the destination can request a fit.
   */
  const select = useCallback(
    async (tool: EditorToolId | null) => {
      // Clearing canvas interaction is also part of activating a panel.
      if (tool === null) return;
      await close();
      if (toolResetsView(tool)) fitPreview();
    },
    [close, fitPreview],
  );

  return { close, openTool, select, toggle };
}
