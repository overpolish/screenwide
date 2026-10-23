// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { GeneralSettings } from "../../../bindings/GeneralSettings";

// The rows General draws from lists, shared with Settings search.

/** The preferences a Switch row can drive. */
type ToggleKey = {
  [K in keyof GeneralSettings]: GeneralSettings[K] extends boolean ? K : never;
}[keyof GeneralSettings];

export type ToggleItem = {
  key: ToggleKey;
  title: string;
  description?: string;
};

export const recordingToggle = (): ToggleItem => ({
  description: t("settings-confidence-checks-description"),
  key: "showRecordingConfidenceChecks",
  title: t("settings-confidence-checks"),
});

export const recordingSizeToggle = (): ToggleItem => ({
  description: t("settings-recording-size-description"),
  key: "showRecordingSize",
  title: t("settings-recording-size"),
});

export const autoZoomToggle = (): ToggleItem => ({
  description: t("settings-auto-zoom-description"),
  key: "autoZoom",
  title: t("settings-auto-zoom"),
});

export const captureToggle = (): ToggleItem => ({
  description: t("settings-include-app-description"),
  key: "recordScreenwideWindows",
  title: t("settings-include-app"),
});

export const startupToggles = (): ToggleItem[] => [
  { key: "launchAtLogin", title: t("settings-launch-at-login") },
  {
    key: "showRecordingControlsOnLaunch",
    title: t("settings-show-controls-on-launch"),
  },
];

/** The folders Settings names: where projects live, and where exports go. */
export const folders = () =>
  [
    {
      description: t("settings-projects-folder-description"),
      key: "projectDirectory",
      kind: "project",
      reset: t("settings-projects-folder-reset"),
      title: t("settings-projects-folder"),
    },
    {
      description: t("settings-export-folder-description"),
      key: "recordingDirectory",
      kind: "recording",
      reset: t("settings-recording-folder-reset"),
      title: t("settings-recording-folder"),
    },
    {
      description: t("settings-export-folder-description"),
      key: "screenshotDirectory",
      kind: "screenshot",
      reset: t("settings-screenshot-folder-reset"),
      title: t("settings-screenshot-folder"),
    },
  ] as const;
