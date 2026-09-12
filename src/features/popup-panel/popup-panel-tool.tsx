// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import { useLayoutEffect, useRef } from "react";

import { ToolPanel } from "../editor/tool-panels/tool-panel";

import { fitPopupPanel } from "./api";
import { popupPanelSpacing } from "./layout";
import { PopupPanelToolContent } from "./store";

/** The tallest a tool panel may grow: the work area less a section of
 * breathing room top and bottom, so it never runs to the screen edge. */
const maximumToolPanelHeight = async () => {
  const monitor = await currentMonitor();
  if (!monitor) return null;
  const workArea = monitor.workArea.size.toLogical(monitor.scaleFactor);

  return Math.max(workArea.height - popupPanelSpacing * 2, 0);
};

/**
 * The panel as an editor tool's own controls.
 *
 * Unlike a list it has no ceiling of its own: it is as tall as the controls
 * it holds, up to what the screen can show.
 */
export function PopupPanelTool({
  content,
}: {
  content: PopupPanelToolContent;
}) {
  const contentRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    if (!contentRef.current) return;

    const panel = contentRef.current;
    const { label } = getCurrentWindow();
    let cancelled = false;

    // The window opened unseen at a guessed height: the fit is what sizes it
    // to these controls and lets it be seen, in that order.
    const resize = async () => {
      const maximumHeight = await maximumToolPanelHeight();
      if (cancelled) return;

      const measured = panel.getBoundingClientRect().height;
      const height =
        maximumHeight === null ? measured : Math.min(measured, maximumHeight);
      if (height <= 0) return;
      await fitPopupPanel(height, label);
    };

    void resize();

    const observer = new ResizeObserver(() => {
      void resize();
    });
    observer.observe(panel);

    return () => {
      cancelled = true;
      observer.disconnect();
    };
  }, [content.tool, content.workspace]);

  return (
    <div className="w-full" ref={contentRef}>
      <ToolPanel tool={content.tool} workspace={content.workspace} />
    </div>
  );
}
