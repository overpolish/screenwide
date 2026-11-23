// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { screenshotFormatChoice } from "./screenshot-format";

import type { ScreenshotFormat } from "../../../bindings/ScreenshotFormat";

/** How the open screenshot is saved, for as long as its window is open: the
 * format, where a canvas too large for the chosen one is saved as a PNG, and
 * whether a PNG keeps every colour rather than a palette of 256. */
export function useScreenshotExport(
  canvas: Parameters<typeof screenshotFormatChoice>[1],
) {
  const [format, setFormat] = useState<ScreenshotFormat>("png");
  const [fullColour, setFullColour] = useState(false);
  const { selected, unavailable } = screenshotFormatChoice(format, canvas);
  return {
    format: selected,
    fullColour,
    setFormat,
    setFullColour,
    unavailable,
  };
}
