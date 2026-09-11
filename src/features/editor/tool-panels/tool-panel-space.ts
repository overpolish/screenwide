// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { getCurrentWindow } from "@tauri-apps/api/window";

import { popupPanelSpacing, toolPanelWidth } from "../../popup-panel/layout";
import { growEditorForPanel } from "../api";

export const toolPanelGutter = toolPanelWidth + popupPanelSpacing;

/** Grow once, waiting for the webview to receive its new bounds before fitting.
 * A constrained display can refuse growth; fitting then uses the space left.
 * Closing a panel does not undo either the window size or the captured fit. */
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
