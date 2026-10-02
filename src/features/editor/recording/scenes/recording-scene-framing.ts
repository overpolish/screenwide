// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CameraOverlaySettings } from "../../types";

import { SceneRect } from "./recording-scene-geometry";
import { MAX_SCENE_ZOOM, SceneFraming } from "./recording-scenes";

/**
 * What part of a picture fills its box in a scene. A framing never uncovers
 * its box: at any zoom the picture is held where it still fills it. The twin
 * of `src-tauri/src/editor/scenes/framing.rs`.
 */

/** The width a picture of `aspect` needs to cover `frame` exactly. */
const cover = (frame: SceneRect, aspect: number) =>
  Math.max(frame.width, frame.height * aspect);

/** The furthest a focus may sit from the picture's middle, as a share of it,
 * where a picture `size` long still covers `length` of box. */
const reach = (size: number, length: number) =>
  Math.max(0, size - length) / 2 / Math.max(size, Number.EPSILON);

const clamped = (value: number, low: number, high: number) =>
  Math.min(high, Math.max(low, value));

/** `framing` held where a picture of `aspect` still covers `frame`. */
export const framingWithin = (
  framing: SceneFraming,
  frame: SceneRect,
  aspect: number,
): SceneFraming => {
  const zoom = clamped(framing.zoom, 1, MAX_SCENE_ZOOM);
  const width = cover(frame, aspect) * zoom;
  const reachX = reach(width, frame.width);
  const reachY = reach(width / aspect, frame.height);
  return {
    focusX: clamped(framing.focusX, 0.5 - reachX, 0.5 + reachX),
    focusY: clamped(framing.focusY, 0.5 - reachY, 0.5 + reachY),
    zoom,
  };
};

/** The centre and width of a picture of `aspect` that frames `frame` the way
 * `framing` says. */
export const placedPicture = (
  framing: SceneFraming,
  frame: SceneRect,
  aspect: number,
) => {
  const held = framingWithin(framing, frame, aspect);
  const width = cover(frame, aspect) * held.zoom;
  return {
    width,
    x: frame.x + frame.width / 2 - (held.focusX - 0.5) * width,
    y: frame.y + frame.height / 2 - ((held.focusY - 0.5) * width) / aspect,
  };
};

/**
 * The camera's box in the recording's own composition as it is drawn, for a
 * camera picture of `aspect`: no larger than the picture and held inside it,
 * the way `bake_geometry` places it, since a stored box can sit partly
 * outside the picture. The twin of `drawn_frame` in `framing.rs`.
 */
export const drawnCameraFrame = (
  overlay: CameraOverlaySettings,
  aspect: number,
): SceneRect => {
  const cameraWidth = overlay.cameraWidth;
  const cameraHeight = cameraWidth / Math.max(aspect, Number.EPSILON);
  const left = overlay.cameraX - cameraWidth / 2;
  const top = overlay.cameraY - cameraHeight / 2;
  const width = Math.min(overlay.frameWidth, cameraWidth);
  const height = Math.min(overlay.frameHeight, cameraHeight);
  return {
    height,
    width,
    x: clamped(
      overlay.frameX,
      left,
      Math.max(left, left + cameraWidth - width),
    ),
    y: clamped(overlay.frameY, top, Math.max(top, top + cameraHeight - height)),
  };
};

/** The framing `overlay` gives a camera picture of `aspect` in the
 * recording's own composition, outside every scene. */
export const cameraFramingOf = (
  overlay: CameraOverlaySettings,
  aspect: number,
): SceneFraming => {
  const frame = drawnCameraFrame(overlay, aspect);
  const width = Math.max(overlay.cameraWidth, Number.EPSILON);
  const height = width / aspect;
  return {
    focusX: (frame.x + frame.width / 2 - (overlay.cameraX - width / 2)) / width,
    focusY:
      (frame.y + frame.height / 2 - (overlay.cameraY - height / 2)) / height,
    zoom: Math.max(1, width / Math.max(cover(frame, aspect), Number.EPSILON)),
  };
};

/**
 * `framing` with its picture, of `aspect`, dragged `delta` canvas pixels and
 * zoomed by `scale` in `frame`. The focus is a share of the picture, so it
 * moves by the pointer's share of the picture as it is drawn there.
 */
export const reframed = ({
  aspect,
  delta,
  frame,
  framing,
  scale,
}: {
  aspect: number;
  delta: { x: number; y: number };
  frame: SceneRect;
  framing: SceneFraming;
  scale: number;
}): SceneFraming => {
  const start = framingWithin(framing, frame, aspect);
  const width = cover(frame, aspect) * start.zoom;
  return framingWithin(
    {
      focusX: start.focusX - delta.x / width,
      focusY: start.focusY - (delta.y * aspect) / width,
      zoom: start.zoom * scale,
    },
    frame,
    aspect,
  );
};

/** The whole picture of `aspect` unzoomed, covering `box` from its middle:
 * what the crop tool shows while it reframes the box. */
const unzoomedPicture = (box: SceneRect, aspect: number): SceneRect => {
  const width = cover(box, aspect);
  const height = width / aspect;
  return {
    height,
    width,
    x: box.x + box.width / 2 - width / 2,
    y: box.y + box.height / 2 - height / 2,
  };
};

/**
 * The crop window the crop tool lays over a picture of `aspect` while it
 * reframes `box`: the whole picture unzoomed, and the part of it `framing`
 * shows, which keeps the box's shape and is smaller the further it zooms.
 * Both are in the box's own units.
 */
export const reframeWindow = ({
  aspect,
  box,
  framing,
}: {
  aspect: number;
  box: SceneRect;
  framing: SceneFraming;
}) => {
  const image = unzoomedPicture(box, aspect);
  const held = framingWithin(framing, box, aspect);
  const width = box.width / held.zoom;
  const height = box.height / held.zoom;
  return {
    image,
    window: {
      height,
      width,
      x: image.x + held.focusX * image.width - width / 2,
      y: image.y + held.focusY * image.height - height / 2,
    },
  };
};

/** The framing a crop `window` laid over the unzoomed picture of `aspect`
 * picks for `box`: the inverse of `reframeWindow`. */
export const framingFromWindow = ({
  aspect,
  box,
  window,
}: {
  aspect: number;
  box: SceneRect;
  window: SceneRect;
}) => {
  const image = unzoomedPicture(box, aspect);
  return framingWithin(
    {
      focusX: (window.x + window.width / 2 - image.x) / image.width,
      focusY: (window.y + window.height / 2 - image.y) / image.height,
      zoom: box.width / Math.max(window.width, Number.EPSILON),
    },
    box,
    aspect,
  );
};
