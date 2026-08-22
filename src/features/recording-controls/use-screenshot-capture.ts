// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { useRecordingInputStore } from "../recording-inputs/store";
import { captureScrollingStill, captureStill } from "../screenshots/api";

import { screenshotTarget } from "./recording-request";
import { ScreenshotAction, ScreenshotState } from "./types";

const SCREENSHOT_FEEDBACK_MS = 2000;
type StillScreenshotAction = Exclude<ScreenshotAction, "scrolling">;

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
    const target = screenshotTarget();
    if (!target) return;

    window.clearTimeout(resetRef.current);
    setScreenshotFeedback({ action: destination, state: "pending" });
    captureStill({
      destination,
      showCursor: useRecordingInputStore.getState().inputs.showCursor,
      target,
    })
      .then(() => {
        setScreenshotFeedback({
          action: destination,
          state: destination === "clipboard" ? "done" : "idle",
        });
      })
      .catch((error: unknown) => {
        console.error("Could not take the screenshot", error);
        setScreenshotFeedback({ action: destination, state: "failed" });
      })
      .finally(() => {
        resetLater(destination);
      });
  };

  const takeScrollingScreenshot = () => {
    const target = screenshotTarget();
    if (!target || target.kind !== "region") return;

    const action: ScreenshotAction = "scrolling";
    window.clearTimeout(resetRef.current);
    setScreenshotFeedback({ action, state: "pending" });
    captureScrollingStill(target)
      .then(() => {
        setScreenshotFeedback({ action, state: "idle" });
      })
      .catch((error: unknown) => {
        console.error("Could not take the scrolling screenshot", error);
        setScreenshotFeedback({ action, state: "failed" });
      })
      .finally(() => {
        resetLater(action);
      });
  };

  return { screenshotFeedback, takeScreenshot, takeScrollingScreenshot };
}
