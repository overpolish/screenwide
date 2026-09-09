// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import { Region } from "../recording-sources/types";
export type ScreenshotDestination = "editor" | "clipboard" | "both";

export type ScreenshotTarget =
  | { kind: "desktopRegion"; monitorId: number; region: Region }
  | { kind: "region"; monitorId: number; region: Region }
  | { kind: "screen"; monitorId: number }
  | { kind: "window"; windowId: number };

type ScrollingScreenshotTarget = Extract<ScreenshotTarget, { kind: "region" }>;

type CaptureStillOptions = {
  destination: ScreenshotDestination;
  target: ScreenshotTarget;
};

/** Resolves to the saved file's path, or null when it went to the clipboard. */
export const captureStill = ({ destination, target }: CaptureStillOptions) =>
  invoke<string | null>("capture_still", {
    destination,
    // The pointer is part of what a screenshot is showing, so it is always
    // captured.
    showCursor: true,
    target,
  });

export const captureScrollingStill = (target: ScrollingScreenshotTarget) =>
  invoke<null>("capture_scrolling_still", { target });
