// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";

import { toolPanelSpacing, toolPanelWidth } from "../../popup-panel/layout";
import { growEditorForPanel } from "../api";

export const toolPanelGutter = toolPanelWidth + toolPanelSpacing;

/** At minimum width the workspace is width-limited. Adding only the gutter
 * grows the picture too, so request its height-limited width in the same move. */
export function panelGrowthDelta(
  viewportWidth: number,
  fitWidth: number,
  gutter: number,
) {
  return Number.isFinite(fitWidth) && fitWidth > 0
    ? Math.max(0, fitWidth + gutter - viewportWidth)
    : gutter;
}

/** Whether a viewport `viewportWidth` wide leaves the picture its full-height
 * `fitWidth` beside the panel. Where a display refused the growth that would
 * make that room, the picture stays centred in the whole viewport rather
 * than squeezed off-centre beside the panel. An unmeasured picture asks for
 * no more than the panel's gutter. */
export function panelLeavesRoom(
  viewportWidth: number,
  fitWidth: number,
  gutter: number,
) {
  return (
    !(Number.isFinite(fitWidth) && fitWidth > 0) ||
    viewportWidth - gutter >= fitWidth - 0.5
  );
}

/** Grow once, waiting for the webview to receive its new bounds before fitting.
 * A constrained display can refuse growth; `panelLeavesRoom` then keeps the
 * picture centred. Closing a panel does not undo either the window size or
 * the captured fit. */
export async function openToolPanelSpace(delta: number) {
  if (delta <= 0) return;
  const window = getCurrentWindow();
  let finishResize: () => void = () => undefined;
  const resized = new Promise<void>((resolve) => {
    finishResize = resolve;
  });
  const unlisten = await window.onResized(finishResize);
  const timeout = setTimeout(finishResize, 500);
  try {
    if (await growEditorForPanel(window.label, delta)) await resized;
    // ResizeObserver and React marker layout run before the next measurement.
    await new Promise<void>((resolve) => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          resolve();
        });
      });
    });
  } finally {
    clearTimeout(timeout);
    unlisten();
  }
}
