// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { PreviewInput } from "../../../bindings/PreviewInput";
import type { PreviewPlacement } from "../../../bindings/PreviewPlacement";
import type { PreviewScene } from "../../../bindings/PreviewScene";

// The app's side of a setting's live picture (`src-tauri/src/settings_
// preview.rs`): a native surface over a box in the page, drawn by the
// tool's own renderer or by the editor's compositor.

/** Pictures painted by the page, every one the same size, as RGBA in
 * device pixels. */
export type PaintedPictures = {
  height: number;
  rgba: Uint8Array[];
  width: number;
};

/** The size the pictures are shown at, in points. */
export type PictureSize = { height: number; width: number };

// A page loaded again leaves its previews open in the window, still drawn,
// so the first preview a page opens closes them.
let leftovers: Promise<unknown> | null = null;

function closeLeftovers() {
  leftovers ??= invoke("close_settings_previews").catch(() => undefined);
  return leftovers;
}

/** Opens `scene` as the preview `id`, drawn on `pictures` shown at `size`.
 * It stays hidden until placed. */
export async function openSettingsPreview(
  id: string,
  scene: PreviewScene,
  { pictures, size }: { pictures: PaintedPictures; size: PictureSize },
) {
  await closeLeftovers();
  const body = new Uint8Array(
    pictures.rgba.reduce((length, rgba) => length + rgba.length, 0),
  );
  pictures.rgba.reduce((offset, rgba) => {
    body.set(rgba, offset);
    return offset + rgba.length;
  }, 0);
  await invoke("open_settings_preview", body, {
    headers: {
      "x-picture-count": String(pictures.rgba.length),
      "x-picture-height": String(pictures.height),
      "x-picture-width": String(pictures.width),
      "x-preview-id": id,
      "x-preview-scene": JSON.stringify(scene),
      "x-size-height": String(size.height),
      "x-size-width": String(size.width),
    },
  });
}

export const placeSettingsPreview = (id: string, placement: PreviewPlacement) =>
  invoke("place_settings_preview", { id, placement });

/** A tool's pointer, in the picture's points. */
export const sendSettingsPreviewInput = (
  id: string,
  input: PreviewInput,
  { x, y } = { x: 0, y: 0 },
) => invoke("settings_preview_input", { id, input, x, y });

/** The open scene with other values; another kind needs a new preview. */
export const setSettingsPreviewScene = (id: string, scene: PreviewScene) =>
  invoke("set_settings_preview_scene", { id, scene });

export const closeSettingsPreview = (id: string) =>
  invoke("close_settings_preview", { id });
