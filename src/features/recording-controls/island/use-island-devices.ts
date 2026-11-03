// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useState } from "react";

import { bestCameraMode } from "../../recording-inputs/camera-mode";
import {
  listCameras,
  listMicrophones,
  listSystemAudioSources,
} from "../../recording-inputs/devices-api";
import { findSavedDevice } from "../../recording-inputs/saved-device";
import {
  ALL_SYSTEM_AUDIO,
  useRecordingInputStore,
} from "../../recording-inputs/store";
import {
  CameraDevice,
  cameraRequestFps,
  InputDevice,
  type RecordingFps,
  SystemAudioSource,
} from "../../recording-inputs/types";

import type { IslandInputKind } from "./island-types";

/** A list that could not be read is empty, as is one the Storybook Tauri
 * mock answers with `null`. */
const listOrEmpty = async <T>(list: () => Promise<T[]>): Promise<T[]> => {
  const result = await list().catch(() => []);
  return Array.isArray(result) ? result : [];
};

/** The camera chosen, at its largest picture among the modes it offers at
 * the cadences asked for, kept so whenever the list is read again. */
const selectCamera = (camera: CameraDevice, requestFps: number[]) => {
  const state = useRecordingInputStore.getState();
  const mode = bestCameraMode(camera, requestFps);
  if (
    state.selectedCamera?.id !== camera.id ||
    JSON.stringify(state.selectedCameraMode) !== JSON.stringify(mode)
  ) {
    state.setSelectedCameraSelection(camera, mode);
  }
};

const firstDefault = <T extends InputDevice>(devices: T[]) =>
  devices.find((device) => device.isDefault) ??
  (devices.length > 0 ? devices[0] : null);

/**
 * The devices behind the island's three inputs, read as the controls open so
 * each input can name what it would record, and an input with nothing chosen
 * yet takes the system's default rather than reading as unset. The cameras
 * are read again when the frame rate or anti-flicker changes in Settings, as
 * the modes they offer follow both.
 */
export function useIslandDevices({
  antiFlicker,
  fps,
  isActive,
  isCameraLocked,
  isMicrophoneLocked,
}: {
  antiFlicker: boolean;
  fps: RecordingFps;
  isActive: boolean;
  isCameraLocked: boolean;
  isMicrophoneLocked: boolean;
}) {
  const [cameras, setCameras] = useState<CameraDevice[]>([]);
  const [microphones, setMicrophones] = useState<InputDevice[]>([]);
  const [audioSources, setAudioSources] = useState<SystemAudioSource[]>([
    ALL_SYSTEM_AUDIO,
  ]);
  const cadences = cameraRequestFps(fps, antiFlicker).join(" ");

  useEffect(() => {
    if (!isActive || isCameraLocked) return;
    let disposed = false;
    const requestFps = cadences.split(" ").map(Number);
    void listOrEmpty(() => listCameras(requestFps)).then((next) => {
      if (disposed) return;
      setCameras(next);
      const state = useRecordingInputStore.getState();
      const selected = state.selectedCamera;
      const camera = selected
        ? findSavedDevice(selected, next)
        : firstDefault(next);
      if (!camera) return;
      // Found again under a new id, it keeps the mirroring chosen for it.
      const flips = state.cameraFlippedById;
      if (selected && selected.id in flips && !(camera.id in flips))
        state.setCameraFlipped(camera.id, flips[selected.id]);
      selectCamera(camera, requestFps);
    });
    return () => {
      disposed = true;
    };
  }, [cadences, isActive, isCameraLocked]);

  useEffect(() => {
    if (!isActive || isMicrophoneLocked) return;
    let disposed = false;
    void listOrEmpty(listMicrophones).then((next) => {
      if (disposed) return;
      setMicrophones(next);
      const state = useRecordingInputStore.getState();
      const selected = state.selectedMicrophone;
      const microphone = selected
        ? findSavedDevice(selected, next)
        : firstDefault(next);
      if (microphone && microphone.id !== selected?.id)
        state.setSelectedMicrophone(microphone);
    });
    return () => {
      disposed = true;
    };
  }, [isActive, isMicrophoneLocked]);

  useEffect(() => {
    if (!isActive) return;
    let disposed = false;
    void listOrEmpty(listSystemAudioSources).then((applications) => {
      if (!disposed) setAudioSources([ALL_SYSTEM_AUDIO, ...applications]);
    });
    return () => {
      disposed = true;
    };
  }, [isActive]);

  const changeDevice = useCallback(
    (kind: IslandInputKind, ids: string[]) => {
      const state = useRecordingInputStore.getState();
      if (kind === "camera") {
        const camera = cameras.find((device) => device.id === ids[0]);
        if (camera) selectCamera(camera, cadences.split(" ").map(Number));
      } else if (kind === "microphone") {
        const microphone = microphones.find((device) => device.id === ids[0]);
        if (microphone) state.setSelectedMicrophone(microphone);
      } else {
        const sources = audioSources.filter((source) =>
          ids.includes(source.id),
        );
        // Nothing chosen reads as the whole system, which the input's switch
        // turns off if nothing is wanted.
        state.setSelectedSystemAudio(
          sources.length > 0 ? sources : [ALL_SYSTEM_AUDIO],
        );
      }
    },
    [audioSources, cadences, cameras, microphones],
  );

  return { audioSources, cameras, changeDevice, microphones };
}
