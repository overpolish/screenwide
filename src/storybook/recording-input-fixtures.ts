// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ALL_SYSTEM_AUDIO,
  useRecordingInputStore,
} from "../features/recording-inputs/store";
import {
  CameraDevice,
  CameraResolution,
  InputDevice,
} from "../features/recording-inputs/types";

const builtInCameraMode: CameraResolution = {
  fps: 30,
  height: 1080,
  id: "1920x1080@30",
  isDefault: true,
  label: "1920 × 1080",
  width: 1920,
};

const builtInCamera: CameraDevice = {
  id: "facetime-hd",
  isDefault: true,
  label: "FaceTime HD Camera",
  modes: [
    builtInCameraMode,
    {
      fps: 30,
      height: 1920,
      id: "1080x1920@30",
      label: "1080 × 1920",
      width: 1080,
    },
    {
      fps: 30,
      height: 720,
      id: "1280x720@30",
      label: "1280 × 720",
      width: 1280,
    },
  ],
};

const builtInMicrophone: InputDevice = {
  id: "builtin-mic",
  isDefault: true,
  label: "MacBook Pro Microphone",
};

/**
 * Puts a typical Mac's devices in the recording input store, so a story that
 * names its inputs has names to show; Storybook's Tauri stub lists none.
 */
export const seedRecordingInputDevices = () => {
  useRecordingInputStore.setState({
    selectedCamera: builtInCamera,
    selectedCameraMode: builtInCameraMode,
    selectedMicrophone: builtInMicrophone,
    selectedSystemAudio: [ALL_SYSTEM_AUDIO],
  });
};
