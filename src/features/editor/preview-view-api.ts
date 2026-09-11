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

/** Move the basis a double-click reset returns to, without moving the view.
 * Omit the width to hand the basis back to the full viewport. */
export const setRecordingPreviewFitBasis = (
  sessionId: number,
  fitWidth?: number,
) => invoke<null>("set_recording_preview_fit_basis", { fitWidth, sessionId });

export const setScreenshotPreviewFitBasis = (
  sessionId: number,
  fitWidth?: number,
) => invoke<null>("set_screenshot_preview_fit_basis", { fitWidth, sessionId });
