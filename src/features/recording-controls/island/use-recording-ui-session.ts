// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { useTauriEvent } from "../../../lib/use-tauri-event";
import {
  cancelWindowPick as cancelPick,
  hideRecordingUi,
  hideRegionSelector,
  recordingUiVisible,
  showRegionSelector,
  startWindowPick,
} from "../../recording-sources/api";
import { useRecordingSourceStore } from "../../recording-sources/store";
import {
  type RecordingMode,
  type WindowDetails,
  type WindowPickHover,
} from "../../recording-sources/types";
import { useRecordingStore } from "../store";
import { RecordingError } from "../types";

/** Picking a window borrows the region overlay, which the mode leaves alone
 * until the pick is over. */
let isPicking = false;

/** Shows the region overlay as the mode wants. */
const synchronizeRecordingUi = async (
  mode = useRecordingSourceStore.getState().recordingMode,
  monitor = useRecordingSourceStore.getState().selectedMonitor,
) => {
  // A rehydrate mid-recording must not re-show the chrome that starting the
  // recording deliberately hid.
  if (useRecordingStore.getState().snapshot.status !== "idle") return;
  // Nor may one take the region overlay away from a screenshot session, which
  // borrows it regardless of the recording mode.
  if (useRecordingSourceStore.getState().isScreenshotCapture) return;
  if (isPicking) return;

  if (mode === "region" && monitor) {
    await showRegionSelector(monitor, true);
  } else {
    await hideRegionSelector();
  }
};

/** The last mode change's re-sync, awaited before a pick borrows the
 * overlay it may be hiding. */
let pendingModeChange: Promise<void> = Promise.resolve();

/** Records `mode`, ending any pick, and puts the region overlay in step. */
export const changeRecordingMode = (mode: RecordingMode) => {
  const { selectedMonitor, setRecordingMode } =
    useRecordingSourceStore.getState();
  setRecordingMode(mode);
  const pick = isPicking ? cancelPick() : Promise.resolve(null);
  pendingModeChange = pick.then(() =>
    synchronizeRecordingUi(mode, selectedMonitor),
  );
};

/** Starts picking the window to record, once a mode change on the same press
 * has settled. */
export const chooseWindow = () => {
  void pendingModeChange
    .then(() =>
      startWindowPick(
        useRecordingSourceStore.getState().selectedMonitor?.id ?? null,
      ),
    )
    .catch((error: unknown) => {
      console.error("Could not start picking a window", error);
    });
};

/** Ends picking without a window. */
export const cancelWindowPick = () => {
  void cancelPick();
};

/**
 * The recording controls' part in the app: whether they are open, which
 * Rust says as it shows and hides them; whether a capture overlay has the
 * screen; the window under the pointer while one is picked; and the
 * dismissals that come to them, from Escape and from the ruler and text
 * recognition they start.
 */
export function useRecordingUiSession() {
  const [isRecordingUiVisible, setIsRecordingUiVisible] = useState(false);
  const [isCaptureOverlayActive, setIsCaptureOverlayActive] = useState(false);
  // While picking, the window under the pointer, or null over none.
  const [windowPick, setWindowPick] = useState<{
    hovered: WindowPickHover | null;
  } | null>(null);
  const recordingMode = useRecordingSourceStore((state) => state.recordingMode);
  const selectedMonitor = useRecordingSourceStore(
    (state) => state.selectedMonitor,
  );
  const setScreenshotCapture = useRecordingSourceStore(
    (state) => state.setScreenshotCapture,
  );
  // What the window is told outranks what it asked before being told.
  const isToldRef = useRef(false);

  useEffect(() => {
    // Any screenshot session that outlived a previous run belongs to a window
    // that is gone.
    setScreenshotCapture(false);
    void recordingUiVisible()
      .then((visible) => {
        if (!isToldRef.current) setIsRecordingUiVisible(visible);
      })
      .catch(() => undefined);
  }, [setScreenshotCapture]);

  useEffect(() => {
    // Returning to idle does not mean the controls should return: a completed
    // capture hands ownership to the editor window. Showing them emits
    // `recording-ui://shown`, which synchronizes them at that point.
    void synchronizeRecordingUi();
  }, [recordingMode, selectedMonitor]);

  useTauriEvent("recording-ui://shown", () => {
    isToldRef.current = true;
    setIsRecordingUiVisible(true);
    void synchronizeRecordingUi();
  });
  useTauriEvent("recording-ui://hidden", () => {
    isToldRef.current = true;
    setIsRecordingUiVisible(false);
  });
  useTauriEvent("capture-overlay://started", () => {
    setIsCaptureOverlayActive(true);
  });
  useTauriEvent("capture-overlay://ended", () => {
    setIsCaptureOverlayActive(false);
  });
  useTauriEvent("recording-ui://dismiss-requested", () => {
    void hideRecordingUi();
  });
  useTauriEvent("window-pick://hovered", (payload) => {
    isPicking = true;
    setWindowPick({ hovered: payload as WindowPickHover | null });
  });
  useTauriEvent("window-pick://picked", (payload) => {
    useRecordingSourceStore
      .getState()
      .setSelectedWindow(payload as WindowDetails);
  });
  useTauriEvent("window-pick://ended", () => {
    isPicking = false;
    setWindowPick(null);
  });
  // Rust puts failures the user has to know about in an alert; the island
  // follows the state Rust reverted to and logs the detail.
  useTauriEvent("recording://error", (payload) => {
    const error = payload as RecordingError;
    console.error(`Recording ${error.phase} failed: ${error.message}`);
  });

  return { isCaptureOverlayActive, isRecordingUiVisible, windowPick };
}
