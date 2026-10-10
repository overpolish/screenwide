// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { CameraEffects } from "../../../../bindings/CameraEffects";
import type { CameraEffectsProgress } from "../../../../bindings/CameraEffectsProgress";

export type { CameraEffects, CameraEffectsProgress };

/** How far a camera effect has got, as the editor is told while it works;
 * `src-tauri/src/editor/camera_effects/commands.rs`. */
export const listenToCameraEffectsProgress = (
  onProgress: (progress: CameraEffectsProgress) => void,
) =>
  listen<CameraEffectsProgress>("editor://camera-effects-progress", (event) => {
    onProgress(event.payload);
  });

/** Told with the recording's id each time its camera is drawn differently;
 * `src-tauri/src/editor/recording_preview_player/camera_effects.rs`. */
export const listenToCameraEffectsChanged = (
  onChanged: (artifactId: number) => void,
) =>
  listen<number>("editor://camera-effects-changed", (event) => {
    onChanged(event.payload);
  });

/** The camera's effects. Where one is on and what it needs is still to
 * make, asking starts making it. */
export const getRecordingCameraEffects = (artifactId: number) =>
  invoke<CameraEffects>("get_recording_camera_effects", { artifactId });

/** Gives the camera these effects, answering once what they need is made
 * with what the project says now. The first time takes a while. */
export const setRecordingCameraEffects = (
  artifactId: number,
  effects: CameraEffects,
) =>
  invoke<CameraEffects>("set_recording_camera_effects", {
    artifactId,
    effects,
  });

/** Asks for a picture to put behind the person on camera and puts it there,
 * answering with what the project says now. */
export const chooseRecordingCameraBackground = (artifactId: number) =>
  invoke<CameraEffects>("choose_recording_camera_background", {
    artifactId,
  });
