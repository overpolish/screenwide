// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  currentMonitor,
  getCurrentWindow,
  LogicalSize,
} from "@tauri-apps/api/window";
import { useLayoutEffect, useRef } from "react";

import { ToolPanel } from "../editor/tool-panels/tool-panel";

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
    const window = getCurrentWindow();
    let cancelled = false;

    const resize = async () => {
      const maximumHeight = await maximumToolPanelHeight();
      const scaleFactor = await window.scaleFactor();
      const currentSize = (await window.innerSize()).toLogical(scaleFactor);
      if (cancelled) return;

      const measured = panel.getBoundingClientRect().height;
      const height =
        maximumHeight === null ? measured : Math.min(measured, maximumHeight);
      if (height <= 0) return;
      await window.setSize(new LogicalSize(currentSize.width, height));
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
