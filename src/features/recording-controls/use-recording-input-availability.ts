// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useMemo, useState } from "react";

import { t } from "../../i18n/i18n";
import {
  CameraModeStatus,
  getCameraModeStatus,
  listMicrophones,
  listSystemAudioSources,
} from "../recording-inputs/devices-api";
import {
  CameraDevice,
  CameraResolution,
  InputDevice,
  SystemAudioSource,
} from "../recording-inputs/types";

const AVAILABILITY_REFRESH_MS = 4_000;

const selectedDeviceIsDetected = (
  selected: InputDevice,
  detected: InputDevice[],
) =>
  detected.some((device) => device.id === selected.id) ||
  (selected.id === "default" && detected.some((device) => device.isDefault));

type DetectionResult<T> = {
  key: string;
  value: T;
};

type DetectionState = {
  camera: DetectionResult<CameraModeStatus> | null;
  microphone: DetectionResult<boolean> | null;
  systemAudio: DetectionResult<boolean> | null;
};

const initialDetection: DetectionState = {
  camera: null,
  microphone: null,
  systemAudio: null,
};

/** What a warned input's tooltip says. A missing input fails the start, so
 * each one is a reason the user can act on. */
const cameraWarning = (
  status: CameraModeStatus | undefined,
  mode: CameraResolution | null,
) => {
  if (status === "missing") return t("recording-controls-camera-missing");
  if (status === "modeUnavailable" && mode) {
    return t("recording-controls-camera-mode-missing", {
      fps: String(mode.fps),
      size: `${String(mode.width)} × ${String(mode.height)}`,
    });
  }
  return undefined;
};

/**
 * Whether each selected input could still be recorded, checked while the bar
 * is in use. An input that could not is returned as the warning its control
 * shows; the camera is asked about its exact mode, by the same lookups the
 * start makes, so the warning and the start never disagree.
 */
export function useRecordingInputAvailability({
  active,
  cameraEnabled,
  cameraPermissionGranted,
  microphoneEnabled,
  microphonePermissionGranted,
  screenRecordingPermissionGranted,
  selectedCamera,
  selectedCameraMode,
  selectedMicrophone,
  selectedSystemAudio,
  systemAudioEnabled,
}: {
  active: boolean;
  cameraEnabled: boolean;
  cameraPermissionGranted: boolean;
  microphoneEnabled: boolean;
  microphonePermissionGranted: boolean;
  screenRecordingPermissionGranted: boolean;
  selectedCamera: CameraDevice | null;
  selectedCameraMode: CameraResolution | null;
  selectedMicrophone: InputDevice | null;
  selectedSystemAudio: SystemAudioSource[];
  systemAudioEnabled: boolean;
}) {
  const selectedApplications = useMemo(
    () => selectedSystemAudio.filter((source) => source.kind === "application"),
    [selectedSystemAudio],
  );
  const checkCamera =
    active &&
    cameraEnabled &&
    cameraPermissionGranted &&
    selectedCamera !== null &&
    selectedCameraMode !== null;
  const checkMicrophone =
    active &&
    microphoneEnabled &&
    microphonePermissionGranted &&
    selectedMicrophone !== null;
  const checkSystemAudio =
    active &&
    systemAudioEnabled &&
    screenRecordingPermissionGranted &&
    selectedApplications.length > 0;
  const cameraKey = checkCamera
    ? `${selectedCamera.id}:${selectedCameraMode.id}`
    : null;
  const microphoneKey = checkMicrophone ? selectedMicrophone.id : null;
  const systemAudioKey = checkSystemAudio
    ? selectedApplications
        .map((source) => source.id)
        .sort()
        .join("\n")
    : null;
  const [detected, setDetected] = useState<DetectionState>(initialDetection);

  useEffect(() => {
    let disposed = false;
    let refreshing = false;

    const refresh = async () => {
      if (refreshing) return;
      refreshing = true;
      const [camera, microphones, applications] = await Promise.all([
        checkCamera
          ? getCameraModeStatus(selectedCamera.id, selectedCameraMode).catch(
              () => null,
            )
          : null,
        checkMicrophone ? listMicrophones().catch(() => null) : null,
        checkSystemAudio ? listSystemAudioSources().catch(() => null) : null,
      ]);
      refreshing = false;
      if (disposed) return;

      setDetected({
        camera: camera && cameraKey ? { key: cameraKey, value: camera } : null,
        microphone:
          microphones && selectedMicrophone && microphoneKey
            ? {
                key: microphoneKey,
                value: selectedDeviceIsDetected(
                  selectedMicrophone,
                  microphones,
                ),
              }
            : null,
        systemAudio:
          applications && systemAudioKey
            ? {
                key: systemAudioKey,
                value: selectedApplications.every((selected) =>
                  applications.some((source) => source.id === selected.id),
                ),
              }
            : null,
      });
    };

    if (checkCamera || checkMicrophone || checkSystemAudio) {
      void refresh();
      const interval = window.setInterval(() => {
        void refresh();
      }, AVAILABILITY_REFRESH_MS);
      return () => {
        disposed = true;
        window.clearInterval(interval);
      };
    }

    return () => {
      disposed = true;
    };
  }, [
    checkCamera,
    checkMicrophone,
    checkSystemAudio,
    cameraKey,
    microphoneKey,
    selectedApplications,
    selectedCamera,
    selectedCameraMode,
    selectedMicrophone,
    systemAudioKey,
  ]);

  const camera =
    detected.camera?.key === cameraKey ? detected.camera.value : undefined;
  return {
    cameraWarning: cameraWarning(camera, selectedCameraMode),
    microphoneWarning:
      detected.microphone?.key === microphoneKey && !detected.microphone.value
        ? t("recording-controls-microphone-missing")
        : undefined,
    systemAudioWarning:
      detected.systemAudio?.key === systemAudioKey &&
      !detected.systemAudio.value
        ? t("recording-controls-app-missing")
        : undefined,
  };
}
