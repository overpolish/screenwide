// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  getCurrentWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";

import { hidePopupPanel, showPopupPanel } from "../../popup-panel/api";
import { initialPopupPanelHeight } from "../../popup-panel/layout";
import { PopupPanelItem, usePopupPanelStore } from "../../popup-panel/store";
import { ALL_SYSTEM_AUDIO } from "../../recording-inputs/store";
import {
  CameraDevice,
  CameraResolution,
  InputDevice,
  SystemAudioSource,
} from "../../recording-inputs/types";

import {
  CAMERA_OPTION_ITEMS,
  CAMERA_PANEL_ID,
  CAMERA_SECTION,
  cameraModeLabel,
  FLIP_OPTION_ID,
  MICROPHONE_PANEL_ID,
  MODE_ID_PREFIX,
  OFF_ITEM,
  PAL_OPTION_ID,
  PANEL_MIN_WIDTH,
  RESOLUTION_SECTION,
  SYSTEM_AUDIO_PANEL_ID,
  toItems,
} from "./recording-bar-input-state";

type OpenPanelOptions = {
  anchor: DOMRect;
  focusContents: boolean;
  id: string;
  items: PopupPanelItem[];
  label: string;
  selectedIds: string[];
  exclusiveId?: string;
  selectionMode?: "multiple" | "single";
};

const openPanel = async ({
  anchor,
  exclusiveId,
  focusContents,
  id,
  items,
  label,
  selectedIds,
  selectionMode = "single",
}: OpenPanelOptions) => {
  const { open } = usePopupPanelStore.getState();
  open({
    content: {
      exclusiveId,
      items,
      kind: "list",
      mode: "select",
      selectedIds,
      selectionMode,
    },
    focusContents,
    id,
    label,
  });
  await showPopupPanel({
    anchor: {
      height: anchor.height,
      width: anchor.width,
      x: anchor.left,
      y: anchor.top,
    },
    focusContents,
    offset: new LogicalPosition(anchor.left, anchor.bottom + 4),
    parentWindowLabel: getCurrentWindow().label,
    size: new LogicalSize(
      Math.max(anchor.width, PANEL_MIN_WIDTH),
      initialPopupPanelHeight(
        items.length,
        new Set(
          items
            .map((item) => item.section)
            .filter((section) => section !== undefined),
        ).size,
      ),
    ),
    triggerId: id,
  });
};

/** A press on an already-open picker dismisses it, as a pop-up button does.
 * The toggle beside a picker dismisses it too: the press lands inside the
 * picker's own trigger, which the outside-press watcher leaves alone. */
export const closeIfOpen = async (id: string) => {
  const current = usePopupPanelStore.getState().active;
  if (current?.id !== id) return false;
  usePopupPanelStore.getState().close();
  await hidePopupPanel(current.focusContents);
  return true;
};

type CameraPickerOptions = {
  anchor: DOMRect;
  focusContents: boolean;
  isCameraFlipped: boolean;
  isCameraOn: boolean;
  isCameraPal: boolean;
  refreshCameras: () => Promise<CameraDevice[]>;
  selectedCamera: CameraDevice | null;
  selectedCameraMode: CameraResolution | null;
};

export const showCameraPicker = async ({
  anchor,
  focusContents,
  isCameraFlipped,
  isCameraOn,
  isCameraPal,
  refreshCameras,
  selectedCamera,
  selectedCameraMode,
}: CameraPickerOptions) => {
  if (await closeIfOpen(CAMERA_PANEL_ID)) return;
  const devices = await refreshCameras();
  // The freshly listed device carries the modes to offer; the stored one is
  // the fallback for a camera that has just been unplugged.
  const camera = selectedCamera
    ? (devices.find((device) => device.id === selectedCamera.id) ??
      selectedCamera)
    : null;
  const modes = camera?.modes ?? [];
  const modeItems: PopupPanelItem[] = modes.map((mode) => ({
    id: `${MODE_ID_PREFIX}${mode.id}`,
    label: cameraModeLabel(mode, modes),
    section: RESOLUTION_SECTION,
  }));
  // Both settings belong to the camera they are remembered for, so they are
  // offered only once one is chosen.
  const optionItems = camera ? CAMERA_OPTION_ITEMS : [];
  await openPanel({
    anchor,
    focusContents,
    id: CAMERA_PANEL_ID,
    items: [
      ...[OFF_ITEM, ...toItems(devices)].map((item) => ({
        ...item,
        section: CAMERA_SECTION,
      })),
      ...optionItems,
      ...modeItems,
    ],
    label: "Camera",
    // A select list always shows its state: the camera in use, or Off, with
    // the chosen shape and whichever of the camera's settings are on.
    selectedIds: [
      isCameraOn && selectedCamera ? selectedCamera.id : OFF_ITEM.id,
      ...(selectedCameraMode && modeItems.length > 0
        ? [`${MODE_ID_PREFIX}${selectedCameraMode.id}`]
        : []),
      ...(camera && isCameraFlipped ? [FLIP_OPTION_ID] : []),
      ...(camera && isCameraPal ? [PAL_OPTION_ID] : []),
    ],
  });
};

type MicrophonePickerOptions = {
  anchor: DOMRect;
  focusContents: boolean;
  isMicrophoneOn: boolean;
  refreshMicrophones: () => Promise<InputDevice[]>;
  selectedMicrophone: InputDevice | null;
};

export const showMicrophonePicker = async ({
  anchor,
  focusContents,
  isMicrophoneOn,
  refreshMicrophones,
  selectedMicrophone,
}: MicrophonePickerOptions) => {
  if (await closeIfOpen(MICROPHONE_PANEL_ID)) return;
  const devices = await refreshMicrophones();
  await openPanel({
    anchor,
    focusContents,
    id: MICROPHONE_PANEL_ID,
    items: [OFF_ITEM, ...toItems(devices)],
    label: "Microphone",
    selectedIds: [
      isMicrophoneOn && selectedMicrophone
        ? selectedMicrophone.id
        : OFF_ITEM.id,
    ],
  });
};

type SystemAudioPickerOptions = {
  anchor: DOMRect;
  focusContents: boolean;
  refreshAudioSources: () => Promise<SystemAudioSource[]>;
  selectedSystemAudio: SystemAudioSource[];
};

export const showSystemAudioPicker = async ({
  anchor,
  focusContents,
  refreshAudioSources,
  selectedSystemAudio,
}: SystemAudioPickerOptions) => {
  if (await closeIfOpen(SYSTEM_AUDIO_PANEL_ID)) return;
  const sources = await refreshAudioSources();
  await openPanel({
    anchor,
    exclusiveId: ALL_SYSTEM_AUDIO.id,
    focusContents,
    id: SYSTEM_AUDIO_PANEL_ID,
    items: sources.map((source) => ({
      iconPath: source.iconPath,
      id: source.id,
      label: source.label,
    })),
    label: "System audio",
    selectedIds: selectedSystemAudio.map((source) => source.id),
    selectionMode: "multiple",
  });
};
