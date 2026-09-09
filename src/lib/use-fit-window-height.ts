// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { LogicalSize, getCurrentWindow } from "@tauri-apps/api/window";
import { RefObject, useEffect } from "react";

/**
 * Keeps the current window exactly as tall as the measured element, the way
 * a native panel is sized to its content. The width stays as configured.
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
      const height = Math.ceil(element.getBoundingClientRect().height);
      if (height === 0 || height === lastHeight) return;
      lastHeight = height;
      appWindow
        .innerSize()
        .then(async (size) => {
          const scale = await appWindow.scaleFactor();
          const width = size.toLogical(scale).width;
          await appWindow.setSize(new LogicalSize(width, height));
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
