// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";

import { Region } from "../recording-sources/types";

import type { QuickScreenshotDestination } from "../../bindings/QuickScreenshotDestination";
import type { ScrollingCaptureIslandState } from "../../bindings/ScrollingCaptureIslandState";

export type ScreenshotDestination =
  "editor" | "clipboard" | "both" | "reference";

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

export const getScrollingCaptureIslandState = () =>
  invoke<ScrollingCaptureIslandState | null>("scrolling_capture_island_state");

/** Stops the scrolling capture at its next step, as Escape does. */
export const cancelScrollingCapture = () =>
  invoke<null>("cancel_scrolling_capture");

/** Brings the island out for a Quick Screenshot session, or tells it where
 * the shot now goes. */
export const showQuickScreenshotIsland = (
  destination: QuickScreenshotDestination,
) => invoke<null>("show_quick_screenshot_island", { destination });

export const getQuickScreenshotIslandState = () =>
  invoke<QuickScreenshotDestination | null>("quick_screenshot_island_state");

/** Ends the Quick Screenshot session, as Escape does: the region selector,
 * which owns the session, listens for it. */
export const cancelQuickScreenshot = () =>
  emit("screenshot-region://dismiss-requested");

/** Hides the recording UI and takes `target` once the Settings delay has run
 * down; the shot always opens in the editor. */
export const startDelayedScreenshot = (target: ScreenshotTarget) =>
  invoke<null>("start_delayed_screenshot", { target });
