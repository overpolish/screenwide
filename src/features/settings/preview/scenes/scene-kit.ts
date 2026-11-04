// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SAMPLE } from "./sample-palette";

import type { PointerScript } from "../../../../bindings/PointerScript";
import type { PreviewScene } from "../../../../bindings/PreviewScene";
import type { IslandNotch } from "../../../../primitives/island/island-geometry";
import type { PictureSize } from "../settings-preview-api";

// What every preview's picture is made of: the desktop the page paints, in
// points at the size it would be on a display, set on the person's own
// desktop picture where the app can read it, and the pointer that works
// over it. What is drawn on it (a tool, a recording) is the app's.

/** Where the pointer is and whether it is pressed, or `null` while it is
 * away. */
export type Pointer = { pressed: boolean; x: number; y: number } | null;

/** Something a tool's scene does at a moment other than moving the
 * pointer: Annotate's tool picked up, or its session ending and the next
 * starting. */
type Cue = {
  at: number;
  input: "enter" | "exit" | "pickArrow" | "pickCounter";
};

/** What a scene's desktop is set on: the person's own desktop picture, and
 * the notch of the display Screenwide's island comes out on, which its
 * menu bar and island are drawn round. */
export type SceneDesktop = {
  notch: IslandNotch | null;
  wallpaper: HTMLImageElement | null;
};

export type Painter = (
  context: CanvasRenderingContext2D,
  size: PictureSize,
  desktop: SceneDesktop,
) => void;

/** A tool at work on a strip of desktop, driven by the page's pointer. */
export type ToolScene = {
  /** How long one pass of the pointer takes, in seconds. */
  duration: number;
  kind: "tool";
  paint: Painter;
  /** Where the pointer is `at` seconds in, on a picture `size` big, for
   * the setting's values in `scene`. */
  pointer: (at: number, size: PictureSize, scene: PreviewScene) => Pointer;
  cues?: readonly Cue[];
  /** The tool draws on glass over the painted desktop rather than drawing
   * the desktop itself. */
  glass?: boolean;
};

/** A recording of a desktop, shown whole in its box. */
export type RecordingScene = {
  kind: "recording";
  paint: Painter;
  size: PictureSize;
  /** What the setting leaves out of the recording, such as Screenwide's
   * windows or the desktop's icons, painted over the desktop for the scenes
   * that show it with and without it. */
  paintLayer?: Painter;
  script?: PointerScript;
};

export type Box = { height: number; width: number; x: number; y: number };

export type Spot = [number, number];

export const FONT = "Inter, system-ui, sans-serif";
/** The size the scenes set their type at. */
export const TYPE_SIZE = 13;

/** A tool's strip of desktop: wider than its box can get in a window,
 * which shows its middle, so a resize needs no repaint. */
export const TOOL_PICTURE: PictureSize = { height: 160, width: 1600 };

/** How big the strip is beside the desktop the tool works on, which is
 * twice its size, so what it draws sits with the strip's small windows as
 * it would with real ones. The twin of `TOOL_SCALE` in the app's
 * `settings_preview.rs`. */
export const TOOL_SCALE = 0.5;

/** A colour token's value, so what Screenwide draws on the desktop is in
 * the app's own colours; `scope` reads a subtree's, such as the island's
 * dark skin. */
export function token(name: string, scope: Element = document.documentElement) {
  return getComputedStyle(scope).getPropertyValue(`--color-${name}`).trim();
}

export const lerp = (from: number, to: number, amount: number) =>
  from + (to - from) * amount;

const ease = (amount: number) => amount * amount * (3 - 2 * amount);

/** How far into the part of a loop from `start` to `end` a time is, 0 to 1,
 * eased in and out. */
export const progress = (at: number, start: number, end: number) =>
  ease(Math.min(Math.max((at - start) / (end - start), 0), 1));

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

/** Blobs of the sample mesh: centre and reach as shares of the canvas. */
const MESH = [
  [0.15, 0.2, 0.75],
  [0.85, 0.1, 0.7],
  [0.7, 0.95, 0.8],
  [0.1, 0.95, 0.6],
] as const;

/** A dusk mesh standing in for a desktop picture where there is none, as
 * in Storybook, so the sample desktop never sits on a flat grey. */
function paintMesh(
  context: CanvasRenderingContext2D,
  { height, width }: PictureSize,
) {
  const [base, ...blobs] = SAMPLE.wallpaper;
  context.fillStyle = base;
  context.fillRect(0, 0, width, height);
  const reach = Math.max(width, height);
  MESH.forEach(([x, y, radius], index) => {
    const colour = blobs[index % blobs.length];
    const gradient = context.createRadialGradient(
      x * width,
      y * height,
      0,
      x * width,
      y * height,
      radius * reach,
    );
    gradient.addColorStop(0, colour);
    gradient.addColorStop(1, `${colour}00`);
    context.fillStyle = gradient;
    context.fillRect(0, 0, width, height);
  });
}

/** The desktop's picture covering the canvas, or the sample mesh where
 * there is none. */
export function paintWallpaper(
  context: CanvasRenderingContext2D,
  size: PictureSize,
  wallpaper: HTMLImageElement | null,
) {
  if (!wallpaper) {
    paintMesh(context, size);
    return;
  }
  const { height, width } = size;
  const source = {
    height: wallpaper.naturalHeight,
    width: wallpaper.naturalWidth,
  };
  const scale = Math.max(width / source.width, height / source.height);
  context.drawImage(
    wallpaper,
    (width - source.width * scale) / 2,
    (height - source.height * scale) / 2,
    source.width * scale,
    source.height * scale,
  );
}

/** Resolves once the scenes' type has loaded, so a painted desktop is set
 * in it rather than a fallback. */
export function sceneFonts() {
  return Promise.all([
    document.fonts.load(`400 ${String(TYPE_SIZE)}px Inter`),
    document.fonts.load(`600 ${String(TYPE_SIZE)}px Inter`),
  ]);
}
