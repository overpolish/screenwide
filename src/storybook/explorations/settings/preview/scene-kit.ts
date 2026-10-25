// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { lerp, progress } from "./loop-timing";

// What every tool's scene in Tools is made of: the screen the page paints
// and the pointer that works the tool over it, in points, at the size they
// would be on a display. The tool itself is the app's, drawn natively, so
// what it finds, measures, reads or draws is real.

/** Where the pointer is and whether it is pressed, or `null` while it is
 * away. */
type Pointer = { pressed: boolean; x: number; y: number } | null;

/** Something the scene does at a moment other than moving the pointer:
 * Annotate's tool picked up, or its session ending and the next starting. */
type Cue = {
  at: number;
  input: "enter" | "exit" | "pickArrow" | "pickCounter";
};

export type ToolScene = {
  /** How long one pass of the pointer takes, in seconds. */
  duration: number;
  paint: (
    context: CanvasRenderingContext2D,
    width: number,
    height: number,
  ) => void;
  pointer: (at: number, width: number, height: number) => Pointer;
  cues?: readonly Cue[];
  /** The tool draws on glass over the painted screen rather than drawing
   * the screen itself. */
  glass?: boolean;
};

export type Box = { height: number; width: number; x: number; y: number };

export type Spot = [number, number];

export const FONT = "Inter, system-ui, sans-serif";
export const INK = "#1d1d1f";
/** The size the scenes set their type at. */
export const TYPE_SIZE = 13;

/** The size every screen is painted at, in points: wider than the box can
 * get in a window, which shows its middle, so a resize needs no repaint. */
export const PICTURE = { height: 160, width: 1600 };

export function paintWallpaper(
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
) {
  const gradient = context.createLinearGradient(0, 0, width, height);
  gradient.addColorStop(0, "#f6a65b");
  gradient.addColorStop(1, "#c4456b");
  context.fillStyle = gradient;
  context.fillRect(0, 0, width, height);
}

export function paintCard(
  context: CanvasRenderingContext2D,
  box: Box,
  radius: number,
) {
  context.save();
  context.shadowColor = "rgb(0 0 0 / 0.18)";
  context.shadowBlur = 12;
  context.shadowOffsetY = 3;
  context.fillStyle = "#fff";
  context.beginPath();
  context.roundRect(box.x, box.y, box.width, box.height, radius);
  context.fill();
  context.restore();
}

/** A point travelling from `from` to `to` over the seconds `start` to
 * `end`. */
export function travel(
  at: number,
  [start, end]: Spot,
  [from, to]: [Spot, Spot],
) {
  const amount = progress(at, start, end);
  return { x: lerp(from[0], to[0], amount), y: lerp(from[1], to[1], amount) };
}

/** Paints `scene` on `canvas` at the picture's size, in device pixels. */
export function paintScene(canvas: HTMLCanvasElement, scene: ToolScene) {
  const { height, width } = PICTURE;
  const scale = window.devicePixelRatio;
  canvas.width = Math.round(width * scale);
  canvas.height = Math.round(height * scale);
  const context = canvas.getContext("2d");
  if (!context) return null;
  context.scale(scale, scale);
  scene.paint(context, width, height);
  return context;
}

/** Resolves once the scenes' type has loaded, so a painted screen is set in
 * it rather than a fallback. */
export function sceneFonts() {
  return Promise.all([
    document.fonts.load(`400 ${String(TYPE_SIZE)}px Inter`),
    document.fonts.load(`600 ${String(TYPE_SIZE)}px Inter`),
  ]);
}
