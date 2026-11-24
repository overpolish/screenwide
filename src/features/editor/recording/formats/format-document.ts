// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { canvasScale } from "../../screenshot/screenshot-canvas-scale";

import { laidScreen, screenIntent } from "./platform-layout";

import type { ClearArea } from "../../../../bindings/ClearArea";
import type { SceneBoxes } from "../../../../bindings/SceneBoxes";
import type {
  RecordingOutputSettings,
  ScreenshotOutputSettings,
} from "../../screenshot/screenshot-output";
import type { CameraOverlaySettings } from "../../types";

/** What each of a recording's formats keeps of its own canvas, as the
 * editor holds it for the format in hand. */
export type FormatDocument = {
  cameraOverlay: CameraOverlaySettings;
  recordingOutput: RecordingOutputSettings;
};

type Size = { height: number; width: number };

/** A custom scene's boxes on a canvas reshaped from `from` to `to`: each
 * keeps its middle's share of the canvas and its own shape, and is sized as
 * what is on a canvas is when the canvas is resized (`canvasScale`). */
export const reshapedBoxes = (
  boxes: SceneBoxes,
  from: Size,
  to: Size,
): SceneBoxes => {
  const scale = canvasScale(from, to);
  const reshaped = (box: SceneBoxes["screen"]) => {
    const width = (box.width * from.width * scale.size) / to.width;
    const height = (box.height * from.height * scale.size) / to.height;
    const middle = { x: box.x + box.width / 2, y: box.y + box.height / 2 };
    return {
      height,
      width,
      x: middle.x - width / 2,
      y: middle.y - height / 2,
    };
  };
  return {
    ...boxes,
    ...(boxes.camera ? { camera: reshaped(boxes.camera) } : {}),
    screen: reshaped(boxes.screen),
  };
};

/** How a point on a canvas moves to another: `offset + p * scale`. */
type Mapping = { scale: number; x: number; y: number };

/** How a canvas `from` big is laid into one `to` big: scaled by the lesser
 * of the two axes' scales, so all of it shows, and centred. */
const fitInside = (from: Size, to: Size): Mapping => {
  const scale = Math.min(
    to.width / Math.max(1, from.width),
    to.height / Math.max(1, from.height),
  );
  return {
    scale,
    x: (to.width - from.width * scale) / 2,
    y: (to.height - from.height * scale) / 2,
  };
};

/** How what is on the canvas of `settings` moves with its screen, laid out
 * on a canvas `to` big whose platform covers `clear` (`laidScreen`). A
 * canvas of the same proportions, kept clear of the same, is scaled whole
 * (`fitInside`), as is a placement with no screen to read. */
const screenMapping = (
  settings: ScreenshotOutputSettings,
  to: Size,
  clear: ClearArea | undefined,
): Mapping => {
  const box = {
    height: settings.cropHeight,
    width: settings.cropWidth,
    x: settings.cropX,
    y: settings.cropY,
  };
  const isScaled =
    Math.abs(settings.width * to.height - to.width * settings.height) <
      Math.max(settings.width, settings.height) &&
    JSON.stringify(settings.clearArea) === JSON.stringify(clear);
  if (isScaled || box.width <= 0 || box.height <= 0 || settings.width <= 0)
    return fitInside(settings, to);
  const laid = laidScreen(to, screenIntent(settings, box), clear);
  const scale = laid.width / box.width;
  return { scale, x: laid.x - box.x * scale, y: laid.y - box.y * scale };
};

/** The screen's placement on its canvas, laid out on a canvas `to` big
 * whose platform covers `clear` (`platform-layout.ts`), the clear area kept
 * with it for the scenes to lay their panes out by. */
export const laidPlacement = (
  settings: ScreenshotOutputSettings,
  to: Size,
  clear: ClearArea | undefined,
): ScreenshotOutputSettings => {
  const map = screenMapping(settings, to, clear);
  const { clearArea: _was, ...rest } = settings;
  return {
    ...rest,
    ...(clear ? { clearArea: clear } : {}),
    cropHeight: settings.cropHeight * map.scale,
    cropWidth: settings.cropWidth * map.scale,
    cropX: map.x + settings.cropX * map.scale,
    cropY: map.y + settings.cropY * map.scale,
    height: Math.max(1, Math.round(to.height)),
    imageWidth: settings.imageWidth * map.scale,
    imageX: map.x + settings.imageX * map.scale,
    imageY: map.y + settings.imageY * map.scale,
    width: Math.max(1, Math.round(to.width)),
  };
};

/** Where a span `length` long starting at `start` is held inside `from` to
 * `end`: where it is if it fits there, else moved in, or centred where it
 * is longer. */
const heldInside = (
  { length, start }: { length: number; start: number },
  { end, from }: { end: number; from: number },
) =>
  length > end - from
    ? from + (end - from - length) / 2
    : Math.min(Math.max(start, from), end - length);

/** A camera placed on the canvas of `screen`, moved with the screen as it
 * is laid out on a canvas `size` big whose platform covers `clear`, then
 * kept inside the part the platform leaves clear where it fits. */
export const laidCamera = (
  overlay: CameraOverlaySettings,
  screen: ScreenshotOutputSettings,
  { clear, size }: { clear: ClearArea | undefined; size: Size },
): CameraOverlaySettings => {
  const map = screenMapping(screen, size, clear);
  const frame = {
    height: overlay.frameHeight * map.scale,
    width: overlay.frameWidth * map.scale,
    x: map.x + overlay.frameX * map.scale,
    y: map.y + overlay.frameY * map.scale,
  };
  const side = size.width * (clear?.side ?? 0);
  const shift = {
    x:
      heldInside(
        { length: frame.width, start: frame.x },
        { end: size.width - side, from: side },
      ) - frame.x,
    y:
      heldInside(
        { length: frame.height, start: frame.y },
        {
          end: size.height * (1 - (clear?.bottom ?? 0)),
          from: size.height * (clear?.top ?? 0),
        },
      ) - frame.y,
  };
  return {
    ...overlay,
    cameraWidth: overlay.cameraWidth * map.scale,
    cameraX: map.x + overlay.cameraX * map.scale + shift.x,
    cameraY: map.y + overlay.cameraY * map.scale + shift.y,
    frameHeight: frame.height,
    frameWidth: frame.width,
    frameX: frame.x + shift.x,
    frameY: frame.y + shift.y,
  };
};

/** `document` on a canvas `size` whose platform covers `clear`: the screen
 * laid out on it from the canvas it came from (`laidPlacement`), and the
 * camera moved with it (`laidCamera`). The camera's placement is the screen
 * canvas's, whether or not it is drawn into it. */
export const laidDocument = (
  document: FormatDocument,
  size: Size,
  clear: ClearArea | undefined,
): FormatDocument => {
  const from = document.recordingOutput.primary;
  return {
    cameraOverlay: laidCamera(document.cameraOverlay, from, { clear, size }),
    recordingOutput: {
      ...document.recordingOutput,
      primary: laidPlacement(from, size, clear),
    },
  };
};
