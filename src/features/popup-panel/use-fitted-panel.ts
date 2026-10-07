// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { currentMonitor, getCurrentWindow } from "@tauri-apps/api/window";
import { RefObject, useLayoutEffect } from "react";

import { fitPopupPanel } from "./api";
import { popupPanelSpacing } from "./layout";

/** The tallest a fitted panel may grow: the work area less a section of
 * breathing room top and bottom, so it never runs to the screen edge. */
const maximumPanelHeight = async () => {
  const monitor = await currentMonitor();
  if (!monitor) return null;
  const workArea = monitor.workArea.size.toLogical(monitor.scaleFactor);

  return Math.max(workArea.height - popupPanelSpacing * 2, 0);
};

/**
 * Sizes the panel window this runs in to the content at `contentRef`, and
 * keeps it sized as that content changes. The window opened unseen at a
 * guessed height: the fit is what sizes it and lets it be seen, in that
 * order. `content` names what is shown, so a new one is measured afresh.
 */
export function useFittedPanel(
  contentRef: RefObject<HTMLElement | null>,
  content: unknown,
) {
  useLayoutEffect(() => {
    // A story has no panel window to size.
    if (!contentRef.current || !isTauri()) return;

    const panel = contentRef.current;
    const { label } = getCurrentWindow();
    let cancelled = false;

    const resize = async () => {
      const maximumHeight = await maximumPanelHeight();
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
  }, [content, contentRef]);
}
