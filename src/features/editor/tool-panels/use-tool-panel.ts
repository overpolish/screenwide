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
import {
  activePopupPanel,
  ToolPanelKind,
  usePopupPanelStore,
} from "../../popup-panel/store";
import { usePreviewFit } from "../components/preview-fit-context";
import { EditorKind } from "../types";

import {
  openToolPanelSpace,
  panelGrowthDelta,
  toolPanelGutter,
} from "./tool-panel-space";
import { toolPanelLabel } from "./tool-panel-window";
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
 * The panel is this workspace's own panel window, so opening a second tool
 * replaces the first in place while the other editor's panel stays as it is.
 * Which tool is up is read back out of the panel store, so it survives the
 * panel being closed from anywhere else: the editor being put away, or
 * Escape.
 *
 * A canvas tool never opens its own panel from here: `useToolPanelFollowsTool`
 * watches the tool in hand and keeps the panel matched to it. What is left
 * here is the panel-only tools' button behaviour and the primitives both use.
 *
 * Opted-in tools grow and fit once on opening. Panel visibility and later
 * resizes never reserve layout space or reset the user's transform.
 */
export function useToolPanel(workspace: EditorKind) {
  const panel = toolPanelLabel(workspace);
  const active = usePopupPanelStore((state) => activePopupPanel(state, panel));
  const openTool = active?.content.kind === "tool" ? active.content.tool : null;
  const { fitPreview, setFitBasis } = usePreviewFit();

  /** Puts the open panel away without resetting the current zoom or pan. The
   * view stays where it is, but the basis a double-click resets to goes back
   * to the full viewport the panel is no longer taking a bite out of. */
  const close = useCallback(async () => {
    const current = activePopupPanel(usePopupPanelStore.getState(), panel);
    if (current?.content.kind !== "tool") return;
    usePopupPanelStore.getState().close(panel);
    setFitBasis();
    await hidePopupPanel(false, panel);
  }, [panel, setFitBasis]);

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
      usePopupPanelStore.getState().open(panel, {
        content: { kind: "tool", tool, workspace },
        focusContents: false,
        id,
      });
      await showPopupPanel({
        anchor: {
          height: anchor.height,
          width: anchor.width,
          x: anchor.left,
          y: anchor.top,
        },
        fitted: true,
        focusContents: false,
        offset: panelOffset(bounds),
        panel,
        parentWindowLabel: getCurrentWindow().label,
        size: new LogicalSize(toolPanelWidth, initialToolPanelHeight),
        // The editor keeps working while a tool panel is up, so a press
        // outside it must not take it away.
        sticky: true,
        triggerId: id,
      });
    },
    [fitPreview, panel, workspace],
  );

  /**
   * A press on the button of a tool that owns a panel. A second press on the
   * same button puts the panel away; a press on another tool's swaps the
   * contents in place, applying only the destination tool's fit policy.
   */
  const toggle = useCallback(
    async (tool: EditorToolId, anchor: DOMRect) => {
      const kind = toolPanel(tool);
      const current = activePopupPanel(usePopupPanelStore.getState(), panel);
      const openContent =
        current?.content.kind === "tool" ? current.content : null;
      const isOpen =
        kind !== undefined && current?.id === toolPanelId(kind) && openContent;
      if (isOpen || kind === undefined) {
        await close();
      } else {
        await openPanel(kind, anchor, toolResetsView(tool));
      }
      if (!isOpen && kind === undefined && toolResetsView(tool)) fitPreview();
    },
    [close, fitPreview, openPanel, panel],
  );

  return { close, openPanel, openTool, toggle };
}
