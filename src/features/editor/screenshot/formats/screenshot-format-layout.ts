// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  laidScreen,
  screenIntent,
} from "../../recording/formats/platform-layout";

import type { ClearArea } from "../../../../bindings/ClearArea";
import type {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "../screenshot-output";

// A screenshot laid out on a platform's canvas, as a recording's screen is
// (`platform-layout.ts`): its pictures read as one, by the box they cover
// together, which keeps its weight and where its empty space sits and is
// scaled and moved for the platform's shape inside the part its interface
// leaves clear. Every layer moves the same way, so the pictures keep where
// they sit against one another, and the annotations, drawn in each layer's
// own pixels, go with their layers.

type Size = { height: number; width: number };

/** A canvas laid out on: its size, and what its platform covers. */
export type Canvas = { clear: ClearArea | undefined; size: Size };

/** How a point on a canvas moves to another: `offset + p * scale`. */
export type Mapping = { scale: number; x: number; y: number };

/** The canvas `from` laid into `to` whole, scaled to fit and centred. */
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

/** The box the pictures of `workspace` cover together, in output pixels;
 * the layers drawn on, `blank`, being no pictures, left out. None without
 * one. */
const picturesBox = (
  workspace: ScreenshotWorkspaceOutputSettings,
  blank: ReadonlySet<number>,
) => {
  const boxes = workspace.items
    .filter((item) => !blank.has(item.id))
    .map(({ output }) => output)
    .filter((output) => output.cropWidth > 0 && output.cropHeight > 0);
  if (boxes.length === 0) return null;
  const left = Math.min(...boxes.map((box) => box.cropX));
  const top = Math.min(...boxes.map((box) => box.cropY));
  const right = Math.max(...boxes.map((box) => box.cropX + box.cropWidth));
  const bottom = Math.max(...boxes.map((box) => box.cropY + box.cropHeight));
  return { height: bottom - top, width: right - left, x: left, y: top };
};

/** How the pictures of `workspace` move onto the canvas `to`. A canvas of
 * the same proportions, kept clear of the same, is scaled whole, as is one
 * with no picture to read. */
export const workspaceMapping = (
  workspace: ScreenshotWorkspaceOutputSettings,
  { clear, size }: Canvas,
  blank: ReadonlySet<number>,
): Mapping => {
  const box = picturesBox(workspace, blank);
  const isScaled =
    Math.abs(workspace.width * size.height - size.width * workspace.height) <
      Math.max(workspace.width, workspace.height) &&
    JSON.stringify(workspace.clearArea) === JSON.stringify(clear);
  if (isScaled || !box || workspace.width <= 0)
    return fitInside(workspace, size);
  const laid = laidScreen(size, screenIntent(workspace, box), clear);
  const scale = laid.width / box.width;
  return { scale, x: laid.x - box.x * scale, y: laid.y - box.y * scale };
};

/** `settings` moved by `map` onto the canvas `to`. */
export const moved = <Settings extends ScreenshotOutputSettings>(
  settings: Settings,
  map: Mapping,
  { clear, size }: Canvas,
): Settings => {
  const { clearArea: _was, ...rest } = settings;
  return {
    ...rest,
    ...(clear ? { clearArea: clear } : {}),
    cropHeight: settings.cropHeight * map.scale,
    cropWidth: settings.cropWidth * map.scale,
    cropX: map.x + settings.cropX * map.scale,
    cropY: map.y + settings.cropY * map.scale,
    height: Math.max(1, Math.round(size.height)),
    imageWidth: settings.imageWidth * map.scale,
    imageX: map.x + settings.imageX * map.scale,
    imageY: map.y + settings.imageY * map.scale,
    width: Math.max(1, Math.round(size.width)),
  } as Settings;
};

/** The move that undoes `map`. */
export const inverseMapping = (map: Mapping): Mapping => ({
  scale: 1 / map.scale,
  x: -map.x / map.scale,
  y: -map.y / map.scale,
});

/** `workspace` laid out on the canvas `to`: every layer, and the canvas's
 * own placement new layers start from, moved alike. `blank` holds the
 * layers drawn on rather than pictures. */
export const laidWorkspace = (
  workspace: ScreenshotWorkspaceOutputSettings,
  to: Canvas,
  blank: ReadonlySet<number>,
): ScreenshotWorkspaceOutputSettings => {
  const map = workspaceMapping(workspace, to, blank);
  return {
    ...moved(workspace, map, to),
    items: workspace.items.map((item) => ({
      ...item,
      output: moved(item.output, map, to),
    })),
  };
};
