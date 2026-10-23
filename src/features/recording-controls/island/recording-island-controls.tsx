// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { useState } from "react";

import {
  openPermissionSettings,
  openPermissionsWindow,
  requestPermission,
} from "../../permissions/api";
import {
  selectCanRecordCamera,
  selectCanRecordMicrophone,
  selectCanRecordScreen,
  selectCanScreenshot,
  usePermissionStore,
} from "../../permissions/store";
import { useRecordingInputStore } from "../../recording-inputs/store";
import { useRecordingSourceStore } from "../../recording-sources/store";
import { type RecordingMode } from "../../recording-sources/types";
import { useGeneralSettings } from "../../settings/use-general-settings";
import {
  cancelRecording,
  pauseRecording,
  resumeRecording,
  startRecording,
  stopRecording,
} from "../api";
import { canStartRecording, type RecordingReadiness } from "../can-record";
import { startRecordingOptions } from "../recording-request";
import { selectSnapshot, useRecordingStore } from "../store";
import { type RecordingStatus } from "../types";
import { useElapsedTime } from "../use-elapsed-time";
import { usePlacedMoment } from "../use-placed-moment";
import { useRecordingInputAvailability } from "../use-recording-input-availability";
import { useRecordingMonitor } from "../use-recording-monitor";
import { useRecordingShortcuts } from "../use-recording-shortcuts";
import { useRecordingWrittenBytes } from "../use-recording-written-bytes";
import { useReplayBuffer } from "../use-replay-buffer";
import { useScreenshotCapture } from "../use-screenshot-capture";

import { RecordingIsland } from "./recording-island";
import { regionDisplays } from "./region-displays";
import { useIslandInputs } from "./use-island-inputs";
import { useIslandSources } from "./use-island-sources";
import {
  cancelWindowPick,
  changeRecordingMode,
  chooseWindow,
  useRecordingUiSession,
} from "./use-recording-ui-session";

import type { IslandMode, IslandPanel, IslandPhase } from "./island-types";
import type { AnnotateRows } from "../../annotate/island/annotate-rows";
import type { IslandHost } from "../../island/island-window";

const report = (action: string) => (error: unknown) => {
  console.error(`Could not ${action} the recording`, error);
};

/** A countdown is the start's first stretch, while seconds remain. */
const phaseOf = (status: RecordingStatus, countdown: number): IslandPhase => {
  if (status === "starting") return countdown > 0 ? "countdown" : "starting";
  if (status === "stopping") return "finishing";
  return status;
};

/** No screen records the camera when it is on, and the audio alone when not. */
const islandModeOf = (mode: RecordingMode): IslandMode =>
  mode === "camera" || mode === "audio" ? "none" : mode;

/** The recording controls, as the island shows them: what it records and
 * with what, from the stores, and the recording, from Rust. While Annotate
 * draws during a recording, its tools lie under the recording's row. */
