// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { ImageArt } from "../annotations/annotations";
import type { EditorKind } from "../types";

/** Where a dropped picture landed: the layer it joins and the point in that
 * layer's image, from 0 to 1 across it and beyond for a drop on the frame
 * around it. The twin of `ImagePoint` in
 * `src-tauri/src/editor/images/dropped.rs`. */
export type ImagePoint = { layer: number; x: number; y: number };

/** A picture dropped on a workspace, kept: which editor it was dropped on,
 * what the image shows, and where it landed, or null with no picture laid
 * out. */
export type ImageDrop = {
  art: ImageArt;
  at: ImagePoint | null;
  workspace: EditorKind;
};

export const IMAGE_DROP_EVENT = "editor://image-drop";

/** Ask for a picture file and keep it for an image. Null when the picker
 * was dismissed. */
export const browseAnnotationImage = () =>
  invoke<ImageArt | null>("browse_annotation_image");

/** The clipboard's picture, kept for an image. Null when it holds none. */
export const readClipboardImage = () =>
  invoke<ImageArt | null>("read_clipboard_image");

/** Place an image showing `art` on the screenshot `at` a point, or in the
 * middle of the layer in hand, and choose it. Whether one was placed: a
 * gesture or typing in progress places nothing. */
export const placeScreenshotImage = (art: ImageArt, at: ImagePoint | null) =>
  invoke<boolean>("place_screenshot_image", { art, at });

/** Place an image showing `art` on the recording at the playhead, `at` a
 * point, or in the middle of the pane in hand, and choose it. Whether one
 * was placed: nothing is placed while playing, or while a gesture or typing
 * is in progress. */
export const placeRecordingImage = (art: ImageArt, at: ImagePoint | null) =>
  invoke<boolean>("place_recording_image", { art, at });
