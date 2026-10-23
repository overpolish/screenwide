// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc } from "@tauri-apps/api/core";
import { createElement, useMemo } from "react";

import { decibelLevel } from "../../../primitives/audio-meter/decibel-level";
import { CameraThumbnail } from "../../recording-inputs/camera-thumbnail";
import {
  systemAudioLabel,
  useRecordingInputStore,
} from "../../recording-inputs/store";
import { useAudioPreview } from "../../recording-inputs/use-audio-preview";
import { useCameraPreview } from "../../recording-inputs/use-camera-preview";
import { useRecordingSourceStore } from "../../recording-sources/store";
import { type RecordingMonitorSnapshot } from "../use-recording-monitor";

import { useIslandDevices } from "./use-island-devices";
import { changeRecordingMode } from "./use-recording-ui-session";

import type { IslandInput, IslandInputKind } from "./island-types";

/** The island's picture box is the size; the picture fills it. */
const PICTURE_CLASS = "size-full rounded-none";

type Warnings = Record<IslandInputKind, string | undefined>;

/**
 * The island's three inputs, from the input store and the devices: on or
 * off, what each would record, why it cannot, and how loud it is. Set up,
 * each input that is on is previewed; recording, the levels and picture are
 * the recording's own, from its monitor.
 */
export function useIslandInputs({
  isCameraLocked,
  isCameraShown,
  isLive,
  isMicrophoneLocked,
  isPreviewActive,
  monitor,
  warnings,
}: {
  isCameraLocked: boolean;
  /** The camera's panel is open, the one place its picture shows; the
   * camera runs for nothing else, so its light is on only while it is seen. */
  isCameraShown: boolean;
  /** Recording or paused. */
  isLive: boolean;
  isMicrophoneLocked: boolean;
  /** The controls are open and idle, so previews may run. */
  isPreviewActive: boolean;
  monitor: RecordingMonitorSnapshot;
  warnings: Warnings;
}) {
  const store = useRecordingInputStore((state) => state);
  const recordingMode = useRecordingSourceStore((state) => state.recordingMode);
  const { audioSources, cameras, changeDevice, microphones } = useIslandDevices(
    { isActive: isPreviewActive, isCameraLocked, isMicrophoneLocked },
  );
  const { selectedCamera, selectedCameraMode, selectedSystemAudio } = store;
  const isCameraOn =
    recordingMode !== "audio" &&
    (store.inputs.camera || recordingMode === "camera");
  const isCameraFlipped = selectedCamera
    ? (store.cameraFlippedById[selectedCamera.id] ?? false)
    : false;
  const applications = useMemo(
    () => selectedSystemAudio.filter((source) => source.kind === "application"),
    [selectedSystemAudio],
  );
  const applicationIds = useMemo(
    () =>
      applications.length > 0
        ? applications.map((source) => source.id)
        : undefined,
    [applications],
  );
  const processIds = useMemo(
    () =>
      applications.length > 0
        ? applications.flatMap((source) => source.processIds ?? [])
        : undefined,
    [applications],
  );

  // Nothing is streamed for an input that is off.
  const cameraPreview = useCameraPreview({
    active:
      isPreviewActive &&
      isCameraShown &&
      isCameraOn &&
      !isCameraLocked &&
      selectedCamera !== null &&
      selectedCameraMode !== null,
    deviceId: selectedCamera?.id,
    mode: selectedCameraMode ?? undefined,
    pal: store.cameraPal,
  });
  const microphonePreview = useAudioPreview({
    active: isPreviewActive && store.inputs.microphone && !isMicrophoneLocked,
    deviceId: store.selectedMicrophone?.id,
    kind: "microphone",
  });
  const systemAudioPreview = useAudioPreview({
    active: isPreviewActive && store.inputs.systemAudio,
    applicationIds,
    kind: "system",
    processIds,
  });

  const camera: IslandInput = {
    devices: cameras.map((device) => {
      const mode = device.id === selectedCamera?.id ? selectedCameraMode : null;
      return {
        aspectRatio: mode ? mode.width / mode.height : undefined,
        id: device.id,
        label: device.label,
      };
    }),
    isLocked: isCameraLocked,
    isOn: isCameraOn,
    selectedIds: selectedCamera ? [selectedCamera.id] : [],
    warning: warnings.camera,
  };
  const microphone: IslandInput = {
    devices: microphones.map(({ id, label }) => ({ id, label })),
    isLocked: isMicrophoneLocked,
    isOn: store.inputs.microphone,
    level: decibelLevel(
      isLive ? monitor.microphoneDecibels : microphonePreview.decibels,
    ),
    selectedIds: store.selectedMicrophone ? [store.selectedMicrophone.id] : [],
    warning: warnings.microphone,
  };
  const systemAudio: IslandInput = {
    devices: audioSources.map((source) => ({
      icon: source.iconPath ? convertFileSrc(source.iconPath) : undefined,
      id: source.id,
      isPinned: source.kind === "all",
      label: systemAudioLabel(source),
    })),
    isOn: store.inputs.systemAudio,
    level: decibelLevel(
      isLive ? monitor.systemAudioDecibels : systemAudioPreview.decibels,
    ),
    selectedIds: selectedSystemAudio.map((source) => source.id),
    warning: warnings.systemAudio,
  };

  // Recording, the picture is the recording's, mirrored as it is recorded.
  const picture = isLive
    ? createElement(CameraThumbnail, {
        canvasRef: monitor.cameraCanvasRef,
        className: PICTURE_CLASS,
        frameSize: monitor.cameraFrameSize,
        hasFrame: monitor.hasCameraFrame,
      })
    : createElement(CameraThumbnail, {
        canvasRef: cameraPreview.canvasRef,
        className: PICTURE_CLASS,
        frameSize: cameraPreview.frameSize ?? selectedCameraMode,
        hasFrame: cameraPreview.hasFrame,
        isFlipped: isCameraFlipped,
      });

  return {
    camera,
    cameraPreview: picture,
    isCameraFlipped,
    microphone,
    onCameraFlipChange: (isFlipped: boolean) => {
      if (selectedCamera) store.setCameraFlipped(selectedCamera.id, isFlipped);
    },
    onDeviceChange: changeDevice,
    onInputChange: (kind: IslandInputKind, isOn: boolean) => {
      store.setInput(kind, isOn);
      // With no screen, the recording is the camera's when it is on, and
      // the audio's alone when it is off.
      if (
        kind === "camera" &&
        (recordingMode === "camera" || recordingMode === "audio")
      ) {
        changeRecordingMode(isOn ? "camera" : "audio");
      }
    },
    systemAudio,
  };
}
