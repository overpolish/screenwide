// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useRef, useState } from "react";

import { usePopupPanelStore } from "../../popup-panel/store";
import {
  listCameras,
  listMicrophones,
  listSystemAudioSources,
} from "../../recording-inputs/devices-api";
import {
  ALL_SYSTEM_AUDIO,
  useRecordingInputStore,
} from "../../recording-inputs/store";
import {
  CameraDevice,
  cameraRequestFps,
  InputDevice,
  RecordingInputs,
  SystemAudioSource,
} from "../../recording-inputs/types";

import {
  CAMERA_PANEL_ID,
  FLIP_OPTION_ID,
  firstOrNull,
  listOrEmpty,
  MICROPHONE_PANEL_ID,
  MODE_ID_PREFIX,
  modeForCamera,
  OFF_ITEM,
  OPTION_ID_PREFIX,
  PAL_OPTION_ID,
  SYSTEM_AUDIO_PANEL_ID,
} from "./recording-bar-input-state";

/** The camera's own settings are remembered per camera and toggled in place,
 * so a press changes that flag and leaves the chosen device alone. */
const toggleCameraOption = (optionId: string) => {
  const state = useRecordingInputStore.getState();
  const camera = state.selectedCamera;
  if (!camera) return;
  if (optionId === FLIP_OPTION_ID) {
    state.setCameraFlipped(
      camera.id,
      !(state.cameraFlippedById[camera.id] ?? false),
    );
  } else if (optionId === PAL_OPTION_ID) {
    state.setCameraPal(camera.id, !(state.cameraPalById[camera.id] ?? false));
  }
};

type RecordingBarInputDevicesOptions = {
  isCameraLocked: boolean;
  isMicrophoneLocked: boolean;
  onInputChange: (input: keyof RecordingInputs, selected: boolean) => void;
};

/**
 * The device lists behind the bar's three inputs: read on mount so each
 * control can name what it would record, refreshed whenever a picker opens,
 * and fanned back out when a picker reports a choice.
 */
export function useRecordingBarInputDevices({
  isCameraLocked,
  isMicrophoneLocked,
  onInputChange,
}: RecordingBarInputDevicesOptions) {
  const {
    setSelectedCameraMode,
    setSelectedCameraSelection,
    setSelectedMicrophone,
    setSelectedSystemAudio,
  } = useRecordingInputStore((state) => state);
  const lastSelection = usePopupPanelStore((state) => state.lastSelection);
  const [cameras, setCameras] = useState<CameraDevice[]>([]);
  const [microphones, setMicrophones] = useState<InputDevice[]>([]);
  const [audioSources, setAudioSources] = useState<SystemAudioSource[]>([
    ALL_SYSTEM_AUDIO,
  ]);
  const handledEventRef = useRef(lastSelection?.eventId ?? null);

  const refreshCameras = useCallback(async () => {
    const state = useRecordingInputStore.getState();
    const pal = state.selectedCamera
      ? (state.cameraPalById[state.selectedCamera.id] ?? false)
      : false;
    const next = await listOrEmpty(() =>
      listCameras(cameraRequestFps(state.fps, pal)),
    );
    setCameras(next);
    return next;
  }, []);

  const refreshMicrophones = useCallback(async () => {
    const next = await listOrEmpty(listMicrophones);
    setMicrophones(next);
    return next;
  }, []);

  const refreshAudioSources = useCallback(async () => {
    const applications = await listOrEmpty(listSystemAudioSources);
    const next = [ALL_SYSTEM_AUDIO, ...applications];
    setAudioSources(next);
    return next;
  }, []);

  const selectCamera = useCallback(
    (camera: CameraDevice) => {
      setSelectedCameraSelection(camera, modeForCamera(camera));
    },
    [setSelectedCameraSelection],
  );

  // The row names the devices it would record, so the lists are read once on
  // mount as well as when a picker opens, and an input with nothing chosen
  // yet adopts the system default rather than reading as unset.
  useEffect(() => {
    void (async () => {
      if (!isCameraLocked) {
        const devices = await refreshCameras();
        if (!useRecordingInputStore.getState().selectedCamera) {
          const camera =
            devices.find((device) => device.isDefault) ?? firstOrNull(devices);
          if (camera) selectCamera(camera);
        }
      }
      if (!isMicrophoneLocked) {
        const devices = await refreshMicrophones();
        if (!useRecordingInputStore.getState().selectedMicrophone) {
          setSelectedMicrophone(
            devices.find((device) => device.isDefault) ?? firstOrNull(devices),
          );
        }
      }
      await refreshAudioSources();
    })();
  }, [
    isCameraLocked,
    isMicrophoneLocked,
    refreshAudioSources,
    refreshCameras,
    refreshMicrophones,
    selectCamera,
    setSelectedMicrophone,
  ]);

  // Each picker reports through the shared store, so one effect fans the
  // latest selection out to whichever input it belongs to.
  useEffect(() => {
    if (
      !lastSelection ||
      lastSelection.eventId === handledEventRef.current ||
      ![CAMERA_PANEL_ID, MICROPHONE_PANEL_ID, SYSTEM_AUDIO_PANEL_ID].includes(
        lastSelection.id,
      )
    ) {
      return;
    }

    handledEventRef.current = lastSelection.eventId;
    const [selectedId] = lastSelection.selectedIds;

    if (lastSelection.id === CAMERA_PANEL_ID) {
      if (selectedId.startsWith(OPTION_ID_PREFIX)) {
        toggleCameraOption(selectedId);
        return;
      }
      if (selectedId === OFF_ITEM.id) {
        onInputChange("camera", false);
        return;
      }
      // A resolution belongs to the camera already chosen, so it changes the
      // shape rather than the device, and leaves the input on.
      if (selectedId.startsWith(MODE_ID_PREFIX)) {
        const modeId = selectedId.slice(MODE_ID_PREFIX.length);
        const current = useRecordingInputStore.getState().selectedCamera;
        const camera =
          cameras.find((item) => item.id === current?.id) ?? current;
        const mode = camera?.modes.find((item) => item.id === modeId);
        if (!mode) return;
        setSelectedCameraMode(mode);
        onInputChange("camera", true);
        return;
      }
      const camera = cameras.find((item) => item.id === selectedId);
      if (!camera) return;
      selectCamera(camera);
      onInputChange("camera", true);
      return;
    }

    if (lastSelection.id === MICROPHONE_PANEL_ID) {
      if (selectedId === OFF_ITEM.id) {
        onInputChange("microphone", false);
        return;
      }
      const microphone = microphones.find((item) => item.id === selectedId);
      if (!microphone) return;
      setSelectedMicrophone(microphone);
      onInputChange("microphone", true);
      return;
    }

    const sources = audioSources.filter((source) =>
      lastSelection.selectedIds.includes(source.id),
    );
    // Deselecting everything reads as "capture nothing", which the toggle
    // already expresses; the selection falls back to the whole system.
    setSelectedSystemAudio(sources.length > 0 ? sources : [ALL_SYSTEM_AUDIO]);
    onInputChange("systemAudio", sources.length > 0);
  }, [
    audioSources,
    cameras,
    lastSelection,
    microphones,
    onInputChange,
    selectCamera,
    setSelectedCameraMode,
    setSelectedMicrophone,
    setSelectedSystemAudio,
  ]);

  return {
    refreshAudioSources,
    refreshCameras,
    refreshMicrophones,
  };
}
