// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { StickerArt } from "../annotations/annotations";
import type { EditorKind } from "../types";

/** Where a dropped picture landed: the layer under it and the point in that
 * layer's image, each from 0 to 1. The twin of `StickerPoint` in
 * `src-tauri/src/editor/stickers/dropped.rs`. */
export type StickerPoint = { layer: number; x: number; y: number };

/** A picture dropped on a workspace, kept: which editor it was dropped on,
 * what the sticker shows, and where it landed, or null off every picture. */
export type StickerDrop = {
  art: StickerArt;
  at: StickerPoint | null;
  workspace: EditorKind;
};

export const STICKER_DROP_EVENT = "editor://sticker-drop";

/** Ask for a picture file and keep it for a sticker. Null when the picker
 * was dismissed. */
export const browseStickerImage = () =>
  invoke<StickerArt | null>("browse_sticker_image");

/** The clipboard's picture, kept for a sticker. Null when it holds none. */
export const readClipboardSticker = () =>
  invoke<StickerArt | null>("read_clipboard_sticker");

/** Place a sticker showing `art` on the screenshot `at` a point, or in the
 * middle of the layer in hand, and choose it. Whether one was placed: a
 * gesture or typing in progress places nothing. */
export const placeScreenshotSticker = (
  art: StickerArt,
  at: StickerPoint | null,
) => invoke<boolean>("place_screenshot_sticker", { art, at });

/** Place a sticker showing `art` on the recording at the playhead, `at` a
 * point, or in the middle of the pane in hand, and choose it. Whether one
 * was placed: nothing is placed while playing, or while a gesture or typing
 * is in progress. */
export const placeRecordingSticker = (
  art: StickerArt,
  at: StickerPoint | null,
) => invoke<boolean>("place_recording_sticker", { art, at });
