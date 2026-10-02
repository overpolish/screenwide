// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { drawnCropRect } from "../../screenshot/screenshot-crop";

import { movedSceneBox } from "./recording-scene-custom";
import { framingWithin, placedPicture } from "./recording-scene-framing";
import { SceneRect } from "./recording-scene-geometry";
import { SceneBox, SceneFraming } from "./recording-scenes";

/**
 * Cropping a custom scene's pane, which works the way the crop tool works on
 * the recording's own composition: the pane's picture stays where it sits on
 * the canvas and the crop window is the pane's box, moved and resized freely
 * over it. A preset's pane is reframed instead, see `reframeWindow`.
 */

/** One pane of a custom scene: its picture's aspect, its box in output
 * pixels, and the framing that places its picture there. */
type CustomPane = { aspect: number; box: SceneRect; framing: SceneFraming };

/** Where `pane`'s picture sits, in output pixels. */
export const pictureRect = ({ aspect, box, framing }: CustomPane) => {
  const picture = placedPicture(framing, box, aspect);
  const height = picture.width / aspect;
  return {
    height,
    width: picture.width,
    x: picture.x - picture.width / 2,
    y: picture.y - height / 2,
  };
};

/** `rect` cut to what of it lies inside `bounds`, slid inside them first
 * where it `slides` and fits, so a carried window stops at the edge while one
 * pulled past it is cut there. */
const within = (
  rect: SceneRect,
  { bounds, slides }: { bounds: SceneRect; slides: boolean },
): SceneRect => {
  const slide = (start: number, length: number, [low, high]: number[]) =>
    slides && length <= high - low
      ? Math.min(high - length, Math.max(low, start))
      : start;
  const x = slide(rect.x, rect.width, [bounds.x, bounds.x + bounds.width]);
  const y = slide(rect.y, rect.height, [bounds.y, bounds.y + bounds.height]);
  const left = Math.max(x, bounds.x);
  const top = Math.max(y, bounds.y);
  const right = Math.min(x + rect.width, bounds.x + bounds.width);
  const bottom = Math.min(y + rect.height, bounds.y + bounds.height);
  return {
    height: Math.max(0, bottom - top),
    width: Math.max(0, right - left),
    x: left,
    y: top,
  };
};

/**
 * `pane` cropped to the window `next`, on a canvas of output pixels: the box,
 * as shares of the canvas, held inside the picture and to the limits every
 * box keeps, and the framing that leaves the picture where it was. A window
 * that `slides`, one carried rather than reshaped, stops at the picture's
 * edge instead of being cut by it.
 */
export const croppedCustomPane = (
  pane: CustomPane,
  {
    canvas,
    next,
    slides,
  }: {
    canvas: { height: number; width: number };
    next: SceneRect;
    slides: boolean;
  },
): { box: SceneBox; framing: SceneFraming } => {
  const picture = pictureRect(pane);
  const cut = within(next, { bounds: picture, slides });
  const box = movedSceneBox(
    {
      height: cut.height / Math.max(1, canvas.height),
      width: cut.width / Math.max(1, canvas.width),
      x: cut.x / Math.max(1, canvas.width),
      y: cut.y / Math.max(1, canvas.height),
    },
    { delta: { x: 0, y: 0 }, scale: 1 },
  );
  const held = {
    height: box.height * canvas.height,
    width: box.width * canvas.width,
    x: box.x * canvas.width,
    y: box.y * canvas.height,
  };
  const cover = Math.max(held.width, held.height * pane.aspect);
  return {
    box,
    framing: framingWithin(
      {
        focusX: (held.x + held.width / 2 - picture.x) / picture.width,
        focusY: (held.y + held.height / 2 - picture.y) / picture.height,
        zoom: picture.width / Math.max(cover, Number.EPSILON),
      },
      held,
      pane.aspect,
    ),
  };
};

/**
 * The crop window a crop gesture leaves on `pane`, in output pixels, from the
 * box as the gesture found it. `delta` is the gesture's travel in output
 * pixels; a move carries the window, a resize carries the edges `edges`
 * names, and a draw spans from `anchor`, a point in output pixels.
 */
export const customCropWindow = (
  box: SceneRect,
  {
    anchor,
    delta,
    edges,
    operation,
  }: {
    anchor: { x: number; y: number };
    delta: { x: number; y: number };
    edges: number;
    operation: "cropDraw" | "cropMove" | "cropResize";
  },
): SceneRect => {
  if (operation === "cropMove")
    return { ...box, x: box.x + delta.x, y: box.y + delta.y };
  if (operation === "cropDraw")
    return drawnCropRect(anchor, {
      deltaX: delta.x,
      deltaY: delta.y,
      edges,
    });
  const left = box.x + ((edges & 1) !== 0 ? delta.x : 0);
  const right = box.x + box.width + ((edges & 2) !== 0 ? delta.x : 0);
  const top = box.y + ((edges & 4) !== 0 ? delta.y : 0);
  const bottom = box.y + box.height + ((edges & 8) !== 0 ? delta.y : 0);
  return { height: bottom - top, width: right - left, x: left, y: top };
};
