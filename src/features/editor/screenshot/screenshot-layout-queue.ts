// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { pageWindowSize } from "../preview/page-window-size";

import { layoutScreenshotPreviewSurface } from "./screenshot-preview-api";

type ScreenshotLayout = Parameters<typeof layoutScreenshotPreviewSurface>[0];

/**
 * Sends the screenshot preview's layouts one at a time, the newest waiting
 * while one is on its way, and a layout the same as the last left out. The
 * page is measured again (`measure`) on every resize of the window, which the
 * native side carries the viewport with until a layout measured at the
 * window's new size arrives.
 */
export function createScreenshotLayoutQueue(measure: {
  readonly current: () => void;
}) {
  const remeasure = () => {
    measure.current();
  };
  window.addEventListener("resize", remeasure);
  let disposed = false;
  let inFlight = false;
  let last = "";
  let pending: {
    layout: ScreenshotLayout;
    windowSize: [number, number];
  } | null = null;

  const flush = () => {
    if (disposed || inFlight || !pending) return;
    const { layout, windowSize } = pending;
    pending = null;
    inFlight = true;
    void layoutScreenshotPreviewSurface({ ...layout, windowSize })
      .catch(() => undefined)
      .finally(() => {
        inFlight = false;
        flush();
      });
  };

  return {
    dispose: () => {
      disposed = true;
      window.removeEventListener("resize", remeasure);
    },
    queue: (layout: ScreenshotLayout) => {
      const windowSize = pageWindowSize();
      // The window's size is part of what is sent: a resize that leaves the
      // viewport as it was still has to reach the native side.
      const next = JSON.stringify([layout, windowSize]);
      if (next === last) return;
      last = next;
      pending = { layout, windowSize };
      flush();
    },
  };
}
