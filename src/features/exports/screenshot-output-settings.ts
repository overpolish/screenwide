// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  MeshGradientPoint,
  randomMeshComposition,
} from "./screenshot-background";
import { fullSourceRect, sourceRect, SourceRect } from "./screenshot-geometry";

type ScreenshotBackgroundType = "mesh" | "solid";

export type ScreenshotOutputSettings = {
  backgroundColor: string;
  backgroundRadiusPercent: number;
  backgroundType: ScreenshotBackgroundType;
  dropShadow: boolean;
  height: number;
  meshColors: string[];
  meshLockedColors: boolean[];
  meshPoints: MeshGradientPoint[];
  meshSeed: number;
  meshWarpPercent: number;
  radiusPercent: number;
  recenterInsetColor: string | null;
  screenshotCropHeightPercent: number;
  screenshotCropWidthPercent: number;
  screenshotCropXPercent: number;
  screenshotCropYPercent: number;
  screenshotImageWidthPercent: number;
  screenshotImageXPercent: number;
  screenshotImageYPercent: number;
  sourceCrop: SourceRect;
  width: number;
};

export const defaultScreenshotOutput = (
  width: number,
  height: number,
  radii: { background?: number; screenshot?: number } = {},
): ScreenshotOutputSettings => {
  const mesh = randomMeshComposition();
  return {
    ...mesh,
    backgroundColor: "#171717",
    backgroundRadiusPercent: radii.background ?? 0,
    backgroundType: "solid",
    dropShadow: true,
    height,
    meshLockedColors: mesh.meshColors.map(() => false),
    radiusPercent: radii.screenshot ?? 0,
    recenterInsetColor: null,
    screenshotCropHeightPercent: 100,
    screenshotCropWidthPercent: 100,
    screenshotCropXPercent: 0,
    screenshotCropYPercent: 0,
    screenshotImageWidthPercent: 100,
    screenshotImageXPercent: 50,
    screenshotImageYPercent: 50,
    sourceCrop: fullSourceRect(),
    width,
  };
};

const finite = (value: number, fallback: number) =>
  Number.isFinite(value) ? value : fallback;

const validSourceRect = (
  value: Partial<SourceRect> | null | undefined,
  fallback: SourceRect,
) => {
  try {
    return sourceRect({
      height: value?.height ?? fallback.height,
      width: value?.width ?? fallback.width,
      x: value?.x ?? fallback.x,
      y: value?.y ?? fallback.y,
    });
  } catch {
    return fallback;
  }
};

/** Validate the canonical crop at the persistence boundary. */
export const screenshotSourceCrop = (
  settings: ScreenshotOutputSettings,
): SourceRect => validSourceRect(settings.sourceCrop, fullSourceRect());

/** Persist the crop as the only source-space screenshot edit. */
export const withScreenshotSourceCrop = <
  Settings extends ScreenshotOutputSettings,
>(
  settings: Settings,
  sourceCrop: SourceRect,
): Settings => ({ ...settings, sourceCrop });

export const normalizedScreenshotOutput = (
  settings: ScreenshotOutputSettings,
): ScreenshotOutputSettings => {
  const defaults = defaultScreenshotOutput(
    finite(settings.width, 1),
    finite(settings.height, 1),
  );
  const sourceCrop = screenshotSourceCrop(settings);
  const normalized: ScreenshotOutputSettings = {
    ...defaults,
    ...settings,
    sourceCrop,
  };
  for (const key of [
    "backgroundRadiusPercent",
    "meshWarpPercent",
    "radiusPercent",
    "screenshotCropHeightPercent",
    "screenshotCropWidthPercent",
    "screenshotCropXPercent",
    "screenshotCropYPercent",
    "screenshotImageWidthPercent",
    "screenshotImageXPercent",
    "screenshotImageYPercent",
  ] as const)
    normalized[key] = finite(settings[key], defaults[key]);
  normalized.height = Math.max(
    1,
    Math.round(finite(settings.height, defaults.height)),
  );
  normalized.width = Math.max(
    1,
    Math.round(finite(settings.width, defaults.width)),
  );
  normalized.meshLockedColors = settings.meshColors.map(
    (_, index) => settings.meshLockedColors[index] ?? false,
  );
  normalized.meshPoints = settings.meshPoints.map((point, index) => {
    const fallback = defaults.meshPoints[index] ?? defaults.meshPoints[0];
    return {
      radiusX: finite(point.radiusX, fallback.radiusX),
      radiusY: finite(point.radiusY, fallback.radiusY),
      rotation: finite(point.rotation, fallback.rotation),
      x: finite(point.x, fallback.x),
      y: finite(point.y, fallback.y),
    };
  });
  normalized.recenterInsetColor = settings.recenterInsetColor ?? null;
  return withScreenshotSourceCrop(normalized, sourceCrop);
};
