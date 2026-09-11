// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRecordingInputStore } from "../recording-inputs/store";
import { useRecordingSourceStore } from "../recording-sources/store";
import { ScreenshotTarget } from "../screenshots/api";

import { StartRecordingOptions } from "./types";

export const startRecordingOptions = (): StartRecordingOptions => {
  const { recordingMode, region, selectedMonitor, selectedWindow } =
    useRecordingSourceStore.getState();
  const {
    cameraFlippedById,
    cameraPalById,
    fps,
    inputs,
    selectedCamera,
    selectedCameraMode,
    selectedMicrophone,
    selectedSystemAudio,
  } = useRecordingInputStore.getState();
  const wantsCamera =
    recordingMode !== "audio" && (inputs.camera || recordingMode === "camera");
  const recordsAllSystemAudio = selectedSystemAudio.some(
    (source) => source.kind === "all",
  );
  const selectedApplications = recordsAllSystemAudio
    ? []
    : selectedSystemAudio.filter((source) => source.kind === "application");

  return {
    cameraFlipped:
      wantsCamera && selectedCamera
        ? (cameraFlippedById[selectedCamera.id] ?? false)
        : false,
    cameraFps: wantsCamera ? (selectedCameraMode?.fps ?? null) : null,
    cameraHeight: wantsCamera ? (selectedCameraMode?.height ?? null) : null,
    cameraId: wantsCamera ? (selectedCamera?.id ?? null) : null,
    cameraPal:
      wantsCamera && selectedCamera
        ? (cameraPalById[selectedCamera.id] ?? false)
        : false,
    cameraWidth: wantsCamera ? (selectedCameraMode?.width ?? null) : null,
    // The cursor and the keyboard overlay are always recorded; the editor
    // decides whether either is shown.
    captureKeyboardShortcuts: true,
    fps,
    microphoneId: inputs.microphone ? (selectedMicrophone?.id ?? null) : null,
    mode: recordingMode,
    monitorId: selectedMonitor?.id ?? null,
    region: recordingMode === "region" ? region : null,
    // The recording never carries the system cursor: cursor movement is
    // recorded beside it and Screenwide's own cursor is drawn at export.
    showCursor: false,
    systemAudio: inputs.systemAudio,
    systemAudioApplicationIds: inputs.systemAudio
      ? selectedApplications.map((source) => source.id)
      : [],
    systemAudioProcessIds: inputs.systemAudio
      ? selectedApplications.flatMap((source) => source.processIds ?? [])
      : [],
    windowId: selectedWindow?.id ?? null,
  };
};

/** Mirrors how `startRecordingOptions` pairs a region with its monitor. */
export const screenshotTarget = (
  desktopRegion = false,
): ScreenshotTarget | null => {
  const { recordingMode, region, selectedMonitor, selectedWindow } =
    useRecordingSourceStore.getState();

  if (recordingMode === "window") {
    return selectedWindow
      ? { kind: "window", windowId: selectedWindow.id }
      : null;
  }
  if (!selectedMonitor) return null;

  return recordingMode === "region"
    ? {
        kind: desktopRegion ? "desktopRegion" : "region",
        monitorId: selectedMonitor.id,
        region,
      }
    : { kind: "screen", monitorId: selectedMonitor.id };
};
