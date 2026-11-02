// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

import { getPermissionSnapshot } from "../../permissions/api";
import { listCameras } from "../../recording-inputs/devices-api";
import { useRecordingInputStore } from "../../recording-inputs/store";
import {
  type CameraDevice,
  cameraRequestFps,
  type RecordingFps,
} from "../../recording-inputs/types";

/** The camera a recording would use, with the modes it offers at the
 * cadences the frame rate and anti-flicker ask for: the one chosen in the
 * controls, or the system's default. `null` while unknown and where there
 * is none to show: Settings never asks for the camera just to show a
 * preview, so only where access has been given already. */
export function usePreviewCamera(fps: RecordingFps, antiFlicker: boolean) {
  const [camera, setCamera] = useState<CameraDevice | null>(null);
  const cadences = cameraRequestFps(fps, antiFlicker).join(" ");

  useEffect(() => {
    if (!isTauri()) return;
    let current = true;
    void getPermissionSnapshot()
      .then(async (permissions) => {
        if (!permissions.camera.granted) return null;
        // Listing reads what is connected without opening a camera, so it
        // asks nothing of the person either.
        const cameras = await listCameras(cadences.split(" ").map(Number));
        const chosen = useRecordingInputStore.getState().selectedCamera;
        return (
          cameras.find((device) => device.id === chosen?.id) ??
          cameras.find((device) => device.isDefault) ??
          (cameras.length > 0 ? cameras[0] : null)
        );
      })
      .then(
        (found) => {
          if (current) setCamera(found);
        },
        () => {
          // No camera to show is the safe answer, and stays.
        },
      );
    return () => {
      current = false;
    };
  }, [cadences]);

  return camera;
}
