// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { useFitWindowWidth } from "../../../lib/use-fit-window-height";
import { focusEditorWindow } from "../../editor/api";
import {
  selectHasPendingRecording,
  selectHasPendingScreenshot,
  useEditorStore,
} from "../../editor/store";
import {
  openPermissionsWindow,
  openPermissionSettings,
  requestPermission,
} from "../../permissions/api";
import {
  selectCanRecordCamera,
  selectCanRecordMicrophone,
  selectCanRecordScreen,
  selectCanScreenshot,
  usePermissionStore,
} from "../../permissions/store";
import { PermissionKind, PermissionStatus } from "../../permissions/types";
import { useRecordingInputStore } from "../../recording-inputs/store";
import { cameraRequestFps } from "../../recording-inputs/types";
import {
  collapseRecordingSourceSelector,
  expandRecordingSourceSelector,
  finishRecordingBarDrag,
  hideRecordingUi,
  hideRegionSelector,
  listMonitors,
  recordingUiVisible,
  setRecordingSourceSelectorVisible,
  showRegionSelector,
  selectedWindowAvailable,
} from "../../recording-sources/api";
import { findCurrentMonitor } from "../../recording-sources/monitor-selection";
import {
  MONITOR_THUMBNAIL_INTERVAL_MS,
  refreshMonitorThumbnails,
} from "../../recording-sources/monitor-thumbnails";
import { useRecordingSourceStore } from "../../recording-sources/store";
import { type RecordingMode } from "../../recording-sources/types";
import { cancelRuler } from "../../region-selector/ruler-screenshot-mode";
import { cancelTextRecognition } from "../../text-recognition/api";
import { startRecording } from "../api";
import { startRecordingOptions } from "../recording-request";
import { selectStatus, useRecordingStore } from "../store";
import { RecordingError } from "../types";
import { useRecordingBarShortcuts } from "../use-recording-bar-shortcuts";
import { useRecordingInputAvailability } from "../use-recording-input-availability";
import { useScreenshotCapture } from "../use-screenshot-capture";

import { RecordingBar } from "./recording-bar";

const RECORDING_ERROR_EVENT = "recording://error";
const RECORDING_DISMISS_REQUESTED_EVENT = "recording-ui://dismiss-requested";
const RULER_DISMISS_REQUESTED_EVENT = "ruler://dismiss-requested";
const TEXT_RECOGNITION_DISMISS_REQUESTED_EVENT =
  "text-recognition://dismiss-requested";
/** A recording started without selected inputs whose devices had vanished. */
const RECORDING_INPUTS_SKIPPED_EVENT = "recording://inputs-skipped";
const SOURCE_AVAILABILITY_INTERVAL_MS = 1_500;

const dismissRecordingUi = () => {
  void hideRecordingUi();
};

const SKIPPED_INPUT_LABELS: Record<string, string> = {
  camera: "camera",
  microphone: "microphone",
  systemAudio: "system audio",
};
const validateSelectedWindow = async (isActive: () => boolean) => {
  const selected = useRecordingSourceStore.getState().selectedWindow;
  if (!selected) return;

  try {
    const available = await selectedWindowAvailable(selected);
    if (!isActive()) return;
    const { recordingMode, selectedWindow, setSelectedWindow } =
      useRecordingSourceStore.getState();
    if (
      !available &&
      recordingMode === "window" &&
      selectedWindow?.id === selected.id &&
      selectedWindow.pid === selected.pid
    ) {
      setSelectedWindow(null);
    }
  } catch (error) {
    console.error("Could not validate the selected window", error);
  }
};

const validateSelectedMonitor = async (isActive: () => boolean) => {
  const selected = useRecordingSourceStore.getState().selectedMonitor;

  try {
    const monitors = await listMonitors();
    if (!isActive()) return;
    const { recordingMode, selectedMonitor, setSelectedMonitor } =
      useRecordingSourceStore.getState();
    if (
      !["region", "screen"].includes(recordingMode) ||
      selectedMonitor?.id !== selected?.id
    ) {
      return;
    }
    const current = findCurrentMonitor(monitors, selectedMonitor);
    if (
      current &&
      JSON.stringify(current) !== JSON.stringify(selectedMonitor)
    ) {
      setSelectedMonitor(current);
    }
  } catch (error) {
    console.error("Could not validate the selected monitor", error);
  }
};

