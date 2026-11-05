// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ImagePlayChange } from "./annotation-channel";
import { rememberAnnotationImage } from "./annotation-defaults";
import { freshSeed, swayedShape } from "./annotation-shape-edits";
import { Annotation, AnnotationShape, ImageArt } from "./annotations";

/**
 * The chosen image's picture, and the edits its island makes to it, each
 * given to `reshape` as the image's next shape: another picture, how a
 * moving one plays, and whether it sways. A no-op on anything but an image.
 */
export const imageEdits = (
  selected: Annotation | null,
  reshape: (shape: AnnotationShape | null) => void,
) => {
  // An image given another picture keeps its place, size and turn, and
  // takes the new picture's proportions and, where it moves, how it plays;
  // the next image shows it too.
  const image = selected?.shape.kind === "image" ? selected.shape : null;
  const art: ImageArt | null = image
    ? { aspect: image.aspect, asset: image.asset, play: image.play }
    : null;
  const applyImage = (next: ImageArt) => {
    if (!selected || selected.shape.kind !== "image") return;
    rememberAnnotationImage(next);
    const shape = selected.shape;
    if (shape.asset === next.asset && shape.aspect === next.aspect) return;
    // A placed image keeps its size: only the picture and its proportions
    // change, whatever size the new picture is of its own.
    const { aspect, asset, play } = next;
    reshape({ ...shape, aspect, asset, play });
  };
  // A moving image's frame, which a still shows and playback starts on,
  // and whether a recording plays it through once.
  const applyImagePlay = (next: ImagePlayChange) => {
    if (!selected || selected.shape.kind !== "image" || !selected.shape.play)
      return;
    reshape({ ...selected.shape, play: { ...selected.shape.play, ...next } });
  };
  // Whether a recording gives the chosen image its slight turn and drift.
  // Turning it on draws a fresh seed, as the dice does.
  const applySway = (sway: boolean) => {
    if (selected)
      reshape(swayedShape(selected.shape, sway ? freshSeed() : null));
  };
  return { applyImage, applyImagePlay, applySway, art };
};
