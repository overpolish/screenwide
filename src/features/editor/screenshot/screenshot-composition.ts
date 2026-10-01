// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotOutputSettings } from "./screenshot-output";
import { screenshotSourceCrop } from "./screenshot-output-settings";

export const hasOutputComposition = (
  settings: ScreenshotOutputSettings,
  source: { height: number; width: number },
) => {
  const sourceCrop = screenshotSourceCrop(settings);
  return (
    settings.width !== source.width ||
    settings.height !== source.height ||
    settings.backgroundRadiusPercent > 0 ||
    settings.radiusPercent > 0 ||
    settings.recenterInsetColor !== null ||
    Math.abs(settings.cropHeight - settings.height) > 0.000_001 ||
    Math.abs(settings.cropWidth - settings.width) > 0.000_001 ||
    Math.abs(settings.cropX) > 0.000_001 ||
    Math.abs(settings.cropY) > 0.000_001 ||
    Math.abs(settings.imageWidth - settings.width) > 0.000_001 ||
    Math.abs(settings.imageX) > 0.000_001 ||
    Math.abs(settings.imageY) > 0.000_001 ||
    Math.abs(sourceCrop.height - 1) > 0.000_001 ||
    Math.abs(sourceCrop.width - 1) > 0.000_001 ||
    Math.abs(sourceCrop.x) > 0.000_001 ||
    Math.abs(sourceCrop.y) > 0.000_001
  );
};
