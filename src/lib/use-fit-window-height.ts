// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { LogicalSize, getCurrentWindow } from "@tauri-apps/api/window";
import { RefObject, useEffect } from "react";

/**
 * Keeps the current window exactly the measured element's height, the way a
 * native panel is sized to its content; the width stays as configured.
 * Measured rather than assumed, so the configured height is only a start.
 */
export function useFitWindowHeight(contentRef: RefObject<HTMLElement | null>) {
  useEffect(() => {
    const element = contentRef.current;
    if (!element || !isTauri()) return;
    const appWindow = getCurrentWindow();
    let frame = 0;
    let lastHeight = 0;

    const fit = () => {
      frame = 0;
      const rect = element.getBoundingClientRect();
      const height = Math.ceil(rect.height);
      if (rect.width === 0 || height === 0 || height === lastHeight) return;
      lastHeight = height;
      appWindow
        .innerSize()
        .then(async (size) => {
          const scale = await appWindow.scaleFactor();
          const current = size.toLogical(scale);
          await appWindow.setSize(new LogicalSize(current.width, height));
        })
        .catch((cause: unknown) => {
          console.error("Could not fit the window to its content", cause);
        });
    };

    // One resize per frame: content can settle over several layouts.
    const observer = new ResizeObserver(() => {
      frame ||= window.requestAnimationFrame(fit);
    });
    observer.observe(element);
    fit();

    return () => {
      observer.disconnect();
      if (frame !== 0) window.cancelAnimationFrame(frame);
    };
  }, [contentRef]);
}
