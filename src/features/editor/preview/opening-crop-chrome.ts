// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { SceneRotation } from "../layer-rotation";
import type { CropTurn } from "./use-crop-turn";

type Box = { height: number; width: number; x: number; y: number };

/**
 * `selection` as the crop's chrome is drawn while the crop opens or closes,
 * taking over from the select tool's on the same box: its outline parts into
 * the crop's ants, its shade fades in, and the gizmo of a layer the select
 * tool turns fades out, each as far as the crop is open (`cropOpen`). The
 * shade lies only over as much of the picture as has opened round the crop,
 * from the crop's window out to the whole picture: laid over the whole
 * picture from the start, it darkened where the picture had not reached yet,
 * which read as the whole picture showing at once with the opening growing
 * inside it. Open, or shown flat, the crop carries no turn: its window
 * stands upright, with no gizmo.
 */
export const openingCropChrome = <
  Selection extends {
    rect: Box;
    cropMode?: boolean;
    cropOpen?: number;
    image?: Box;
    rotation?: SceneRotation;
  },
>(
  selection: Selection,
  { isFlat, open }: Pick<CropTurn, "isFlat" | "open">,
): Selection => {
  if (selection.cropMode !== true) return selection;
  if (isFlat || open >= 1) return { ...selection, rotation: undefined };
  const { image, rect } = selection;
  const toward = (from: number, to: number) => from + (to - from) * open;
  return {
    ...selection,
    cropOpen: open,
    image: image && {
      height: toward(rect.height, image.height),
      width: toward(rect.width, image.width),
      x: toward(rect.x, image.x),
      y: toward(rect.y, image.y),
    },
  };
};
