// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotLayout } from "../screenshot-output";

/** Convert output-pixel layout into the pane fractions native OSCs consume. */
export function normalizedScreenshotSelection(
  layout: ScreenshotLayout,
  output: { height: number; width: number },
  mode: "crop" | "recenter" | "select" = "select",
) {
  const height = Math.max(1, output.height);
  const width = Math.max(1, output.width);
  const fractions = (box: ScreenshotLayout["crop"]) => ({
    height: box.height / height,
    width: box.width / width,
    x: box.x / width,
    y: box.y / height,
  });
  const image = fractions(layout.image);
  const bounds = fractions(layout.crop);
  const sourceCrop = fractions(layout.sourceCrop);
  const canvas = { height: 1, width: 1, x: 0, y: 0 };
  return {
    image: mode === "crop" ? image : mode === "recenter" ? sourceCrop : image,
    recenterBounds:
      mode === "recenter" ? canvas : mode === "crop" ? image : undefined,
    rect: mode === "recenter" || mode === "select" ? bounds : sourceCrop,
  };
}
