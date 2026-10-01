// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelItem } from "../../popup-panel/store";
import { useRecordingInputStore } from "../../recording-inputs/store";
import {
  CameraDevice,
  CameraResolution,
  InputDevice,
  SystemAudioSource,
} from "../../recording-inputs/types";

/** Identifies each picker in the shared popup panel window. */
export const CAMERA_PANEL_ID = "recording-bar-camera";
export const MICROPHONE_PANEL_ID = "recording-bar-microphone";
export const SYSTEM_AUDIO_PANEL_ID = "recording-bar-system-audio";
/** Device names read wider than the control they hang off. */
export const PANEL_MIN_WIDTH = 220;
/** The camera picker offers the cameras and, under them, the resolutions of
 * whichever one is chosen; a resolution's id is prefixed so the two runs stay
 * apart in the one flat list the panel carries. */
export const CAMERA_SECTION = "Camera";
export const RESOLUTION_SECTION = "Resolution";
export const MODE_ID_PREFIX = "mode:";
/** The camera's own settings, which are remembered per camera rather than
 * chosen: pressing one ticks or unticks it. */
const OPTIONS_SECTION = "Options";
export const OPTION_ID_PREFIX = "option:";
export const FLIP_OPTION_ID = `${OPTION_ID_PREFIX}flip`;
export const PAL_OPTION_ID = `${OPTION_ID_PREFIX}pal`;
export const CAMERA_OPTION_ITEMS: PopupPanelItem[] = [
  {
    id: FLIP_OPTION_ID,
    label: "Flip Horizontally",
    section: OPTIONS_SECTION,
    togglesInPlace: true,
  },
  {
    id: PAL_OPTION_ID,
    label: "Anti-Flicker 50 Hz",
    section: OPTIONS_SECTION,
    togglesInPlace: true,
  },
];

/** Turning the input off is the first choice in a device list. */
export const OFF_ITEM: PopupPanelItem = { id: "off", label: "Off" };

/** e.g. "1920 × 1080", the size a camera would record at. The frame rate is
 * named only where it tells two modes of the same size apart. */
export const cameraModeLabel = (
  mode: CameraResolution,
  modes: CameraResolution[],
) => {
  const size = `${mode.width.toString()} × ${mode.height.toString()}`;
  const isSizeShared = modes.some(
    (other) =>
      other.id !== mode.id &&
      other.width === mode.width &&
      other.height === mode.height,
  );
  return isSizeShared ? `${size} · ${mode.fps.toString()} fps` : size;
};

/** Whether one of a camera's own remembered settings is on, which nothing
 * without a camera can be. */
export const cameraOptionOn = (
  flags: Record<string, boolean>,
  camera: CameraDevice | null,
) => (camera ? (flags[camera.id] ?? false) : false);

export const firstOrNull = <T>(items: T[]): T | null =>
  items.length > 0 ? items[0] : null;

/** The Storybook Tauri mock answers unknown commands with `null`, and a
 * device list that could not be read is simply empty. */
export const listOrEmpty = async <T>(
  list: () => Promise<T[]>,
): Promise<T[]> => {
  const result = await list().catch(() => []);
  return Array.isArray(result) ? result : [];
};

export const toItems = (devices: InputDevice[]): PopupPanelItem[] =>
  devices.map((device) => ({ id: device.id, label: device.label }));

/** The mode a camera should record in: the one it was last used at, then the
 * current shape, then whatever the camera calls its default. */
export const modeForCamera = (
  camera: CameraDevice,
): CameraResolution | null => {
  const state = useRecordingInputStore.getState();
  const rememberedId = state.cameraModeIdById[camera.id];
  const current = state.selectedCameraMode;
  return (
    camera.modes.find((mode) => mode.id === rememberedId) ??
    camera.modes.find(
      (mode) => mode.width === current?.width && mode.height === current.height,
    ) ??
    camera.modes.find((mode) => mode.isDefault) ??
    firstOrNull(camera.modes)
  );
};

/** Each control is named by the device it would record, the same name its
 * picker shows as chosen. Nothing chosen names the absence instead. */
export const cameraName = (camera: CameraDevice | null) =>
  camera?.label ?? "No Camera";

export const microphoneName = (microphone: InputDevice | null) =>
  microphone?.label ?? "No Microphone";

/** The whole system is one source, so it names itself; a run of applications
 * is counted rather than listed, which no bezel is wide enough for. */
export const systemAudioName = (sources: SystemAudioSource[]) => {
  if (sources.length === 0) return "System Audio";
  if (sources.length === 1) return sources[0].label;
  return `${sources.length.toString()} Sources`;
};
