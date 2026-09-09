// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

export type MonitorDetails = {
  id: number;
  isBuiltin: boolean;
  isPrimary: boolean;
  layoutPosition: { x: number; y: number };
  layoutSize: { height: number; width: number };
  name: string;
  physicalPosition: { x: number; y: number };
  physicalSize: { height: number; width: number };
  position: { x: number; y: number };
  scaleFactor: number;
  size: { height: number; width: number };
};

/** Where the cached still of one display sits on disk. */
export type MonitorThumbnail = {
  id: number;
  path: string;
};

export type Region = {
  position: { x: number; y: number };
  size: { height: number; width: number };
};

export type RecordingMode = "screen" | "region" | "window" | "camera" | "audio";

export type SelectorPlacement = "above" | "below";

export type SelectorState = {
  expanded: boolean;
  focusContents: boolean;
  /** The popover is open but still offscreen, waiting for the webview to paint
   * the mode it was opened for and ask for the reveal. */
  placement: SelectorPlacement;
  revision: number;
  /** Which selector the popover was opened for. */
  windowSelector: boolean;
};

export type WindowDetails = {
  appIconPath: string | null;
  appName: string;
  id: number;
  pid: number;
  position: { x: number; y: number };
  size: { height: number; width: number };
  thumbnailPath: string | null;
  title: string;
};
