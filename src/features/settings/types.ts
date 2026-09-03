// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type ShortcutAction =
  | "toggleRecordingBar"
  | "startStopRecording"
  | "pauseResumeRecording"
  | "takeScreenshot"
  | "takeScreenshotToClipboard"
  | "recognizeText"
  | "rulerOverlay";

type ShortcutBinding = {
  action: ShortcutAction;
  shortcut: string | null;
};

export type ShortcutSettings = {
  bindings: ShortcutBinding[];
};

export type GlideControl = string;

export type GlideSettings = {
  cursorFollows: boolean;
  doubleTapCenter: boolean;
  enabled: boolean;
  haptics: boolean;
  mouseModifier: GlideControl;
  thirdsModifier: GlideControl;
  windowGap: number;
};

export type GeneralSettings = {
  launchAtLogin: boolean;
  openLocationAfterExport: boolean;
  recordScreenwideWindows: boolean;
  recordingCountdownSeconds: 0 | 3 | 5;
  recordingDirectory: string | null;
  screenshotDirectory: string | null;
  showRecordingBarOnLaunch: boolean;
  showRecordingConfidenceChecks: boolean;
};
