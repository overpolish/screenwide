// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { CameraPick } from "../../../../bindings/CameraPick";

export type { CameraPick };

/** Told with the recording's id each time the objects picked in its camera
 * change; `src-tauri/src/editor/camera_effects/picks/commands.rs`. */
export const listenToCameraPicksChanged = (
  onChanged: (artifactId: number) => void,
) =>
  listen<number>("editor://camera-picks-changed", (event) => {
    onChanged(event.payload);
  });

/** An object on the preview: `chosen` as a pick is made on it,
 * or `left` once the preview leaves the frame it was picked in;
 * `src-tauri/src/editor/recording_preview_player/camera_picking.rs`. */
export const listenToCameraObject = (
  what: "chosen" | "left",
  onObject: (object: { artifactId: number; id: number }) => void,
) =>
  listen<{ artifactId: number; id: number }>(
    `editor://camera-object-${what}`,
    (event) => {
      onObject(event.payload);
    },
  );

/** How far following the objects `ids` has got, 0 to 1; objects not yet
 * followed and not among them wait their turn;
 * `src-tauri/src/editor/camera_effects/picks/following.rs`. */
export type CameraObjectsFollowing = {
  artifactId: number;
  fraction: number;
  ids: number[];
};

export const listenToCameraObjectsFollowing = (
  onFollowing: (following: CameraObjectsFollowing) => void,
) =>
  listen<CameraObjectsFollowing>(
    "editor://camera-objects-following",
    (event) => {
      onFollowing(event.payload);
    },
  );

/** The objects picked in the camera. Asking starts following any that are
 * not followed yet. */
export const getRecordingCameraPicks = (artifactId: number) =>
  invoke<CameraPick[]>("get_recording_camera_picks", { artifactId });

/** Picks objects on the preview, which then draws the camera as recorded
 * with the objects picked in the frame shown marked, `chosen` among them;
 * or stops. A click on the camera picks the object under it. */
export const setRecordingCameraPicking = (
  artifactId: number,
  picking: boolean,
  chosen: number | null,
) =>
  invoke<null>("set_recording_camera_picking", {
    artifactId,
    chosen,
    picking,
  });

/** Takes an object out of the picks, answering with the objects picked now. */
export const removeRecordingCameraPick = (artifactId: number, id: number) =>
  invoke<CameraPick[]>("remove_recording_camera_pick", { artifactId, id });
