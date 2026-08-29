// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { useRecordingInputStore } from "../recording-inputs/store";
import {
  hideRecordingUi,
  setRegionSelectorOscFrameVisible,
} from "../recording-sources/api";
import { captureScrollingStill, captureStill } from "../screenshots/api";

import { screenshotTarget } from "./recording-request";
import { ScreenshotAction, ScreenshotState } from "./types";

const SCREENSHOT_FEEDBACK_MS = 2000;
type StillScreenshotAction = Exclude<ScreenshotAction, "scrolling">;

async function setRegionCaptureFrame(visible: boolean) {
  try {
    await setRegionSelectorOscFrameVisible(visible);
  } catch (error: unknown) {
    console.error("Could not update the Region capture frame", error);
  }
}

async function dismissAfterScreenshot() {
  try {
    await hideRecordingUi();
  } catch (error: unknown) {
    console.error("Could not dismiss the recording UI after capture", error);
  }
}

export function useScreenshotCapture() {
  const [screenshotFeedback, setScreenshotFeedback] = useState<{
    action: ScreenshotAction;
    state: ScreenshotState;
  }>({ action: "export", state: "idle" });
  const resetRef = useRef<number | undefined>(undefined);

  useEffect(
    () => () => {
      window.clearTimeout(resetRef.current);
    },
    [],
  );

  const resetLater = (action: ScreenshotAction) => {
    resetRef.current = window.setTimeout(() => {
      setScreenshotFeedback({ action, state: "idle" });
    }, SCREENSHOT_FEEDBACK_MS);
  };

  const takeScreenshot = (destination: StillScreenshotAction) => {
    const target = screenshotTarget(true);
    if (!target) return;

    window.clearTimeout(resetRef.current);
    setScreenshotFeedback({ action: destination, state: "pending" });
    void (async () => {
      if (target.kind === "desktopRegion") await setRegionCaptureFrame(false);
      try {
        await captureStill({
          destination,
          showCursor: useRecordingInputStore.getState().inputs.showCursor,
          target,
        });
        setScreenshotFeedback({
          action: destination,
          state: destination === "clipboard" ? "done" : "idle",
        });
        await dismissAfterScreenshot();
      } catch (error: unknown) {
        console.error("Could not take the screenshot", error);
        setScreenshotFeedback({ action: destination, state: "failed" });
      } finally {
        // Restore the native state after dismissal so reopening the recording
        // UI later presents its frame immediately. This does not show a hidden
        // Region window.
        if (target.kind === "desktopRegion") await setRegionCaptureFrame(true);
        resetLater(destination);
      }
    })();
  };

  const takeScrollingScreenshot = () => {
    const target = screenshotTarget();
    if (!target || target.kind !== "region") return;

    const action: ScreenshotAction = "scrolling";
    window.clearTimeout(resetRef.current);
    setScreenshotFeedback({ action, state: "pending" });
    void (async () => {
      await setRegionCaptureFrame(false);
      try {
        await captureScrollingStill(target);
        setScreenshotFeedback({ action, state: "idle" });
        await dismissAfterScreenshot();
      } catch (error: unknown) {
        console.error("Could not take the scrolling screenshot", error);
        setScreenshotFeedback({ action, state: "failed" });
      } finally {
        await setRegionCaptureFrame(true);
        resetLater(action);
      }
    })();
  };

  return { screenshotFeedback, takeScreenshot, takeScrollingScreenshot };
}
