// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  createResizeTimer,
  readyLayoutTiming,
  type ReadyLayoutTiming,
} from "../preview/resize-timing";

import { layoutScreenshotPreviewSurface } from "./screenshot-preview-api";

type ScreenshotLayout = Parameters<typeof layoutScreenshotPreviewSurface>[0];

/**
 * Sends the screenshot preview's layouts one at a time, the newest waiting
 * while one is on its way, and a layout the same as the last left out.
 */
export function createScreenshotLayoutQueue() {
  let disposed = false;
  let inFlight = false;
  let last = "";
  let pending: {
    layout: ScreenshotLayout;
    ready: ReadyLayoutTiming;
  } | null = null;
  // TEMPORARY(#12)
  const resizeTimer = createResizeTimer();

  const flush = () => {
    if (disposed || inFlight || !pending) return;
    const { layout, ready } = pending;
    pending = null;
    inFlight = true;
    const { settled, timing } = resizeTimer.send(ready);
    void layoutScreenshotPreviewSurface({ ...layout, resizeTiming: timing })
      .catch(() => undefined)
      .finally(() => {
        settled();
        inFlight = false;
        flush();
      });
  };

  return {
    dispose: () => {
      disposed = true;
    },
    /** `measureStartedAt` is when measuring the page for it began. */
    queue: (layout: ScreenshotLayout, measureStartedAt: number) => {
      const next = JSON.stringify(layout);
      if (next === last) return;
      last = next;
      pending = {
        layout,
        ready: readyLayoutTiming(next.length, measureStartedAt),
      };
      flush();
    },
  };
}
