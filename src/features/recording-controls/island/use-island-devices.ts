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
  SYSTEM_DEFAULT_MICROPHONE_ID,
  SystemAudioSource,
} from "../../recording-inputs/types";

import type { IslandInputKind, IslandPanel } from "./island-types";

/** How often an open input's devices are read again, in ms. */
const REFRESH_MS = 2_000;

/** A list that could not be read is empty, as is one the Storybook Tauri
 * mock answers with `null`. */
const listOrEmpty = async <T>(list: () => Promise<T[]>): Promise<T[]> => {
  const result = await list().catch(() => []);
  return Array.isArray(result) ? result : [];
};

/** Reads `list` into `apply` now and, while `isPolled`, every `REFRESH_MS`,
 * skipping a turn while the last read is still going. Returns the effect's
 * cleanup. */
const readDevices = <T>(
  list: () => Promise<T[]>,
  apply: (devices: T[]) => void,
  isPolled: boolean,
) => {
  let disposed = false;
  let reading = false;
  const read = () => {
    if (reading) return;
    reading = true;
    void listOrEmpty(list).then((devices) => {
      reading = false;
      if (!disposed) apply(devices);
    });
  };
  read();
  const interval = isPolled ? window.setInterval(read, REFRESH_MS) : undefined;
  return () => {
    disposed = true;
    window.clearInterval(interval);
  };
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
 * yet takes the system's default rather than reading as unset. An input's
 * devices are read again as its panel opens and while it stays open, so one
 * plugged in or removed shows there. The cameras are read again when the
 * frame rate or anti-flicker changes in Settings, as the modes they offer
 * follow both.
 */
export function useIslandDevices({
  antiFlicker,
  fps,
  isActive,
  isCameraLocked,
  isMicrophoneLocked,
  panel,
}: {
  antiFlicker: boolean;
  fps: RecordingFps;
  isActive: boolean;
  isCameraLocked: boolean;
  isMicrophoneLocked: boolean;
  /** The part the island has grown into. */
  panel: IslandPanel;
}) {
  const [cameras, setCameras] = useState<CameraDevice[]>([]);
  const [microphones, setMicrophones] = useState<InputDevice[]>([]);
  const [audioSources, setAudioSources] = useState<SystemAudioSource[]>([
    ALL_SYSTEM_AUDIO,
  ]);
  const cadences = cameraRequestFps(fps, antiFlicker).join(" ");
  const isCameraOpen = panel === "camera";
  const isMicrophoneOpen = panel === "microphone";
  const isSystemAudioOpen = panel === "systemAudio";

  useEffect(() => {
    if (!isActive || isCameraLocked) return;
    const requestFps = cadences.split(" ").map(Number);
    return readDevices(
      () => listCameras(requestFps),
      (next) => {
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
      },
      isCameraOpen,
    );
  }, [cadences, isActive, isCameraLocked, isCameraOpen]);

  useEffect(() => {
    if (!isActive || isMicrophoneLocked) return;
    return readDevices(
      listMicrophones,
      (next) => {
        setMicrophones(next);
        const state = useRecordingInputStore.getState();
        const selected = state.selectedMicrophone;
        // With nothing chosen, the island follows the system's default
        // rather than keeping the microphone that is the default now. Its
        // name follows the default too, so it is taken again as it changes.
        const microphone = selected
          ? findSavedDevice(selected, next)
          : (next.find(
              (device) => device.id === SYSTEM_DEFAULT_MICROPHONE_ID,
            ) ?? firstDefault(next));
        if (
          microphone &&
          (microphone.id !== selected?.id ||
            microphone.label !== selected.label)
        )
          state.setSelectedMicrophone(microphone);
      },
      isMicrophoneOpen,
    );
  }, [isActive, isMicrophoneLocked, isMicrophoneOpen]);

  useEffect(() => {
    if (!isActive) return;
    return readDevices(
      listSystemAudioSources,
      (applications) => {
        setAudioSources([ALL_SYSTEM_AUDIO, ...applications]);
      },
      isSystemAudioOpen,
    );
  }, [isActive, isSystemAudioOpen]);

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
