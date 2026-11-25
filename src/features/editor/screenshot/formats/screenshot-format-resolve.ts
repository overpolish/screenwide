// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  BACKGROUND,
  placedFrom,
  SCREEN,
} from "../../recording/formats/format-look-carry";
import { proportionKey } from "../../recording/formats/format-scope";
import { platformClearArea } from "../../recording/formats/platform-clear-area";
import {
  isMainCanvas,
  mainDocument,
} from "../../recording/formats/recording-formats";
import { frameSizePreset } from "../../tools/frame/frame-sizes";

import { withLayouts } from "./screenshot-annotation-layouts";
import {
  type Canvas,
  laidWorkspace,
  workspaceMapping,
} from "./screenshot-format-layout";
import { laidApart } from "./screenshot-show-in";

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";
import type { ScreenshotShape } from "../../../../bindings/ScreenshotShape";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

// What a screenshot's shape shows of its main canvas (`screenshot-formats.ts`):
// its canvas, the main canvas's layers laid out on it, and what it sets its
// own way.

type Workspace = ScreenshotWorkspaceOutputSettings;
/** The layers drawn on rather than pictures, by id. */
type Blank = ReadonlySet<number>;
type Shown = { blank: Blank; main: Workspace; shape: ScreenshotShape };

/** The canvas of `shape`, at its platform's template's size and kept clear
 * of what its apps cover, or the main canvas's own. */
export const canvasOf = (shape: ScreenshotShape, main: Workspace): Canvas => {
  const platforms = shape.platforms ?? [];
  return {
    clear: platformClearArea(platforms),
    size: frameSizePreset(platforms[0]) ?? shape.look ?? main,
  };
};

/** The size of `shape`'s canvas, `inHand` the workspace of the one in hand. */
export const shapeSize = (
  formats: ScreenshotFormats,
  shape: ScreenshotShape,
  inHand: Workspace,
) => {
  if (shape.id === formats.active) return inHand;
  if (isMainCanvas(formats, shape.id)) return shape.look ?? inHand;
  return canvasOf(shape, mainDocument(formats, inHand)).size;
};

/** The workspace `shape` shows of the main canvas `main`: laid out from it,
 * with its own layers' places where it set them its own way, and its own
 * background where it set that. */
const resolvedWorkspace = (
  main: Workspace,
  shape: ScreenshotShape,
  blank: Blank,
): Workspace => {
  if (!shape.platforms?.length) return main;
  const laid = laidWorkspace(main, canvasOf(shape, main), blank);
  const own = shape.placed ? shape.look : undefined;
  const places = new Map(own?.items.map((item) => [item.id, item.output]));
  const shown = own
    ? {
        ...placedFrom(SCREEN, own, laid),
        items: laid.items.map((item) => {
          const placed = places.get(item.id);
          return placed
            ? { ...item, output: placedFrom(SCREEN, placed, item.output) }
            : item;
        }),
      }
    : laid;
  return shape.ownBackground && shape.look
    ? placedFrom(BACKGROUND, shape.look, shown)
    : shown;
};

/** The workspace `shape` shows in `formats` of the main canvas `main`
 * (`resolvedWorkspace`), laid out without the layers its formats leave out
 * and with its proportion's own annotations; `blank` the layers drawn on
 * rather than pictures. */
export const shownWorkspace = (
  formats: ScreenshotFormats,
  { blank, main, shape }: Shown,
): Workspace => {
  if (!shape.platforms?.length) return main;
  const shown = resolvedWorkspace(
    main,
    shape,
    laidApart(formats, { blank, shape }),
  );
  return withLayouts(shown, {
    key: proportionKey(canvasOf(shape, main).size),
    layouts: formats.layouts,
  });
};

/** How much larger than on the main canvas `shape` draws what annotations
 * draw: the scale its layout lays the main canvas out at, so a platform
 * shows the main canvas made smaller or larger, strokes and text with it.
 * One on the main canvas. */
export const sizeScaleOf = (formats: ScreenshotFormats, shown: Shown) =>
  shown.shape.platforms?.length
    ? workspaceMapping(
        shown.main,
        canvasOf(shown.shape, shown.main),
        laidApart(formats, shown),
      ).scale
    : 1;

/** `workspace` as a format drawing annotations `scale` times their size on
 * the main canvas draws it (`sizeScaleOf`). */
export const withSizeScale = (workspace: Workspace, scale: number) =>
  scale === 1
    ? workspace
    : {
        ...workspace,
        items: workspace.items.map((item) => ({ ...item, sizeScale: scale })),
      };
