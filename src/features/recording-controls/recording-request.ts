// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRecordingInputStore } from "../recording-inputs/store";
import { SYSTEM_DEFAULT_MICROPHONE_ID } from "../recording-inputs/types";
import { useRecordingSourceStore } from "../recording-sources/store";
import { ScreenshotTarget } from "../screenshots/api";
import { getGeneralSettings } from "../settings/api";

import { StartRecordingOptions } from "./types";

import type { StartNotesOptions } from "../../bindings/StartNotesOptions";

/** What a recording started now records, from the controls' choices and
 * the stored settings: the frame rate and anti-flicker are read as it
 * starts, so a change in Settings reaches the next recording. Started by
 * its shortcut, with no island open to say the chosen microphone is not
 * connected, the system's default stands in for it. */
export const startRecordingOptions = async ({
  fromShortcut = false,
}: { fromShortcut?: boolean } = {}): Promise<StartRecordingOptions> => {
  const { cameraAntiFlicker, recordingFps } = await getGeneralSettings();
  const { recordingMode, region, selectedMonitor, selectedWindow } =
    useRecordingSourceStore.getState();
  const {
    cameraFlippedById,
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
    cameraPal: wantsCamera && selectedCamera ? cameraAntiFlicker : false,
    cameraWidth: wantsCamera ? (selectedCameraMode?.width ?? null) : null,
    // The cursor and the keyboard overlay are always recorded; the editor
    // decides whether either is shown.
    captureKeyboardShortcuts: true,
    fps: recordingFps,
    microphoneFallsBack: fromShortcut,
    microphoneId: inputs.microphone
      ? (selectedMicrophone?.id ?? SYSTEM_DEFAULT_MICROPHONE_ID)
      : null,
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

/** What notes started now listen to: the island's microphone, and its
 * system audio apps, or all of the computer's audio. Notes always hear both,
 * whatever the island's switches say: a call has two sides. */
export const notesOptions = (): StartNotesOptions => {
  const { selectedMicrophone, selectedSystemAudio } =
    useRecordingInputStore.getState();
  const applications = selectedSystemAudio.some(
    (source) => source.kind === "all",
  )
    ? []
    : selectedSystemAudio.filter((source) => source.kind === "application");
  return {
    microphoneId: selectedMicrophone?.id ?? null,
    systemAudioApplicationIds: applications.map((source) => source.id),
    systemAudioProcessIds: applications.flatMap(
      (source) => source.processIds ?? [],
    ),
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
