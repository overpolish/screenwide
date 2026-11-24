// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  DEFAULT_BUBBLE_SIZE,
  DEFAULT_CAMERA_SIZE,
  DEFAULT_CORNER,
  type SceneVariant,
} from "./recording-scene-variant";

import type { SceneRect } from "./recording-scene-geometry";
import type { ClearArea } from "../../../../bindings/ClearArea";

// How the presets that pair the screen and the camera lay them out on a
// canvas taller than it is wide, as a short video is. The screen comes
// first: it is shown whole at its own shape, as wide as the margins allow,
// and inside the part of the canvas the platform's interface leaves clear.
// The camera is the pane that gives way. The twin of `tall.rs` in
// `src-tauri/crates/scenes`.

type Canvas = { height: number; width: number };

/** How a pair is laid out, and what the platform covers. */
type TallLayout = { variant: SceneVariant; clear?: ClearArea };

/** The camera's length in a pair over the screen's, at the least. */
const CAMERA_RATIO = { third: 0.5, "two-thirds": 2 } as const;

/** A picture in picture's camera's side, as a share of the screen's
 * width. */
const BUBBLE_SHARE = { large: 0.5, small: 0.34 } as const;

/** How much of the camera hangs past the screen's edge, as a share of its
 * side. */
const BUBBLE_HANG = 0.75;

const NO_CLEAR_AREA: ClearArea = { bottom: 0, side: 0, top: 0 };

/** The top and bottom of the clear part of `canvas`, kept at least
 * `margin` from its own edges. */
const clearRows = (canvas: Canvas, clear: ClearArea, margin: number) => ({
  bottom: Math.min(canvas.height - margin, canvas.height * (1 - clear.bottom)),
  top: Math.max(margin, clear.top * canvas.height),
});

/** The margin kept at the canvas's sides, at least the clear area's. */
const sideMargin = (canvas: Canvas, clear: ClearArea) =>
  Math.max(canvas.width / 18, clear.side * canvas.width);

/** Side by side or stacked: the screen whole and as wide as the margins
 * allow, the camera directly under it (over it where swapped) at the same
 * width, taking the rest of the clear area and at least its share of the
 * pair. A camera two thirds of the pair keeps them by making the screen
 * smaller, and may run on under the platform's bar on its side. */
export function tallPair(
  canvas: Canvas,
  screenAspect: number,
  { clear = NO_CLEAR_AREA, variant }: TallLayout,
): { camera: SceneRect; screen: SceneRect } {
  const margin = canvas.width / 18;
  const gap = margin / 2;
  const ratio = CAMERA_RATIO[variant.cameraSize ?? DEFAULT_CAMERA_SIZE];
  const swap = variant.swap ?? false;
  const rows = clearRows(canvas, clear, margin);
  const clearHeight = rows.bottom - rows.top;
  const reach = swap ? rows.bottom - margin : canvas.height - margin - rows.top;
  const height = Math.min(
    (canvas.width - sideMargin(canvas, clear) * 2) / screenAspect,
    clearHeight,
    (reach - gap) / (1 + ratio),
  );
  const width = height * screenAspect;
  const cameraHeight = Math.max(height * ratio, clearHeight - height - gap);
  const total = height + gap + cameraHeight;
  const top =
    total <= clearHeight
      ? rows.top + (clearHeight - total) / 2
      : swap
        ? rows.bottom - total
        : rows.top;
  const x = (canvas.width - width) / 2;
  return {
    camera: {
      height: cameraHeight,
      width,
      x,
      y: swap ? top : top + height + gap,
    },
    screen: { height, width, x, y: swap ? top + cameraHeight + gap : top },
  };
}

/** Picture in picture: the screen whole and as wide as the margins allow,
 * the round camera hanging below it for a bottom corner and above it for a
 * top one, on its corner's side, just overlapping the screen's edge. The
 * pair sits in the middle of the clear area. */
export function tallBubble(
  canvas: Canvas,
  screenAspect: number,
  { clear = NO_CLEAR_AREA, variant }: TallLayout,
): { camera: SceneRect; screen: SceneRect } {
  const rows = clearRows(canvas, clear, canvas.width / 18);
  const clearHeight = rows.bottom - rows.top;
  const share = BUBBLE_SHARE[variant.size ?? DEFAULT_BUBBLE_SIZE];
  const width = Math.min(
    canvas.width - sideMargin(canvas, clear) * 2,
    clearHeight / (1 / screenAspect + share * BUBBLE_HANG),
  );
  const height = width / screenAspect;
  const side = width * share;
  const top = rows.top + (clearHeight - height - side * BUBBLE_HANG) / 2;
  const x = (canvas.width - width) / 2;
  const corner = variant.corner ?? DEFAULT_CORNER;
  const below = corner.startsWith("bottom");
  const right = corner.endsWith("right");
  const screenY = below ? top : top + side * BUBBLE_HANG;
  return {
    camera: {
      height: side,
      width: side,
      x: right ? x + width - side * 1.1 : x + side * 0.1,
      y: below ? screenY + height - side * (1 - BUBBLE_HANG) : top,
    },
    screen: { height, width, x, y: screenY },
  };
}
