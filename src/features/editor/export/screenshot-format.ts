// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";

/** The longest side each format holds, in pixels: a JPEG's header gives a
 * side sixteen bits, and libwebp, which Rust's `encode_webp` writes with, 14.
 * A PNG holds any screenshot there is memory for. */
const LONGEST_SIDE: Record<ScreenshotFormat, number> = {
  jpeg: 65_535,
  png: Number.POSITIVE_INFINITY,
  webp: 16_383,
};

export type ScreenshotFormatChoice = {
  /** The format the screenshot is saved as. */
  selected: ScreenshotFormat;
  /** The formats too small to hold it, which cannot be chosen. */
  unavailable: ScreenshotFormat[];
};

/** The format a `canvas`-sized screenshot is saved as, and those that
 * cannot hold it. A choice that cannot hold it is saved as a PNG. */
export function screenshotFormatChoice(
  chosen: ScreenshotFormat,
  canvas: { height: number; width: number } | null,
): ScreenshotFormatChoice {
  const longest = canvas ? Math.max(canvas.width, canvas.height) : 0;
  const unavailable = (["png", "jpeg", "webp"] as const).filter(
    (format) => longest > LONGEST_SIDE[format],
  );
  return {
    selected: unavailable.includes(chosen) ? "png" : chosen,
    unavailable,
  };
}
