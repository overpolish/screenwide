// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

/** Apply a one-time native fit. Omit width to reset to the full viewport. */
export const resetRecordingPreviewView = (
  sessionId: number,
  fitWidth?: number,
) => invoke<null>("reset_recording_preview_view", { fitWidth, sessionId });

export const resetScreenshotPreviewView = (
  sessionId: number,
  fitWidth?: number,
) => invoke<null>("reset_screenshot_preview_view", { fitWidth, sessionId });
