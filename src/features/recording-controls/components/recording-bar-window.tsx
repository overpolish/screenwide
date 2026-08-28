// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

import { focusExportWindow } from "../../exports/api";
import {
  selectHasPendingRecording,
  selectHasPendingScreenshot,
  useExportStore,
} from "../../exports/store";
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
import {
  hideRecordingOptions,
  toggleRecordingOptions,
} from "../../recording-inputs/api";
import { useRecordingInputStore } from "../../recording-inputs/store";
import { cameraRequestFps } from "../../recording-inputs/types";
import {
  collapseRecordingSourceSelector,
  expandRecordingSourceSelector,
  finishRecordingBarDrag,
  getRecordingSourceSelectorState,
  hideRecordingUi,
  hideRegionSelector,
  listMonitors,
  recordingUiVisible,
  setRecordingSourceSelectorVisible,
  showRegionSelector,
  selectedWindowAvailable,
  toggleRecordingUi,
} from "../../recording-sources/api";
import { findCurrentMonitor } from "../../recording-sources/monitor-selection";
import { RecordingSourceTrigger } from "../../recording-sources/recording-source-trigger";
import { RegionSourceControls } from "../../recording-sources/region-source-controls";
import { useRecordingSourceStore } from "../../recording-sources/store";
import {
  type RecordingMode,
  SelectorState,
} from "../../recording-sources/types";
import { WindowSourceControls } from "../../recording-sources/window-source-controls";
import { ShortcutAction } from "../../settings/types";
import { startRecording } from "../api";
import { startRecordingOptions } from "../recording-request";
import { selectStatus, useRecordingStore } from "../store";
import { RecordingError } from "../types";
import { useRecordingInputAvailability } from "../use-recording-input-availability";
import { useScreenshotCapture } from "../use-screenshot-capture";

import { RecordingBar } from "./recording-bar";

const RECORDING_ERROR_EVENT = "recording://error";
const RECORDING_DISMISS_REQUESTED_EVENT = "recording-ui://dismiss-requested";
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
const SHORTCUT_ACTION_EVENT = "global-shortcut://action";
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

  const hasSourceSelector = !["audio", "camera"].includes(mode);

  await setRecordingSourceSelectorVisible(hasSourceSelector);
  if (mode === "region" && monitor) {
    await showRegionSelector(monitor);
  } else {
    await hideRegionSelector();
  }
};

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
  const hasPendingRecording = useExportStore(selectHasPendingRecording);
  const hasPendingScreenshot = useExportStore(selectHasPendingScreenshot);
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
  const [isSourceSelectorExpanded, setIsSourceSelectorExpanded] =
    useState(false);
  const isSourceSelectorExpandedRef = useRef(false);
  const {
    isScreenshotCapture,
    recordingMode,
    selectedMonitor,
    selectedWindow,
    setRecordingMode,
    setScreenshotCapture,
  } = useRecordingSourceStore((state) => state);
  const {
    cameraPalById,
    fps,
    inputs,
    selectedCamera,
    selectedMicrophone,
    selectedSystemAudio,
    setFps,
    setInput,
  } = useRecordingInputStore((state) => state);
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
    let disposed = false;
    let unlisten: UnlistenFn | undefined;

    const initialize = async () => {
      unlisten = await listen<SelectorState>(
        "recording-source-selector://state",
        ({ payload }) => {
          isSourceSelectorExpandedRef.current = payload.expanded;
          setIsSourceSelectorExpanded(payload.expanded);
        },
      );
      const state = await getRecordingSourceSelectorState();
      if (!disposed) {
        isSourceSelectorExpandedRef.current = state.expanded;
        setIsSourceSelectorExpanded(state.expanded);
      }
      if (disposed) unlisten();
    };

    void initialize();

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    // Returning to idle does not mean the controls should return: a completed
    // capture hands ownership to the export window. Explicitly showing the
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
    // Emitting to a window does not scope delivery: `listen` registers for any
    // target, so every window sees every shortcut action and each listener has
    // to match the one it owns exactly.
    void listen<ShortcutAction>(SHORTCUT_ACTION_EVENT, ({ payload }) => {
      if (payload === "toggleRecordingBar") {
        void toggleRecordingUi();
        return;
      }
      if (payload !== "startStopRecording") return;
      startRecording(startRecordingOptions()).catch((error: unknown) => {
        console.error("Could not start the recording", error);
      });
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
      fps={fps}
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
      isScreenshotLocked={hydrated && !canScreenshot}
      mode={recordingMode}
      onCameraLockedPress={() => {
        grantPermission("camera", permissions.camera);
      }}
      onCancel={() => {
        dismissRecordingUi();
      }}
      onFocusPendingExport={() => {
        // Only a pending recording routes here now; a screenshot workspace
        // never blocks a capture.
        focusExportWindow("recording").catch((error: unknown) => {
          console.error("Could not focus the export window", error);
        });
      }}
      onFpsChange={setFps}
      onInputChange={setInput}
      onInteract={() => {
        void collapseRecordingSourceSelector();
        void hideRecordingOptions();
      }}
      onMicrophoneLockedPress={() => {
        grantPermission("microphone", permissions.microphone);
      }}
      onModeChange={(mode) => {
        setRecordingMode(mode);
        void Promise.all([
          collapseRecordingSourceSelector(),
          hideRecordingOptions(),
        ]).then(() => synchronizeRecordingUi(mode, selectedMonitor));
      }}
      onOptions={(anchor, focusContents) => {
        void collapseRecordingSourceSelector().then(() =>
          toggleRecordingOptions(anchor, focusContents),
        );
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
        takeScreenshot("export");
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
      pendingExports={{
        recording: hasPendingRecording,
        screenshot: hasPendingScreenshot,
      }}
      screenshotAction={screenshotFeedback.action}
      screenshotState={screenshotFeedback.state}
      sourceSelector={
        <>
          <RecordingSourceTrigger
            isExpanded={isSourceSelectorExpanded}
            mode={recordingMode}
            onPress={(event) => {
              void hideRecordingOptions();
              const expanded = !isSourceSelectorExpandedRef.current;
              const focusContents = ["keyboard", "virtual"].includes(
                event.pointerType,
              );
              isSourceSelectorExpandedRef.current = expanded;
              setIsSourceSelectorExpanded(expanded);
              const transition = expanded
                ? expandRecordingSourceSelector(
                    recordingMode === "window",
                    focusContents,
                  )
                : collapseRecordingSourceSelector(focusContents);
              void transition.catch(() =>
                getRecordingSourceSelectorState().then((state) => {
                  isSourceSelectorExpandedRef.current = state.expanded;
                  setIsSourceSelectorExpanded(state.expanded);
                }),
              );
            }}
            selectedMonitor={selectedMonitor}
            selectedWindow={selectedWindow}
          />
          {recordingMode === "region" ? <RegionSourceControls /> : null}
          {recordingMode === "window" ? (
            <WindowSourceControls selectedWindow={selectedWindow} />
          ) : null}
        </>
      }
      status={status}
    />
  );
}
