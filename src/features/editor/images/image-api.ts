// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { ImageDrop } from "../../../bindings/ImageDrop";
import type { ImagePoint } from "../../../bindings/ImagePoint";
import type { ImageArt } from "../annotations/annotations";

export type { ImageDrop, ImagePoint };

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
