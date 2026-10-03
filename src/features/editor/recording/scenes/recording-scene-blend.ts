// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SceneRect } from "./recording-scene-geometry";
import { Picture, Placement } from "./recording-scene-placement";

/** How small a pane that appears or vanishes is drawn at the far end of its
 * fade, as a share of its own size. */
const VANISHED_SCALE = 0.25;

const mixed = (a: number, b: number, t: number) => a + (b - a) * t;

const mixedRect = (a: SceneRect, b: SceneRect, t: number): SceneRect => ({
  height: mixed(a.height, b.height, t),
  width: mixed(a.width, b.width, t),
  x: mixed(a.x, b.x, t),
  y: mixed(a.y, b.y, t),
});

const mixedPicture = (a: Picture, b: Picture, t: number): Picture => ({
  width: mixed(a.width, b.width, t),
  x: mixed(a.x, b.x, t),
  y: mixed(a.y, b.y, t),
});

/** For a pane whose opacity goes from `from` to `to`, whether it vanishes,
 * drawn at the start's place, or appears, drawn at the end's, and how large
 * it is `t` of the way through; null for a pane shown, or hidden,
 * throughout. */
const settling = (from: number, to: number, t: number) =>
  from > 0 && to === 0
    ? { factor: 1 + (VANISHED_SCALE - 1) * t, vanishes: true }
    : from === 0 && to > 0
      ? { factor: VANISHED_SCALE + (1 - VANISHED_SCALE) * t, vanishes: false }
      : null;

/** `box` scaled by `factor` about its middle, and `picture` with it. */
const shrunk = (box: SceneRect, picture: Picture, factor: number) => {
  const centre = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
  return {
    box: {
      height: box.height * factor,
      width: box.width * factor,
      x: centre.x - (box.width * factor) / 2,
      y: centre.y - (box.height * factor) / 2,
    },
    // The screen's image is placed by its corner and the camera's picture
    // by its middle; either is carried towards the box's middle alike.
    picture: {
      width: picture.width * factor,
      x: centre.x + (picture.x - centre.x) * factor,
      y: centre.y + (picture.y - centre.y) * factor,
    },
  };
};

/**
 * The placement `t` of the way from `from` to `to`. A pane present in both
 * moves and resizes between them. A pane one of them hides has nowhere to
 * move to, so it stays at its place in the other, shrinking to a quarter of
 * its size about its middle as it fades out, or growing from there as it
 * fades in. The twin of `Placement::toward` in
 * `src-tauri/src/editor/scenes/placement.rs`.
 */
export const towardPlacement = (
  from: Placement,
  to: Placement,
  t: number,
): Placement => {
  const next: Placement = {
    camera: mixedPicture(from.camera, to.camera, t),
    cameraFront: mixed(from.cameraFront, to.cameraFront, t),
    frame: mixedRect(from.frame, to.frame, t),
    image: mixedPicture(from.image, to.image, t),
    opacity: {
      camera: mixed(from.opacity.camera, to.opacity.camera, t),
      screen: mixed(from.opacity.screen, to.opacity.screen, t),
    },
    radius: {
      camera: mixed(from.radius.camera, to.radius.camera, t),
      screen: mixed(from.radius.screen, to.radius.screen, t),
    },
    screen: mixedRect(from.screen, to.screen, t),
  };
  const screen = settling(from.opacity.screen, to.opacity.screen, t);
  if (screen) {
    const source = screen.vanishes ? from : to;
    const held = shrunk(source.screen, source.image, screen.factor);
    next.screen = held.box;
    next.image = held.picture;
  }
  const camera = settling(from.opacity.camera, to.opacity.camera, t);
  if (camera) {
    const source = camera.vanishes ? from : to;
    const held = shrunk(source.frame, source.camera, camera.factor);
    next.frame = held.box;
    next.camera = held.picture;
  }
  return next;
};
