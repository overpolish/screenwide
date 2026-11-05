// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import {
  hideRegionSelector,
  setRegionSelectorOpacity,
  setRegionSelectorPassthrough,
  setScreenshotRegionSession,
} from "../recording-sources/api";
import { useRecordingSourceStore } from "../recording-sources/store";
import { Region } from "../recording-sources/types";
import {
  captureStill,
  ScreenshotDestination,
  showQuickScreenshotIsland,
} from "../screenshots/api";
import { cancelTextRecognition } from "../text-recognition/api";

import { setRulerScreenshotMode } from "./ruler-screenshot-mode";

import type { ShortcutAction } from "../../bindings/ShortcutAction";

type ScreenshotShortcutAction = Extract<
  ShortcutAction,
  "takeReferenceScreenshot" | "takeScreenshot" | "takeScreenshotToClipboard"
>;
type CleanupStep = () => Promise<unknown>;
let sessionDestination: ScreenshotDestination = "editor";
let sessionAction: ScreenshotShortcutAction | null = null;
let shortcutTransition: Promise<void> = Promise.resolve();

export const screenshotCaptureDestination = () => sessionDestination;

const DESTINATIONS: Record<ScreenshotShortcutAction, ScreenshotDestination> = {
  takeReferenceScreenshot: "reference",
  takeScreenshot: "editor",
  takeScreenshotToClipboard: "clipboard",
};

const selectScreenshotAction = (action: ScreenshotShortcutAction) => {
  sessionDestination = DESTINATIONS[action];
  sessionAction = action;
};

// The island is advisory: a session goes on without it.
const showIsland = () =>
  showQuickScreenshotIsland(
    sessionDestination === "both" ? "editor" : sessionDestination,
  ).catch((error: unknown) => {
    console.error("Could not show the screenshot island", error);
  });

/** The other shortcut, pressed mid-session, changes where the shot goes. */
const switchScreenshotAction = async (action: ScreenshotShortcutAction) => {
  selectScreenshotAction(action);
  await showIsland();
};

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
  action === "takeScreenshot" ||
  action === "takeScreenshotToClipboard" ||
  action === "takeReferenceScreenshot";

/**
 * The screenshot shortcut's borrowing of the region overlay, from opening it
 * in region-edit mode to handing the still to its requested destination.
 *
 * The session touches nothing the user chose: the recording mode, source and
 * region are all left exactly as they were found. It starts with no region at
 * all - the overlay opens empty and the user draws the one shot's region.
 */
const beginScreenshotCapture = async (action: ScreenshotShortcutAction) => {
  const { setScreenshotCapture } = useRecordingSourceStore.getState();
  selectScreenshotAction(action);
  try {
    // Rust has to know the overlay is allowed on screen before it is asked for:
    // the recording controls may well be hidden behind it. Do this before
    // entering ruler screenshot mode as well, because that transition can
    // briefly synchronize the shared OSC while it still holds the persisted
    // recording region.
    await setScreenshotRegionSession(true);
    // Claim the session first so any shortcut arriving during OCR teardown is
    // routed through the same serialized handoff. The capture windows are then
    // closed on this later IPC turn before Region's surfaces are borrowed.
    await cancelTextRecognition();
    await setRulerScreenshotMode(true);
    // The prior session deliberately leaves the borrowed overlay passthrough
    // while it is hidden. Re-arm it explicitly before React shows it again;
    // the selected monitor may be unchanged, so no dependency effect is
    // guaranteed to do this for us.
    await setRegionSelectorPassthrough(false);
    // After the Ruler and Annotate have stepped back, as both put the island
    // away, and before the recording controls are borrowed, so they leave
    // the island's window to it.
    await showIsland();
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
  let restoringRegion = false;

  // Undoing exactly what starting the session did, so the overlay goes back to
  // being the recording region's - or to being off screen.
  try {
    await runCleanupSteps([
      () =>
        setScreenshotRegionSession(false, recordingMonitor !== null).then(
          (restoring) => {
            restoringRegion = restoring;
          },
        ),
      () => (restoringRegion ? Promise.resolve() : hideRegionSelector()),
      () => setRegionSelectorOpacity(1),
      () => {
        setScreenshotCapture(false);
        return Promise.resolve();
      },
      // With the session flag already cleared, this asks for the overlay on the
      // recording UI's terms. Successful and cancelled quick screenshots both
      // leave the recording controls and ruler exactly where the user had them.
      () => setRegionSelectorPassthrough(recordingMonitor === null),
      // Ruler teardown/focus restoration comes last so re-showing the normal
      // recording region cannot take Escape back during cleanup.
      () => setRulerScreenshotMode(false),
    ]);
  } finally {
    sessionAction = null;
  }
};

const enqueueShortcutTransition = (work: () => Promise<void>) => {
  const result = shortcutTransition.then(work, work);
  shortcutTransition = result.catch(() => {});
  return result;
};

export const handleScreenshotShortcut = (action: ScreenshotShortcutAction) =>
  enqueueShortcutTransition(async () => {
    const active = useRecordingSourceStore.getState().isScreenshotCapture;
    const sameAction = active && sessionAction === action;
    if (sameAction) {
      await endScreenshotCapture();
      return;
    }
    if (active) {
      await switchScreenshotAction(action);
      return;
    }
    await beginScreenshotCapture(action);
  });

export const handoffScreenshotShortcut = (action: ShortcutAction) =>
  enqueueShortcutTransition(async () => {
    if (isScreenshotShortcut(action)) {
      const active = useRecordingSourceStore.getState().isScreenshotCapture;
      if (!active) {
        await invoke("resume_shortcut_action", { action });
        return;
      }
      if (sessionAction === action) {
        await endScreenshotCapture();
      } else {
        await switchScreenshotAction(action);
      }
      return;
    }
    await endScreenshotCapture();
    await invoke("resume_shortcut_action", { action });
  });

/** Captures the region to the session's destination, then ends the session. */
export const captureScreenshotRegion = (
  destination: ScreenshotDestination,
  monitorId: number,
  region: Region,
) => {
  // The overlay is on top of what is being captured. macOS hides it for the
  // shot; Windows keeps it visible but temporarily excludes its native window
  // graph from capture, avoiding a flash.
  const capture = async () => {
    try {
      await setRegionSelectorOpacity(0);
      await captureStill({
        destination,
        target: { kind: "desktopRegion", monitorId, region },
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
