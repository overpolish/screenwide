// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { canvasShare } from "../layer-measure";

import { type ScreenshotOutputSettings } from "./screenshot-output";

// A blank canvas (#41) is a screenshot whose layer holds no picture: the
// canvas's own layer, all clear, which holds what is drawn on the canvas as
// any layer holds its annotations. It is drawn with no shadow or corners, so
// it shows nothing of its own; a press on empty canvas never chooses it as a
// picture (the native side's `PreviewSelection::blank`), though a pen drawing
// there joins it. A screenshot left with no layer is given one, so the
// canvas always has a layer to draw on.

type Layer = { id: number; isBlank: boolean };

/** The look a blank layer is drawn in: no shadow, corners or padding, so
 * nothing of it shows but what is drawn on it. */
export const blankLayerOutput = (
  output: ScreenshotOutputSettings,
): ScreenshotOutputSettings => ({
  ...output,
  dropShadow: false,
  inset: canvasShare(0),
  radius: canvasShare(0),
});

/** The layer chosen; the picture chosen, none where the blank layer is, which
 * is never chosen as a picture; the layer a tool takes up where none is
 * chosen, the newest picture or else the blank layer; whether a layer is
 * blank; and whether the workspace holds a picture at all, which Crop and
 * the Screenshot mode need: a blank canvas has none until one is added. */
export function screenshotLayersInHand<T extends Layer>(
  items: readonly T[],
  selectedItemId: number | null,
  workspace: { items: readonly { id: number }[] } | undefined,
) {
  const selectedItem = items.find((item) => item.id === selectedItemId);
  const isBlank = (id: number) =>
    items.some((item) => item.id === id && item.isBlank);
  return {
    hasPicture:
      workspace?.items.some(
        ({ id }) => items.some((item) => item.id === id) && !isBlank(id),
      ) ?? false,
    isBlank,
    newestItemId: newestScreenshotLayer(items)?.id ?? null,
    selectedItem,
    selectedPicture: selectedItem?.isBlank ? undefined : selectedItem,
  };
}

/** The newest picture, or the blank layer where there is none. */
export const newestScreenshotLayer = <T extends Layer>(
  items: readonly T[],
): T | undefined =>
  [...items].reverse().find((item) => !item.isBlank) ?? items[items.length - 1];
