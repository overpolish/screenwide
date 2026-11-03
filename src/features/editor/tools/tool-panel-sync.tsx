// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { synchronizeToolPanelStore } from "./tool-panel-store";

/**
 * Keeps the tool panel settings mirror in step across windows.
 *
 * Mounted app-wide rather than per window: the editor publishes and the panel
 * window reads, and each of them also receives what the other wrote.
 */
export function ToolPanelSync() {
  useEffect(() => {
    window.addEventListener("storage", synchronizeToolPanelStore);
    return () => {
      window.removeEventListener("storage", synchronizeToolPanelStore);
    };
  }, []);

  return null;
}
