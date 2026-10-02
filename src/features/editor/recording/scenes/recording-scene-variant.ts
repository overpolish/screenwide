// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * How a preset is laid out: which side the camera takes and how much of the
 * pair it is, and for a picture in picture which corner it sits over and how
 * big it is. The twin of `SceneVariant` in
 * `src-tauri/src/editor/scenes/variant.rs`.
 */

/** How much of a side by side or stacked pair the camera takes. */
export type SceneCameraSize = "third" | "two-thirds";

/** The corner of the screen a picture in picture's camera sits over. */
export type SceneCorner =
  "bottom-left" | "bottom-right" | "top-left" | "top-right";

/** How big a picture in picture's camera is beside the screen. */
export type SceneBubbleSize = "large" | "small";

/** A preset's options, each its default where unset. */
export type SceneVariant = {
  cameraSize?: SceneCameraSize;
  corner?: SceneCorner;
  size?: SceneBubbleSize;
  /** The camera before the screen: on the left beside it, or above it. */
  swap?: boolean;
};

export const DEFAULT_CAMERA_SIZE: SceneCameraSize = "third";
export const DEFAULT_CORNER: SceneCorner = "bottom-right";
export const DEFAULT_BUBBLE_SIZE: SceneBubbleSize = "large";
