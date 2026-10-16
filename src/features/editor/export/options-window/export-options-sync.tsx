// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import { synchronizeExportPreviewStore } from "../preview/export-preview-store";

import { synchronizeExportOptionsStore } from "./store";

/**
 * Keeps the export settings mirror, and the preview frame the editor makes,
 * in step across windows.
 *
 * Mounted app-wide rather than per window: the editor publishes and the
 * options window reads, and each of them also receives what the other wrote.
 */
export function ExportOptionsSync() {
  useEffect(() => {
    window.addEventListener("storage", synchronizeExportOptionsStore);
    window.addEventListener("storage", synchronizeExportPreviewStore);
    return () => {
      window.removeEventListener("storage", synchronizeExportOptionsStore);
      window.removeEventListener("storage", synchronizeExportPreviewStore);
    };
  }, []);

  return null;
}
