// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useCallback, useEffect, useState } from "react";

import { bestCameraMode } from "../../recording-inputs/camera-mode";
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
  SystemAudioSource,
} from "../../recording-inputs/types";

import type { IslandInputKind } from "./island-types";

/** A list that could not be read is empty, as is one the Storybook Tauri
 * mock answers with `null`. */
const listOrEmpty = async <T>(list: () => Promise<T[]>): Promise<T[]> => {
  const result = await list().catch(() => []);
  return Array.isArray(result) ? result : [];
};

const requestFps = () => {
  const { cameraPal, fps } = useRecordingInputStore.getState();
  return cameraRequestFps(fps, cameraPal);
};

/** The camera chosen, at its largest picture among the modes it offers at
 * the cadences now asked for, kept so whenever the list is read again. */
const selectCamera = (camera: CameraDevice) => {
  const state = useRecordingInputStore.getState();
  const mode = bestCameraMode(camera, requestFps());
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
 * are read again when the frame rate or anti-flicker changes, as the modes
 * they offer follow both.
 */
export function useIslandDevices({
  isActive,
  isCameraLocked,
  isMicrophoneLocked,
}: {
  isActive: boolean;
  isCameraLocked: boolean;
  isMicrophoneLocked: boolean;
}) {
  const fps = useRecordingInputStore((state) => state.fps);
  const cameraPal = useRecordingInputStore((state) => state.cameraPal);
  const [cameras, setCameras] = useState<CameraDevice[]>([]);
  const [microphones, setMicrophones] = useState<InputDevice[]>([]);
  const [audioSources, setAudioSources] = useState<SystemAudioSource[]>([
    ALL_SYSTEM_AUDIO,
  ]);

  useEffect(() => {
    if (!isActive || isCameraLocked) return;
    let disposed = false;
    void listOrEmpty(() => listCameras(cameraRequestFps(fps, cameraPal))).then(
      (next) => {
        if (disposed) return;
        setCameras(next);
        const selected = useRecordingInputStore.getState().selectedCamera;
        const camera =
          next.find((device) => device.id === selected?.id) ??
          (selected ? null : firstDefault(next));
        if (camera) selectCamera(camera);
      },
    );
    return () => {
      disposed = true;
    };
  }, [cameraPal, fps, isActive, isCameraLocked]);

  useEffect(() => {
    if (!isActive || isMicrophoneLocked) return;
    let disposed = false;
    void listOrEmpty(listMicrophones).then((next) => {
      if (disposed) return;
      setMicrophones(next);
      const state = useRecordingInputStore.getState();
      if (!state.selectedMicrophone) {
        state.setSelectedMicrophone(firstDefault(next));
      }
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
        if (camera) selectCamera(camera);
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
    [audioSources, cameras, microphones],
  );

  return { audioSources, cameras, changeDevice, microphones };
}
