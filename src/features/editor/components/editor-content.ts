// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type RecordingOutputChange = (
  trackId: import("../types").RecordingVideoTrackId,
  settings: import("../screenshot-output").ScreenshotOutputSettings,
) => void;

export type ScreenshotOutputChange = (
  settings: import("../screenshot-output").ScreenshotOutputSettings,
  itemId?: number,
) => void;
