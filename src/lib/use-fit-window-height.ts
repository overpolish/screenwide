// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { LogicalSize, getCurrentWindow } from "@tauri-apps/api/window";
import { RefObject, useEffect } from "react";

type FitAxes = { height: boolean; width: boolean };

/**
 * Keeps the current window exactly the measured element's size on the chosen
 * axes, the way a native panel is sized to its content. An axis left out
 * stays as configured. Measured rather than assumed, so the configured size
 * is only a start.
 */
function useFitWindow(
  contentRef: RefObject<HTMLElement | null>,
  axes: FitAxes,
) {
  const { height: fitHeight, width: fitWidth } = axes;
  useEffect(() => {
    const element = contentRef.current;
    if (!element || !isTauri()) return;
    const appWindow = getCurrentWindow();
    let frame = 0;
    let lastWidth = 0;
    let lastHeight = 0;

    const fit = () => {
      frame = 0;
      const rect = element.getBoundingClientRect();
      const width = Math.ceil(rect.width);
      const height = Math.ceil(rect.height);
      if (width === 0 || height === 0) return;
      if (
        (!fitWidth || width === lastWidth) &&
        (!fitHeight || height === lastHeight)
      ) {
        return;
      }
      lastWidth = width;
      lastHeight = height;
      appWindow
        .innerSize()
        .then(async (size) => {
          const scale = await appWindow.scaleFactor();
          const current = size.toLogical(scale);
          await appWindow.setSize(
            new LogicalSize(
              fitWidth ? width : current.width,
              fitHeight ? height : current.height,
            ),
          );
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
  }, [contentRef, fitHeight, fitWidth]);
}

/** Fits the window's height to the element; the width stays as configured. */
export function useFitWindowHeight(contentRef: RefObject<HTMLElement | null>) {
  useFitWindow(contentRef, { height: true, width: false });
}

/** Fits the window's width to the element; the height stays as configured.
 * A bar whose controls change label grows and shrinks at its trailing edge
 * rather than reserving room for its widest state. */
export function useFitWindowWidth(contentRef: RefObject<HTMLElement | null>) {
  useFitWindow(contentRef, { height: false, width: true });
}