const validateSelectedSource = async (
  mode: RecordingMode,
  isActive: () => boolean,
) => {
  if (mode === "window") {
    await validateSelectedWindow(isActive);
  } else if (["region", "screen"].includes(mode)) {
    await validateSelectedMonitor(isActive);
  }
};

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

  // Displays are chosen from a native menu now, so only the window list still
  // needs the selector panel armed.
  await setRecordingSourceSelectorVisible(!["audio", "camera"].includes(mode));
  if (mode === "region" && monitor) {
    await showRegionSelector(monitor, true);
  } else {
    await hideRegionSelector();
  }
};

/** The last mode change's collapse and re-sync, awaited before an expand. */
let pendingModeChange: Promise<void> = Promise.resolve();

const grantPermission = (
  permission: PermissionKind,
  status: PermissionStatus,
) => {
  const action = status.canRequest
    ? requestPermission(permission)
    : openPermissionSettings(permission);
  void action;
};

export function RecordingBarWindow() {
  const barRef = useRef<HTMLElement>(null);
  // The bar is as wide as its controls: a label that changes with state
  // moves the trailing edge rather than leaving slack in the bar.
  useFitWindowWidth(barRef);
  const hasPendingRecording = useEditorStore(selectHasPendingRecording);
  const hasPendingScreenshot = useEditorStore(selectHasPendingScreenshot);
  const canRecordCamera = usePermissionStore(selectCanRecordCamera);
  const canRecordMicrophone = usePermissionStore(selectCanRecordMicrophone);
  const canRecordScreen = usePermissionStore(selectCanRecordScreen);
  const canScreenshot = usePermissionStore(selectCanScreenshot);
  const hydrated = usePermissionStore((state) => state.hydrated);
  const permissions = usePermissionStore((state) => state.permissions);
  const status = useRecordingStore(selectStatus);
  const { screenshotFeedback, takeScreenshot, takeScrollingScreenshot } =
    useScreenshotCapture();
  const [isCaptureOverlayActive, setIsCaptureOverlayActive] = useState(false);
  const [isRecordingUiVisible, setIsRecordingUiVisible] = useState(false);
  const {
    isScreenshotCapture,
    monitorThumbnails,
    recordingMode,
    selectedMonitor,
    selectedWindow,
    setRecordingMode,
    setScreenshotCapture,
  } = useRecordingSourceStore((state) => state);
  const isScreenMode = recordingMode === "screen";
  const {
    cameraPalById,
    fps,
    inputs,
    selectedCamera,
    selectedMicrophone,
    selectedSystemAudio,
    setInput,
  } = useRecordingInputStore((state) => state);
  useRecordingBarShortcuts();
  const inputAvailability = useRecordingInputAvailability({
    active:
      isRecordingUiVisible &&
      !isCaptureOverlayActive &&
      !isScreenshotCapture &&
      screenshotFeedback.state !== "pending" &&
      status === "idle",
    cameraEnabled: inputs.camera || recordingMode === "camera",
    cameraFps: cameraRequestFps(
      fps,
      selectedCamera ? (cameraPalById[selectedCamera.id] ?? false) : false,
    ),
    cameraPermissionGranted: hydrated && permissions.camera.granted,
    microphoneEnabled: inputs.microphone,
    microphonePermissionGranted: hydrated && permissions.microphone.granted,
    screenRecordingPermissionGranted:
      hydrated && permissions.screenRecording.granted,
    selectedCamera,
    selectedMicrophone,
    selectedSystemAudio,
    systemAudioEnabled: inputs.systemAudio,
  });

  useEffect(() => {
    // Any screenshot session that outlived a previous run belongs to a window
    // that is gone.
    setScreenshotCapture(false);
  }, [setScreenshotCapture]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    void listen(RULER_DISMISS_REQUESTED_EVENT, () => {
      void cancelRuler();
    }).then((listener) => {
      if (disposed) listener();
      else unlisten = listener;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    void listen(TEXT_RECOGNITION_DISMISS_REQUESTED_EVENT, () => {
      void cancelTextRecognition();
    }).then((listener) => {
      if (disposed) listener();
      else unlisten = listener;
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    // Returning to idle does not mean the controls should return: a completed
    // capture hands ownership to the editor window. Explicitly showing the
    // recording UI emits the event below and synchronizes it at that point.
    void synchronizeRecordingUi();
  }, [recordingMode, selectedMonitor]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let unlistenHidden: UnlistenFn | undefined;
    let unlistenCaptureEnded: UnlistenFn | undefined;
    let unlistenCaptureStarted: UnlistenFn | undefined;
    let disposed = false;
    let receivedVisibilityEvent = false;

    void Promise.all([
      listen("recording-ui://shown", () => {
        receivedVisibilityEvent = true;
        setIsRecordingUiVisible(true);
        void synchronizeRecordingUi();
      }),
      listen("recording-ui://hidden", () => {
        receivedVisibilityEvent = true;
        setIsRecordingUiVisible(false);
      }),
      listen("capture-overlay://started", () => {
        setIsCaptureOverlayActive(true);
      }),
      listen("capture-overlay://ended", () => {
        setIsCaptureOverlayActive(false);
      }),
    ]).then(([shown, hidden, captureStarted, captureEnded]) => {
      if (disposed) {
        shown();
        hidden();
        captureStarted();
        captureEnded();
      } else {
        unlisten = shown;
        unlistenHidden = hidden;
        unlistenCaptureStarted = captureStarted;
        unlistenCaptureEnded = captureEnded;
      }
      void recordingUiVisible()
        .then((visible) => {
          if (!disposed && !receivedVisibilityEvent) {
            setIsRecordingUiVisible(visible);
          }
        })
        .catch(() => {});
    });

    return () => {
      disposed = true;
      unlisten?.();
      unlistenHidden?.();
      unlistenCaptureEnded?.();
      unlistenCaptureStarted?.();
    };
  }, []);

  // The Screen segment shows a still of the chosen display whatever mode is
  // current, so one is captured when the bar appears. While a display is what
  // is being set up, the still is retaken on the chooser's cadence so the two
  // pictures stay live together.
  useEffect(() => {
    if (!isRecordingUiVisible) return;
    void refreshMonitorThumbnails();
    if (!isScreenMode || status !== "idle") return;
    const interval = window.setInterval(() => {
      void refreshMonitorThumbnails();
    }, MONITOR_THUMBNAIL_INTERVAL_MS);
    return () => {
      window.clearInterval(interval);
    };
  }, [isRecordingUiVisible, isScreenMode, status]);

  useEffect(() => {
    if (!isRecordingUiVisible || status !== "idle") return;

    let disposed = false;
    let validating = false;
    const validate = async () => {
      if (validating) return;
      validating = true;
      try {
        await validateSelectedSource(recordingMode, () => !disposed);
      } finally {
        validating = false;
      }
    };

    void validate();
    const interval = window.setInterval(() => {
      if (!disposed) void validate();
    }, SOURCE_AVAILABILITY_INTERVAL_MS);

    return () => {
      disposed = true;
      window.clearInterval(interval);
    };
  }, [isRecordingUiVisible, recordingMode, status]);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    void listen(RECORDING_DISMISS_REQUESTED_EVENT, dismissRecordingUi).then(
      (listener) => {
        if (disposed) listener();
        else unlisten = listener;
      },
    );

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    let disposed = false;

    // There is no toast surface, so a failure is logged and the UI simply
    // follows the state Rust reverted to.
    void listen<RecordingError>(RECORDING_ERROR_EVENT, ({ payload }) => {
      console.error(`Recording ${payload.phase} failed: ${payload.message}`);
    }).then((listener) => {
      if (disposed) {
        listener();
      } else {
        unlisten = listener;
      }
    });

    let unlistenSkipped: (() => void) | undefined;
    // The dock already reflects the sanitized inputs (their meters are gone);
    // this names what was dropped and why the recording still started.
    void listen<{ inputs: string[] }>(
      RECORDING_INPUTS_SKIPPED_EVENT,
      ({ payload }) => {
        const labels = payload.inputs
          .map((input) => SKIPPED_INPUT_LABELS[input] ?? input)
          .join(", ");
        console.warn(
          `Recording started without ${labels}: the selected device is no longer available.`,
        );
      },
    ).then((listener) => {
      if (disposed) {
        listener();
      } else {
        unlistenSkipped = listener;
      }
    });

    return () => {
      disposed = true;
      unlisten?.();
      unlistenSkipped?.();
    };
  }, []);

  return (
    <RecordingBar
      hasCameraWarning={inputAvailability.cameraMissing}
      hasMicrophoneWarning={inputAvailability.microphoneMissing}
      hasSelectedMonitor={selectedMonitor !== null}
      hasSelectedWindow={selectedWindow !== null}
      hasSystemAudioWarning={inputAvailability.systemAudioMissing}
      initialMode={recordingMode}
      inputs={inputs}
      isCameraLocked={!hydrated || !canRecordCamera}
      isLocked={hydrated && !canRecordScreen}
      isMicrophoneLocked={!hydrated || !canRecordMicrophone}
      // The window is only hidden, never unmounted: the previews stop when the
      // bar goes away rather than streaming on behind it.
      isPreviewActive={isRecordingUiVisible && status === "idle"}
      isScreenshotLocked={hydrated && !canScreenshot}
      mode={recordingMode}
      monitorThumbnails={monitorThumbnails}
      onCameraLockedPress={() => {
        grantPermission("camera", permissions.camera);
      }}
      onCancel={() => {
        dismissRecordingUi();
      }}
      onChooseMonitor={(_anchor, fromKeyboard) => {
        void pendingModeChange.then(() =>
          expandRecordingSourceSelector(false, fromKeyboard),
        );
      }}
      onChooseWindow={(_anchor, fromKeyboard) => {
        void pendingModeChange.then(() =>
          expandRecordingSourceSelector(true, fromKeyboard),
        );
      }}
      onFocusPendingEditor={() => {
        // Only a pending recording routes here now; a screenshot workspace
        // never blocks a capture.
        focusEditorWindow("recording").catch((error: unknown) => {
          console.error("Could not focus the editor window", error);
        });
      }}
      onInputChange={setInput}
      onInteract={() => {
        void collapseRecordingSourceSelector();
      }}
      onMicrophoneLockedPress={() => {
        grantPermission("microphone", permissions.microphone);
      }}
      onModeChange={(mode) => {
        setRecordingMode(mode);
        // Remembered so a chevron press on the same gesture can expand the
        // selector after this collapse and re-sync, not racing them.
        pendingModeChange = collapseRecordingSourceSelector().then(() =>
          synchronizeRecordingUi(mode, selectedMonitor),
        );
        void pendingModeChange;
      }}
      onPointerUp={() => {
        void finishRecordingBarDrag();
      }}
      onRecord={() => {
        startRecording(startRecordingOptions()).catch((error: unknown) => {
          console.error("Could not start the recording", error);
        });
      }}
      onRequiredPermissionsPress={() => {
        void openPermissionsWindow();
      }}
      onScreenshot={() => {
        takeScreenshot("editor");
      }}
      onScreenshotToClipboard={() => {
        takeScreenshot("clipboard");
      }}
      onScrollingScreenshot={() => {
        if (!permissions.accessibility.granted) {
          grantPermission("accessibility", permissions.accessibility);
          return;
        }
        takeScrollingScreenshot();
      }}
      pendingEditors={{
        recording: hasPendingRecording,
        screenshot: hasPendingScreenshot,
      }}
      ref={barRef}
      screenshotAction={screenshotFeedback.action}
      screenshotState={screenshotFeedback.state}
      selectedMonitor={selectedMonitor}
      selectedWindow={selectedWindow}
      status={status}
    />
  );
}
