// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listMonitorThumbnails } from "./api";
import { useRecordingSourceStore } from "./store";

/** How often a surface showing the stills takes new ones, so a display's
 * picture follows what is on it rather than freezing at the first capture. */
export const MONITOR_THUMBNAIL_INTERVAL_MS = 1_500;

/**
 * Captures a fresh still of every attached display into the store. The stills
 * are decoration, so a failed capture is logged and otherwise ignored: the
 * surfaces that draw them fall back to their glyph or fill.
 */
export const refreshMonitorThumbnails = async () => {
  try {
    const thumbnails = await listMonitorThumbnails();
    useRecordingSourceStore.getState().setMonitorThumbnails(thumbnails);
  } catch (error) {
    console.error("Could not capture the display thumbnails", error);
  }
};
