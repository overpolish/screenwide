// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type CropAspect } from "../../island/crop-aspect-choice";
import { drawnCropRect } from "../../screenshot/screenshot-crop";
import { sizeOfAspect } from "../../tool-panels/crop/crop-aspect-store";

import { MIN_BOX_SHARE, movedSceneBox } from "./recording-scene-custom";
import { framingWithin, placedPicture } from "./recording-scene-framing";
import { SceneRect } from "./recording-scene-geometry";
import {
  MAX_SCENE_ZOOM,
  SceneBox,
  SceneFraming,
  WHOLE_FRAMING,
} from "./recording-scenes";

/**
 * Cropping a custom scene's pane, which works the way the crop tool works on
 * the recording's own composition: the pane's picture stays where it sits on
 * the canvas and the crop window is the pane's box, moved and resized freely
 * over it. The screen of a preset's scene is made custom first, so it crops
 * the same way; a preset's camera is reframed instead, see `reframeWindow`.
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

type Canvas = { height: number; width: number };

/** `rect`, in output pixels, as a box: shares of `canvas` held to the limits
 * every box keeps, and that box back in output pixels. */
const heldBox = (rect: SceneRect, canvas: Canvas) => {
  const box = movedSceneBox(
    {
      height: rect.height / Math.max(1, canvas.height),
      width: rect.width / Math.max(1, canvas.width),
      x: rect.x / Math.max(1, canvas.width),
      y: rect.y / Math.max(1, canvas.height),
    },
    { delta: { x: 0, y: 0 }, scale: 1 },
  );
  return {
    box,
    held: {
      height: box.height * canvas.height,
      width: box.width * canvas.width,
      x: box.x * canvas.width,
      y: box.y * canvas.height,
    },
  };
};

/**
 * `rect` grown, where it is too small, to the smallest window a box can show
 * with its picture left where it sits: no side under `MIN_BOX_SHARE` of the
 * canvas, and wide enough that the picture is not zoomed past
 * `MAX_SCENE_ZOOM`. A smaller window would need the picture scaled to fit
 * it, so the crop stops there instead. It grows from the edges `moving`
 * names (bits 1 left, 2 right, 4 top, 8 bottom, as the gesture's), or about
 * its middle where none moves; for the zoom, across where a side edge moves
 * or none does, else down.
 */
const heldToLimits = (
  rect: SceneRect,
  {
    aspect,
    canvas,
    moving,
    picture,
  }: { aspect: number; canvas: Canvas; moving: number; picture: SceneRect },
): SceneRect => {
  type Span = { length: number; start: number };
  const grown = (span: Span, target: number, [low, high]: number[]): Span => {
    if (span.length >= target) return span;
    const grow = target - span.length;
    const start =
      (moving & low) !== 0
        ? span.start - grow
        : (moving & high) !== 0
          ? span.start
          : span.start - grow / 2;
    return { length: target, start };
  };
  let across = grown(
    { length: rect.width, start: rect.x },
    MIN_BOX_SHARE * canvas.width,
    [1, 2],
  );
  let down = grown(
    { length: rect.height, start: rect.y },
    MIN_BOX_SHARE * canvas.height,
    [4, 8],
  );
  const minimum = picture.width / MAX_SCENE_ZOOM;
  if (Math.max(across.length, down.length * aspect) < minimum) {
    if ((moving & 3) !== 0 || (moving & 12) === 0)
      across = grown(across, minimum, [1, 2]);
    else down = grown(down, minimum / aspect, [4, 8]);
  }
  return {
    height: down.length,
    width: across.length,
    x: across.start,
    y: down.start,
  };
};

/**
 * `pane` cropped to the window `next`, on a canvas of output pixels: the box,
 * as shares of the canvas, held inside the picture, to the limits every box
 * keeps and to the furthest a scene zooms (`heldToLimits`, `next`'s `moving`
 * edges growing it), and the framing that leaves the picture where it was. A
 * window that `slides`, one carried rather than reshaped, stops at the
 * picture's edge instead of being cut by it.
 */
export const croppedCustomPane = (
  pane: CustomPane,
  {
    canvas,
    moving,
    next,
    slides,
  }: {
    canvas: Canvas;
    moving: number;
    next: SceneRect;
    slides: boolean;
  },
): { box: SceneBox; framing: SceneFraming } => {
  const picture = pictureRect(pane);
  const cut = within(
    heldToLimits(within(next, { bounds: picture, slides }), {
      aspect: pane.aspect,
      canvas,
      moving,
      picture,
    }),
    { bounds: picture, slides: true },
  );
  const { box, held } = heldBox(cut, canvas);
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
 * `pane` cropped to `aspect`'s shape, as the Screen island's shapes crop: the
 * largest window of that shape inside the box, about its middle, the picture
 * staying where it sits. `original` is the picture's own shape. Null for a
 * free crop, which keeps any shape, or where the box has that shape already.
 */
export const shapedCustomPane = (
  pane: CustomPane,
  { aspect, canvas }: { aspect: CropAspect; canvas: Canvas },
): { box: SceneBox; framing: SceneFraming } | null => {
  const { box } = pane;
  const size = sizeOfAspect(aspect, box, { height: 1, width: pane.aspect });
  if (
    !size ||
    (Math.abs(size.width - box.width) < 1 &&
      Math.abs(size.height - box.height) < 1)
  )
    return null;
  return croppedCustomPane(pane, {
    canvas,
    moving: 0,
    next: {
      ...size,
      x: box.x + (box.width - size.width) / 2,
      y: box.y + (box.height - size.height) / 2,
    },
    slides: true,
  });
};

/**
 * `pane` with its crop taken away: the whole picture, unzoomed, in the
 * smallest box of the picture's shape that holds the one it had, about the
 * same middle, so the crop's window grows back out; one too big for the
 * canvas shrinks to fit it, and the box is slid back onto the canvas.
 */
export const uncroppedCustomPane = (
  pane: CustomPane,
  { canvas }: { canvas: Canvas },
): { box: SceneBox; framing: SceneFraming } => {
  const { aspect, box } = pane;
  const holding = Math.max(box.width, box.height * aspect);
  const width = Math.min(holding, canvas.width, canvas.height * aspect);
  const height = width / aspect;
  const onCanvas = (start: number, length: number, room: number) =>
    Math.min(room - length, Math.max(0, start));
  const { box: whole, held } = heldBox(
    {
      height,
      width,
      x: onCanvas(box.x + (box.width - width) / 2, width, canvas.width),
      y: onCanvas(box.y + (box.height - height) / 2, height, canvas.height),
    },
    canvas,
  );
  return { box: whole, framing: framingWithin(WHOLE_FRAMING, held, aspect) };
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
