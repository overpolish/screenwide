// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { platformArg, t } from "../../../i18n/i18n";

import type { GeneralSettings } from "../../../bindings/GeneralSettings";

// The rows General and Recording draw from lists, shared with Settings
// search.

/** The preferences a Switch row can drive. */
type ToggleKey = {
  [K in keyof GeneralSettings]: GeneralSettings[K] extends boolean ? K : never;
}[keyof GeneralSettings];

export type ToggleItem = {
  key: ToggleKey;
  title: string;
  description?: string;
};

export const captureToggle = (): ToggleItem => ({
  description: t("settings-include-app-description"),
  key: "recordScreenwideWindows",
  title: t("settings-include-app"),
});

export const desktopIconsToggle = (): ToggleItem => ({
  description: t("settings-hide-desktop-icons-description", {
    platform: platformArg(),
  }),
  key: "hideDesktopIcons",
  title: t("settings-hide-desktop-icons"),
});

/** Apps kept out of captures: left out of each capture on macOS, hidden
 * from every capture while recording on Windows. */
export const hiddenAppsRow = () => ({
  description: t("settings-hidden-apps-description", {
    platform: platformArg(),
  }),
  title: t("settings-hidden-apps"),
});

export const launchToggle = (): ToggleItem => ({
  key: "launchAtLogin",
  title: t("settings-launch-at-login"),
});

export const controlsOnLaunchToggle = (): ToggleItem => ({
  key: "showRecordingControlsOnLaunch",
  title: t("settings-show-controls-on-launch"),
});

export const confidenceChecksToggle = (): ToggleItem => ({
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

export const antiFlickerToggle = (): ToggleItem => ({
  description: t("settings-anti-flicker-description"),
  key: "cameraAntiFlicker",
  title: t("settings-anti-flicker"),
});

/** The menu bar on macOS, the taskbar on Windows. */
export const menuBarToggle = (): ToggleItem => ({
  description: t("settings-hide-menu-bar-description", {
    platform: platformArg(),
  }),
  key: "hideMenuBar",
  title: t("settings-hide-menu-bar", { platform: platformArg() }),
});

/** macOS only: Windows has no Dock. */
export const dockToggle = (): ToggleItem => ({
  description: t("settings-hide-dock-description"),
  key: "hideDock",
  title: t("settings-hide-dock"),
});

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
