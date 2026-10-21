// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { ShortcutAction } from "../../../bindings/ShortcutAction";

// The shortcuts that work anywhere, shared with Settings search.
export type GlobalShortcut = {
  action: ShortcutAction;
  label: string;
  description?: string;
};

export const recordingShortcuts = (): GlobalShortcut[] => [
  { action: "toggleRecordingBar", label: t("settings-shortcut-toggle-bar") },
  { action: "startStopRecording", label: t("settings-shortcut-start-stop") },
  {
    action: "pauseResumeRecording",
    label: t("settings-shortcut-pause-resume"),
  },
  {
    action: "saveReplay",
    description: t("settings-shortcut-save-replay-description"),
    label: t("settings-shortcut-save-replay"),
  },
];

export const screenshotShortcuts = (): GlobalShortcut[] => [
  {
    action: "takeScreenshot",
    description: t("settings-shortcut-screenshot-description"),
    label: t("settings-shortcut-screenshot"),
  },
  {
    action: "takeScreenshotToClipboard",
    description: t("settings-shortcut-copy-screenshot-description"),
    label: t("settings-shortcut-copy-screenshot"),
  },
];