export function RecordingIslandControls({
  annotate,
  island,
}: {
  island: IslandHost;
  annotate?: AnnotateRows;
}) {
  const settings = useGeneralSettings();
  // The checks are on until the stored settings say otherwise.
  const hasConfidenceChecks = settings?.showRecordingConfidenceChecks ?? true;
  const hydrated = usePermissionStore((state) => state.hydrated);
  const permissions = usePermissionStore((state) => state.permissions);
  const canRecordCamera = usePermissionStore(selectCanRecordCamera);
  const canRecordMicrophone = usePermissionStore(selectCanRecordMicrophone);
  const canRecordScreen = usePermissionStore(selectCanRecordScreen);
  const canScreenshot = usePermissionStore(selectCanScreenshot);
  const snapshot = useRecordingStore(selectSnapshot);
  const { status } = snapshot;
  const sources = useRecordingSourceStore((state) => state);
  const { recordingMode, selectedMonitor, selectedWindow } = sources;
  const inputStore = useRecordingInputStore((state) => state);
  const { isCaptureOverlayActive, isRecordingUiVisible, windowPick } =
    useRecordingUiSession();
  useRecordingShortcuts();
  const screenshot = useScreenshotCapture();
  const isLive = status === "recording" || status === "paused";
  const isOpenAndIdle = isRecordingUiVisible && status === "idle";
  const displays = useIslandSources(isOpenAndIdle);
  const monitor = useRecordingMonitor(hasConfidenceChecks && status !== "idle");
  const elapsedMs = useElapsedTime(snapshot);
  const moment = usePlacedMoment();
  const writtenBytes = useRecordingWrittenBytes(
    settings?.showRecordingSize ?? false,
    status,
  );

  const [panel, setPanel] = useState<IslandPanel>("source");
  const isCameraLocked = !hydrated || !canRecordCamera;
  const isMicrophoneLocked = !hydrated || !canRecordMicrophone;
  const availability = useRecordingInputAvailability({
    active:
      isOpenAndIdle &&
      !isCaptureOverlayActive &&
      !sources.isScreenshotCapture &&
      screenshot.screenshotFeedback.state !== "pending",
    cameraEnabled: inputStore.inputs.camera || recordingMode === "camera",
    cameraPermissionGranted: hydrated && permissions.camera.granted,
    microphoneEnabled: inputStore.inputs.microphone,
    microphonePermissionGranted: hydrated && permissions.microphone.granted,
    screenRecordingPermissionGranted:
      hydrated && permissions.screenRecording.granted,
    selectedCamera: inputStore.selectedCamera,
    selectedCameraMode: inputStore.selectedCameraMode,
    selectedMicrophone: inputStore.selectedMicrophone,
    selectedSystemAudio: inputStore.selectedSystemAudio,
    systemAudioEnabled: inputStore.inputs.systemAudio,
  });
  const inputs = useIslandInputs({
    isCameraLocked,
    isCameraShown: island.isShown && panel === "camera",
    isLive,
    isMicrophoneLocked,
    isPreviewActive: isOpenAndIdle,
    monitor,
    warnings: {
      camera: availability.cameraWarning,
      microphone: availability.microphoneWarning,
      systemAudio: availability.systemAudioWarning,
    },
  });
  const readiness: RecordingReadiness = {
    hasCameraWarning: availability.cameraWarning !== undefined,
    hasMicrophoneWarning: availability.microphoneWarning !== undefined,
    hasSelectedMonitor: selectedMonitor !== null,
    hasSelectedWindow: selectedWindow !== null,
    hasSystemAudioWarning: availability.systemAudioWarning !== undefined,
    inputs: inputStore.inputs,
    isCameraLocked,
    isMicrophoneLocked,
    isScreenLocked: hydrated && !canRecordScreen,
    mode: recordingMode,
  };
  // Not gated on an idle recording as Record is: the buffer runs beside a
  // recording, and the tray offers it during one.
  const { replay, setReplayOn } = useReplayBuffer(canStartRecording(readiness));

  return (
    <RecordingIsland
      {...inputs}
      annotate={annotate}
      chosenRegion={
        recordingMode === "region" && selectedMonitor
          ? {
              displays: regionDisplays(
                sources.region,
                selectedMonitor,
                displays,
              ).map(({ name }) => name),
              height: sources.region.size.height,
              width: sources.region.size.width,
            }
          : undefined
      }
      chosenWindow={
        selectedWindow
          ? {
              appName: selectedWindow.appName,
              icon: selectedWindow.appIconPath
                ? convertFileSrc(selectedWindow.appIconPath)
                : undefined,
              title: selectedWindow.title,
            }
          : undefined
      }
      countdown={snapshot.countdownSecondsRemaining}
      displays={displays.map((display) => ({
        id: display.id,
        label: display.name,
        thumbnail: sources.monitorThumbnails[display.id],
      }))}
      elapsed={elapsedMs / 1_000}
      hasConfidenceChecks={hasConfidenceChecks}
      isScreenLocked={readiness.isScreenLocked}
      isShown={island.isShown}
      mode={islandModeOf(recordingMode)}
      moment={moment}
      notch={island.notch}
      onCancelCountdown={() => {
        cancelRecording().catch(report("cancel"));
      }}
      onCancelStarting={() => {
        cancelRecording().catch(report("cancel"));
      }}
      onCancelWindowPick={cancelWindowPick}
      onChooseWindow={chooseWindow}
      onDiscard={() => {
        cancelRecording().catch(report("discard"));
      }}
      onDisplayChange={(id) => {
        const display = displays.find((candidate) => candidate.id === id);
        if (display) sources.setSelectedMonitor(display);
      }}
      onHidden={island.onHidden}
      onModeChange={(mode) => {
        changeRecordingMode(
          mode === "none"
            ? inputStore.inputs.camera
              ? "camera"
              : "audio"
            : mode,
        );
      }}
      onOpenPermissions={() => {
        void openPermissionsWindow();
      }}
      onPanelChange={setPanel}
      onPause={() => {
        pauseRecording().catch(report("pause"));
      }}
      onRecord={() => {
        startRecording(startRecordingOptions()).catch(report("start"));
      }}
      onReplayChange={setReplayOn}
      onResume={() => {
        resumeRecording().catch(report("resume"));
      }}
      onScreenshot={(action) => {
        if (hydrated && !canScreenshot) {
          void openPermissionsWindow();
        } else if (action === "delayed") {
          screenshot.takeDelayedScreenshot();
        } else if (action === "scrolling") {
          const { accessibility } = permissions;
          if (accessibility.granted) screenshot.takeScrollingScreenshot();
          else if (accessibility.canRequest)
            void requestPermission("accessibility");
          else void openPermissionSettings("accessibility");
        } else {
          screenshot.takeScreenshot(
            action === "clipboard" ? "clipboard" : "editor",
          );
        }
      }}
      onShapeChange={island.onShapeChange}
      onStop={() => {
        stopRecording().catch(report("stop"));
      }}
      phase={phaseOf(status, snapshot.countdownSecondsRemaining)}
      replay={replay.available ? { status: replay.status } : undefined}
      screenshotState={screenshot.screenshotFeedback.state}
      selectedDisplayId={
        selectedMonitor?.id ?? (displays.length > 0 ? displays[0].id : 0)
      }
      windowPick={
        windowPick && {
          hovered: windowPick.hovered && {
            appName: windowPick.hovered.appName,
            icon: windowPick.hovered.iconPath
              ? convertFileSrc(windowPick.hovered.iconPath)
              : undefined,
            title: windowPick.hovered.title,
          },
        }
      }
      writtenBytes={writtenBytes}
    />
  );
}
