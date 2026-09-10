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

export type RulerAction =
  | "toggleCrosshair"
  | "copyColour"
  | "deleteMeasurement"
  | "copyMeasurement"
  | "undo"
  | "redo"
  | "stampHorizontal"
  | "stampVertical"
  | "guideVertical"
  | "guideHorizontal"
  | "cycleTolerance"
  | "measureRadius"
  | "toggleCenterlines";

export type RulerSettings = {
  bindings: Record<RulerAction, string | null>;
  enabled: boolean;
};

export type OcrAction = "selectAll" | "copyText";

export type OcrSettings = {
  bindings: Record<OcrAction, string | null>;
  enabled: boolean;
};

export type ShortcutDefaults = {
  glide: GlideSettings;
  ocr: OcrSettings;
  ruler: RulerSettings;
  shortcuts: ShortcutSettings;
};

export type GlideControl = string;

export type GlideSettings = {
  cursorFollows: boolean;
  doubleTapCenter: boolean;
  enabled: boolean;
  haptics: boolean;
  monitorsModifier: GlideControl;
  mouseModifier: GlideControl;
  spacesModifier: GlideControl;
  thirdsModifier: GlideControl;
  windowGap: number;
};

export type GeneralSettings = {
  accent: "screenwide" | "system";
  launchAtLogin: boolean;
  openLocationAfterExport: boolean;
  recordScreenwideWindows: boolean;
  recordingCountdownSeconds: 0 | 3 | 5;
  recordingDirectory: string | null;
  screenshotDirectory: string | null;
  showRecordingBarOnLaunch: boolean;
  showRecordingConfidenceChecks: boolean;
};
