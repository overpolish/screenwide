// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import { type CropAspect } from "../../island/crop-aspect-choice";
import { EditorKind } from "../../types";

// The shape each layer's crop keeps as its handles are dragged, for as long
// as the editor is open: a way of cropping, chosen on the Crop tool's island,
// not part of the finished picture, so it is not saved with the project. A
// layer with none chosen crops freely. A locked crop records no ratio of its
// own: it keeps the shape it has, which a drag starts from.

type CropAspectStore = {
  aspects: Partial<Record<string, CropAspect>>;
  choose: (key: string, aspect: CropAspect) => void;
};

export const useCropAspectStore = create<CropAspectStore>()((set) => ({
  aspects: {},
  choose: (key, aspect) => {
    set((state) => ({ aspects: { ...state.aspects, [key]: aspect } }));
  },
}));

/** Which crop a shape belongs to: a screenshot's layer, by its id, or a
 * recording's track. */
export const cropAspectKey = (workspace: EditorKind, layer: number | string) =>
  `${workspace}:${String(layer)}`;

/** Which of a recording's crops a shape belongs to: the camera's, or the
 * screen's, which over the scene `sceneId` is that scene's own. */
export const recordingCropAspectKey = (
  track: "camera" | "primary",
  sceneId: string | null,
) =>
  cropAspectKey(
    "recording",
    track === "primary" && sceneId !== null ? `scene:${sceneId}` : track,
  );

/** Whether a crop with `aspect`, none where nothing was chosen, keeps its
 * shape as its handles are dragged: any but a free one. */
export const keepsShape = (aspect: CropAspect | undefined) =>
  aspect !== undefined && aspect !== "free";

/** The largest size of `aspect`'s shape that fits inside `size`, which is
 * what a crop given that shape keeps of the one it had: a ratio of width to
 * height, or `source`'s own for the whole capture's. `null` for a free crop,
 * which keeps any shape, and a locked one, which has its shape already. */
export const sizeOfAspect = (
  aspect: CropAspect,
  size: { height: number; width: number },
  source: { height: number; width: number },
) => {
  if (aspect === "free" || aspect === "locked") return null;
  const [width, height] =
    aspect === "original"
      ? [source.width, source.height]
      : aspect.split(":").map(Number);
  const ratio = width / height;
  return size.width / size.height > ratio
    ? { height: size.height, width: Math.round(size.height * ratio) }
    : { height: Math.round(size.width / ratio), width: size.width };
};
