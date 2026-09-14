// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { DEFAULT_GENERATOR_ID } from "../../components/shared/background-picker/background-generators";

import { Annotation, validAnnotations } from "./annotations";
import {
  MeshGradientPoint,
  randomMeshComposition,
} from "./screenshot-background";
import { fullSourceRect, sourceRect, SourceRect } from "./screenshot-geometry";

type ScreenshotBackgroundType = "image" | "mesh" | "solid";

/**
 * One layer's canvas and its placement in it.
 *
 * Placement is in output pixels, not in shares of the canvas, so the canvas
 * can be resized without moving or rescaling anything placed in it: the crop
 * is the visible rectangle, and the image behind it is given by its top left
 * corner and its width, its height following the source's aspect.
 */
/** The crop tool's live result rectangle, in output pixels. */
type CropPreviewRect = {
  height: number;
  width: number;
  x: number;
  y: number;
};

export type ScreenshotOutputSettings = {
  /** Marks drawn over this layer, in the source's own pixel space. */
  annotations: Annotation[];
  backgroundColor: string;
  /** A picture of your own behind the layers, or null for a painted one. */
  backgroundImagePath: string | null;
  backgroundRadiusPercent: number;
  backgroundType: ScreenshotBackgroundType;
  cropHeight: number;
  cropWidth: number;
  cropX: number;
  cropY: number;
  dropShadow: boolean;
  height: number;
  imageWidth: number;
  imageX: number;
  imageY: number;
  meshColors: string[];
  /** Which painter draws the mesh, by the name in the picker's generator
   * table. The classic "mesh" is the only one that reads the points and the
   * warp below. */
  meshGenerator: string;
  meshLockedColors: boolean[];
  meshPoints: MeshGradientPoint[];
  meshSeed: number;
  meshWarpPercent: number;
  radiusPercent: number;
  recenterInsetColor: string | null;
  sourceCrop: SourceRect;
  width: number;
  /**
   * Where the cropped layer lands while the crop tool previews the whole
   * uncropped source, so the compositor can draw it over that ghost with its
   * real corner radius and drop shadow. Only the crop-mode preview payload
   * carries one; it is never persisted.
   */
  cropPreview?: CropPreviewRect;
};

export const defaultScreenshotOutput = (
  width: number,
  height: number,
  radii: { background?: number; screenshot?: number } = {},
): ScreenshotOutputSettings => {
  const mesh = randomMeshComposition();
  return {
    ...mesh,
    annotations: [],
    backgroundColor: "#171717",
    backgroundImagePath: null,
    backgroundRadiusPercent: radii.background ?? 0,
    backgroundType: "solid",
    cropHeight: height,
    cropWidth: width,
    cropX: 0,
    cropY: 0,
    dropShadow: true,
    height,
    imageWidth: width,
    imageX: 0,
    imageY: 0,
    meshGenerator: DEFAULT_GENERATOR_ID,
    meshLockedColors: mesh.meshColors.map(() => false),
    radiusPercent: radii.screenshot ?? 0,
    recenterInsetColor: null,
    sourceCrop: fullSourceRect(),
    width,
  };
};

/**
 * Settings borrowed as a template: for a new document, a layer added to one,
 * or the look remembered from the last export.
 *
 * Marks are document content, not a preference. They are drawn on one layer
 * of one capture and mean nothing on another, so anything copied as a default
 * leaves them behind - otherwise the arrow drawn on yesterday's screenshot
 * turns up on today's.
 */
export const screenshotOutputTemplate = <
  Settings extends ScreenshotOutputSettings,
>(
  settings: Settings,
): Settings => ({ ...settings, annotations: [] });

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
    "cropHeight",
    "cropWidth",
    "cropX",
    "cropY",
    "imageWidth",
    "imageX",
    "imageY",
    "meshWarpPercent",
    "radiusPercent",
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
  // A canvas saved before the generators existed, or with the field emptied,
  // is the composition mesh: that is what it was drawn as.
  normalized.meshGenerator =
    normalized.meshGenerator.trim() || DEFAULT_GENERATOR_ID;
  normalized.annotations = validAnnotations(settings.annotations);
  normalized.backgroundImagePath = settings.backgroundImagePath ?? null;
  normalized.recenterInsetColor = settings.recenterInsetColor ?? null;
  // Display-only, and never persisted: a rectangle that is not wholly finite
  // is no rectangle at all rather than something the compositor has to guard.
  const cropPreview = settings.cropPreview;
  if (
    cropPreview &&
    [cropPreview.height, cropPreview.width, cropPreview.x, cropPreview.y].every(
      (value) => Number.isFinite(value),
    )
  )
    normalized.cropPreview = cropPreview;
  else delete normalized.cropPreview;
  return withScreenshotSourceCrop(normalized, sourceCrop);
};
