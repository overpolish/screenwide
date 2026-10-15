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

/** Turns Enhance on or off, answering once what it needs is made with what
 * the project says now. The first time takes a while. */
export const setRecordingCameraEffects = (
  artifactId: number,
  enhance: boolean,
) =>
  invoke<CameraEffects>("set_recording_camera_effects", {
    artifactId,
    enhance,
  });

/** Asks for a picture to put behind the person over the background range
 * `rangeId` and keeps it as that range's, answering with what the project
 * says now. */
export const chooseRecordingCameraBackground = (
  artifactId: number,
  rangeId: string,
) =>
  invoke<CameraEffects>("choose_recording_camera_background", {
    artifactId,
    rangeId,
  });
