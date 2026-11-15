// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";

import { Region } from "../recording-sources/types";

import type { CopiedScreenshot } from "../../bindings/CopiedScreenshot";
import type { QuickScreenshotDestination } from "../../bindings/QuickScreenshotDestination";
import type { ScrollingCaptureIslandState } from "../../bindings/ScrollingCaptureIslandState";

/** `preview` copies the shot and shows it on the island; `reference` keeps
 * it on screen. */
export type ScreenshotDestination = "preview" | "reference";

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

export const captureStill = ({ destination, target }: CaptureStillOptions) =>
  invoke<null>("capture_still", {
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
 * down; the shot is copied and previewed, as every screenshot is. */
export const startDelayedScreenshot = (target: ScreenshotTarget) =>
  invoke<null>("start_delayed_screenshot", { target });

/** The shot just copied that the island shows, if any. */
export const getCopiedScreenshot = () =>
  invoke<CopiedScreenshot | null>("copied_screenshot");

/** The copied shot's time on the island is up: it stays on the clipboard
 * alone. */
export const dismissCopiedScreenshot = (id: number) =>
  invoke<null>("dismiss_copied_screenshot", { id });

/** Opens the copied shot in the editor, which keeps it as a project. */
export const editCopiedScreenshot = (id: number) =>
  invoke<null>("edit_copied_screenshot", { id });

/** Keeps the copied shot in Projects without opening it. */
export const saveCopiedScreenshot = (id: number) =>
  invoke<null>("save_copied_screenshot", { id });
