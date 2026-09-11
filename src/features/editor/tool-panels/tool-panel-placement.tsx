// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect } from "react";

import { movePopupPanel } from "../../popup-panel/api";
import { activePopupPanel, usePopupPanelStore } from "../../popup-panel/store";
import { usePreviewFit } from "../components/preview-fit-context";
import { EditorKind } from "../types";

import { toolPanelGutter } from "./tool-panel-space";
import { toolPanelLabel } from "./tool-panel-window";
import { panelResetsView } from "./tool-registry";
import { panelOffset, previewViewport } from "./use-tool-panel";

/** Keeps the panel attached to the preview's corner as the window resizes.
 * Moving it never changes the preview fit, including when it overlaps. */
export function ToolPanelPlacement({ workspace }: { workspace: EditorKind }) {
  const panel = toolPanelLabel(workspace);
  const active = usePopupPanelStore((state) => activePopupPanel(state, panel));
  const openTool = active?.content.kind === "tool" ? active.content.tool : null;
  const { setFitBasis } = usePreviewFit();

  // A panel can also go away from outside the editor - Escape, the editor
  // being minimised. However it went, the reset basis is the whole viewport
  // again; the view itself never moves.
  useEffect(() => {
    if (openTool === null) setFitBasis();
  }, [openTool, setFitBasis]);

  // An editor resize moves the corner the panel hangs from. It is re-placed
  // rather than put away: only its position changed.
  useEffect(() => {
    if (openTool === null) return;
    const viewport = previewViewport();
    if (!viewport || typeof ResizeObserver === "undefined") return;
    let disposed = false;
    let inFlight = false;
    let pending: { bounds: DOMRect } | null = null;
    let frame = 0;
    const flush = () => {
      if (disposed || inFlight || pending === null) return;
      const next = pending;
      pending = null;
      inFlight = true;
      void movePopupPanel(
        getCurrentWindow().label,
        panelOffset(next.bounds),
        panel,
      )
        .catch(() => undefined)
        .finally(() => {
          inFlight = false;
          flush();
        });
    };
    const queueMove = () => {
      const bounds = previewViewport()?.getBoundingClientRect();
      if (!bounds) return;
      if (panelResetsView(openTool))
        setFitBasis(Math.max(1, bounds.width - toolPanelGutter));
      pending = { bounds };
      flush();
    };
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        queueMove();
      });
    });
    observer.observe(viewport);
    return () => {
      disposed = true;
      pending = null;
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  }, [openTool, panel, setFitBasis]);

  return null;
}
