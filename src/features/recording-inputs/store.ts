// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

import { t } from "../../i18n/i18n";
import {
  legacyRecordingFormat,
  moveRecordingFormat,
} from "../settings/recording-format-migration";

import {
  CameraDevice,
  CameraResolution,
  InputDevice,
  RecordingInputs,
  SystemAudioSource,
} from "./types";

const STORE_NAME = "screenwide-recording-inputs";
/** 1: the frame rate and anti-flicker left for the settings. */
const STORE_VERSION = 1;

/** Every app's sound. Its name is the app's language, read through
 * `systemAudioLabel` when shown, so it is not stored with the selection. */
export const ALL_SYSTEM_AUDIO: SystemAudioSource = {
  id: "all",
  kind: "all",
  label: "",
};

/** The name a system audio source is shown under. */
export const systemAudioLabel = (source: SystemAudioSource) =>
  source.kind === "all" ? t("recording-inputs-all-audio") : source.label;

/** Older builds persisted inputs that are no longer offered, such as the
 * cursor and keyboard overlays that are now always recorded. Only the inputs
 * still in the type are carried across. */
const mergeInputs = (
  current: RecordingInputs,
  persisted: unknown,
): RecordingInputs => {
  const stored = (
    persisted && typeof persisted === "object" ? persisted : {}
  ) as Partial<RecordingInputs>;
  return {
    camera: stored.camera ?? current.camera,
    microphone: stored.microphone ?? current.microphone,
    systemAudio: stored.systemAudio ?? current.systemAudio,
  };
};

const isCameraResolution = (value: unknown): value is CameraResolution => {
  if (!value || typeof value !== "object") return false;
  const mode = value as Partial<CameraResolution>;
  return (
    typeof mode.id === "string" &&
    typeof mode.width === "number" &&
    typeof mode.height === "number" &&
    typeof mode.fps === "number"
  );
};

const isCameraDevice = (value: unknown): value is CameraDevice => {
  if (!value || typeof value !== "object") return false;
  const camera = value as Partial<CameraDevice>;
  return (
    typeof camera.id === "string" &&
    typeof camera.label === "string" &&
    Array.isArray(camera.modes) &&
    camera.modes.every(isCameraResolution)
  );
};

type RecordingInputStore = {
  cameraFlippedById: Record<string, boolean>;
  inputs: RecordingInputs;
  selectedCamera: CameraDevice | null;
  selectedCameraMode: CameraResolution | null;
  selectedMicrophone: InputDevice | null;
  selectedSystemAudio: SystemAudioSource[];
  setCameraFlipped: (cameraId: string, flipped: boolean) => void;
  setInput: (input: keyof RecordingInputs, selected: boolean) => void;
  setSelectedCameraSelection: (
    camera: CameraDevice | null,
    mode: CameraResolution | null,
  ) => void;
  setSelectedMicrophone: (microphone: InputDevice | null) => void;
  setSelectedSystemAudio: (sources: SystemAudioSource[]) => void;
};

/** The inputs' storage from before the frame rate and anti-flicker moved to
 * the settings (`recording-format-migration.ts`): what it held of them is
 * handed over, once, and `merge` drops it. */
const migrateRecordingInputs = (persisted: unknown, version: number) => {
  if (version < STORE_VERSION)
    moveRecordingFormat(legacyRecordingFormat(persisted)).catch(
      (error: unknown) => {
        console.error("Could not keep the frame rate and anti-flicker", error);
      },
    );
  return persisted as RecordingInputStore;
};

export const useRecordingInputStore = create<RecordingInputStore>()(
  persist(
    (set) => ({
      cameraFlippedById: {},
      inputs: {
        camera: false,
        microphone: false,
        systemAudio: false,
      },
      selectedCamera: null,
      selectedCameraMode: null,
      selectedMicrophone: null,
      selectedSystemAudio: [ALL_SYSTEM_AUDIO],
      setCameraFlipped: (cameraId, flipped) => {
        set((state) => ({
          cameraFlippedById: {
            ...state.cameraFlippedById,
            [cameraId]: flipped,
          },
        }));
      },
      setInput: (input, selected) => {
        set((state) => ({
          inputs: { ...state.inputs, [input]: selected },
        }));
      },
      setSelectedCameraSelection: (selectedCamera, selectedCameraMode) => {
        set({ selectedCamera, selectedCameraMode });
      },
      setSelectedMicrophone: (selectedMicrophone) => {
        set({ selectedMicrophone });
      },
      setSelectedSystemAudio: (selectedSystemAudio) => {
        set({ selectedSystemAudio });
      },
    }),
    {
      merge: (persistedState, currentState) => {
        // Older builds remembered a mode per camera, which cameras no longer
        // have: they record at their largest picture. The frame rate and
        // anti-flicker are settings now (`migrateRecordingInputs`).
        const {
          cameraModeIdById: _cameraModeIdById,
          cameraPal: _cameraPal,
          cameraPalById: _cameraPalById,
          fps: _fps,
          ...persisted
        } = persistedState as Partial<RecordingInputStore> & {
          cameraModeIdById?: unknown;
          cameraPal?: unknown;
          cameraPalById?: unknown;
          fps?: unknown;
        };
        return {
          ...currentState,
          ...persisted,
          cameraFlippedById:
            persisted.cameraFlippedById &&
            typeof persisted.cameraFlippedById === "object"
              ? persisted.cameraFlippedById
              : {},
          inputs: mergeInputs(currentState.inputs, persisted.inputs),
          // Older builds persisted only a physical camera. A formatless
          // camera cannot faithfully preview or record, so discovery chooses
          // a real mode instead of guessing behind the user's back.
          selectedCamera: isCameraDevice(persisted.selectedCamera)
            ? persisted.selectedCamera
            : null,
          selectedCameraMode: isCameraResolution(persisted.selectedCameraMode)
            ? persisted.selectedCameraMode
            : null,
          selectedSystemAudio: Array.isArray(persisted.selectedSystemAudio)
            ? persisted.selectedSystemAudio
            : [ALL_SYSTEM_AUDIO],
        };
      },
      migrate: migrateRecordingInputs,
      name: STORE_NAME,
      storage: createJSONStorage(() => localStorage),
      version: STORE_VERSION,
    },
  ),
);

export const synchronizeRecordingInputStore = (event: StorageEvent) => {
  if (event.key === STORE_NAME) {
    void useRecordingInputStore.persist.rehydrate();
  }
};
