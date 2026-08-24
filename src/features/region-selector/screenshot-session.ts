// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRecordingInputStore } from "../recording-inputs/store";
import {
  hideRegionSelector,
  listMonitors,
  setRegionSelectorOpacity,
  setRegionSelectorPassthrough,
  setScreenshotRegionSession,
  showRegionSelector,
} from "../recording-sources/api";
import { useRecordingSourceStore } from "../recording-sources/store";
import { MonitorDetails, Region } from "../recording-sources/types";
import { setRulerScreenshotMode } from "../ruler/api";
import { captureStill, ScreenshotDestination } from "../screenshots/api";
import { ShortcutAction } from "../settings/types";

let screenshotDestination: ScreenshotDestination = "export";
type ScreenshotShortcutAction = Extract<
  ShortcutAction,
  "takeScreenshot" | "takeScreenshotToClipboard"
>;
type CleanupStep = () => Promise<unknown>;

async function runCleanupSteps(steps: CleanupStep[]) {
  let firstError: unknown;
  for (const step of steps) {
    try {
      await step();
    } catch (error: unknown) {
      firstError ??= error;
    }
  }
  if (firstError instanceof Error) throw firstError;
  if (firstError !== undefined) {
    throw new Error("Screenshot session cleanup failed");
  }
}

export const isScreenshotShortcut = (
  action: ShortcutAction,
): action is ScreenshotShortcutAction =>
  action === "takeScreenshot" || action === "takeScreenshotToClipboard";

/** Reveals the transparent native overlay after its empty DOM has painted. */
export const revealScreenshotRegion = (monitor: MonitorDetails) => {
  let disposed = false;
  void showRegionSelector(monitor).then(() => {
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        if (!disposed) void setRegionSelectorOpacity(1);
      });
    });
  });
  return () => {
    disposed = true;
  };
};

/**
 * The screenshot shortcut's borrowing of the region overlay, from opening it
 * in region-edit mode to handing the still to its requested destination.
 *
 * The session touches nothing the user chose: the recording mode, source and
 * region are all left exactly as they were found. It starts with no region at
 * all - the overlay opens empty and the user draws the one shot's region.
 */
export const beginScreenshotCapture = async (
  action: ScreenshotShortcutAction,
) => {
  const { selectedMonitor, setScreenshotCapture, setSelectedMonitor } =
    useRecordingSourceStore.getState();

  if (!selectedMonitor) {
    const monitors = await listMonitors();
    const monitor = monitors.find((candidate) => candidate.isPrimary);
    if (!monitor) return;
    setSelectedMonitor(monitor);
  }
  try {
    await setRulerScreenshotMode(true);
    // Rust has to know the overlay is allowed on screen before it is asked for:
    // the recording controls may well be hidden behind it.
    await setScreenshotRegionSession(true);
    // The prior session deliberately leaves the borrowed overlay passthrough
    // while it is hidden. Re-arm it explicitly before React shows it again;
    // the selected monitor may be unchanged, so no dependency effect is
    // guaranteed to do this for us.
    await setRegionSelectorPassthrough(false);
    screenshotDestination =
      action === "takeScreenshotToClipboard" ? "clipboard" : "export";
    setScreenshotCapture(true);
  } catch (error: unknown) {
    try {
      await endScreenshotCapture();
    } catch (cleanupError: unknown) {
      console.error("Could not roll back the screenshot session", cleanupError);
    }
    throw error;
  }
};

export const endScreenshotCapture = async () => {
  const { recordingMode, selectedMonitor, setScreenshotCapture } =
    useRecordingSourceStore.getState();
  const recordingMonitor = recordingMode === "region" ? selectedMonitor : null;

  // Undoing exactly what starting the session did, so the overlay goes back to
  // being the recording region's - or to being off screen.
  await runCleanupSteps([
    () => setScreenshotRegionSession(false),
    () => hideRegionSelector(),
    () => setRegionSelectorOpacity(1),
    () => {
      setScreenshotCapture(false);
      return Promise.resolve();
    },
    // With the session flag already cleared, this asks for the overlay on the
    // recording UI's terms. Successful and cancelled quick screenshots both
    // leave the recording controls and ruler exactly where the user had them.
    () => setRegionSelectorPassthrough(recordingMonitor === null),
    ...(recordingMonitor ? [() => showRegionSelector(recordingMonitor)] : []),
    // Restoring ruler focus comes last so re-showing the normal recording
    // region cannot take Escape back from the overlay opened most recently.
    () => setRulerScreenshotMode(false),
  ]);
};

/** Captures the region to the session's destination, then ends the session. */
export const captureScreenshotRegion = (monitorId: number, region: Region) => {
  const destination = screenshotDestination;
  // The overlay is on top of what is being captured, so it goes invisible for
  // the shot exactly as it does for the magnifier's monitor image.
  const capture = async () => {
    try {
      await setRegionSelectorOpacity(0);
      await captureStill({
        destination,
        showCursor: useRecordingInputStore.getState().inputs.showCursor,
        target: { kind: "region", monitorId, region },
      });
    } catch (error: unknown) {
      console.error("Could not take the screenshot", error);
    } finally {
      try {
        await endScreenshotCapture();
      } catch (error: unknown) {
        console.error("Could not close the screenshot session", error);
      }
    }
  };
  void capture();
};
