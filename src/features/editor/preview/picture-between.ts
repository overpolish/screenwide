// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { CameraOverlaySettings } from "../../../bindings/CameraOverlaySettings";
import type { MeshGradientPoint } from "../../../bindings/MeshGradientPoint";
import type { SceneRotation } from "../../../bindings/SceneRotation";
import type { ScreenshotOutputSettings } from "../../../bindings/ScreenshotOutputSettings";

// The settings a picture is drawn with part of the way from one look to
// another (`picture-tween.ts`). Every in-between is drawn only, never saved,
// and the last step of a tween is the new settings themselves, so whatever
// an in-between borrows from either side to keep the picture continuous,
// such as a shadow's strength, never outlives the tween.

export const between = (from: number, to: number, share: number) =>
  from + (to - from) * share;

const UPRIGHT: SceneRotation = { x: 0, y: 0, z: 0 };

/** A turn between `from` and `to`, upright standing in for a side that has
 * none, so a turn grows out of upright and settles back into it. */
export const rotationBetween = (
  from: SceneRotation | undefined,
  to: SceneRotation | undefined,
  share: number,
): SceneRotation | undefined => {
  if (!from && !to) return to;
  const [start, end] = [from ?? UPRIGHT, to ?? UPRIGHT];
  return {
    x: between(start.x, end.x, share),
    y: between(start.y, end.y, share),
    z: between(start.z, end.z, share),
  };
};

const HEX_COLOUR = /^#([0-9a-f]{6})$/i;

/** A colour between two `#rrggbb` colours; any other colour is `to`'s. */
export const colourBetween = (from: string, to: string, share: number) => {
  const start = HEX_COLOUR.exec(from)?.[1];
  const end = HEX_COLOUR.exec(to)?.[1];
  if (!start || !end || start.toLowerCase() === end.toLowerCase()) return to;
  const channel = (hex: string, index: number) =>
    parseInt(hex.slice(index * 2, index * 2 + 2), 16);
  return `#${[0, 1, 2]
    .map((index) =>
      Math.round(between(channel(start, index), channel(end, index), share))
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`;
};

/** How strong a picture's shadow is drawn, none to whole. */
const shadowStrength = (settings: ScreenshotOutputSettings) =>
  settings.dropShadow ? (settings.dropShadowStrength ?? 1) : 0;

/** A mesh between two that paint the same picture with the same number of
 * colours and points; another is `to`'s at once. */
const meshBetween = (
  from: ScreenshotOutputSettings,
  to: ScreenshotOutputSettings,
  share: number,
): Partial<ScreenshotOutputSettings> => {
  if (
    from.backgroundType !== to.backgroundType ||
    from.meshGenerator !== to.meshGenerator ||
    from.meshColors.length !== to.meshColors.length ||
    from.meshPoints.length !== to.meshPoints.length
  )
    return {};
  return {
    meshColors: to.meshColors.map((colour, index) =>
      colourBetween(from.meshColors[index], colour, share),
    ),
    meshPoints: to.meshPoints.map((point, index): MeshGradientPoint => ({
      radiusX: between(from.meshPoints[index].radiusX, point.radiusX, share),
      radiusY: between(from.meshPoints[index].radiusY, point.radiusY, share),
      rotation: between(from.meshPoints[index].rotation, point.rotation, share),
      x: between(from.meshPoints[index].x, point.x, share),
      y: between(from.meshPoints[index].y, point.y, share),
    })),
    meshWarpPercent: between(from.meshWarpPercent, to.meshWarpPercent, share),
  };
};

/** One picture's settings drawn between `from` and `to`: its numbers and
 * colours part of the way, its turn grown out of upright, its shadow faded
 * in or out, and the colour its padding is filled with kept from whichever
 * side has one, so the padding never shows the canvas through it while it
 * shrinks away. What cannot be drawn in between, such as a picture
 * background, is `to`'s. */
export const outputBetween = <T extends ScreenshotOutputSettings>(
  from: T,
  to: T,
  share: number,
): T => {
  const strength = between(shadowStrength(from), shadowStrength(to), share);
  const rotation = rotationBetween(from.rotation, to.rotation, share);
  return {
    ...to,
    backgroundColor: colourBetween(
      from.backgroundColor,
      to.backgroundColor,
      share,
    ),
    backgroundRadiusPercent: between(
      from.backgroundRadiusPercent,
      to.backgroundRadiusPercent,
      share,
    ),
    cropHeight: between(from.cropHeight, to.cropHeight, share),
    cropWidth: between(from.cropWidth, to.cropWidth, share),
    cropX: between(from.cropX, to.cropX, share),
    cropY: between(from.cropY, to.cropY, share),
    dropShadow: strength > 0,
    dropShadowStrength: strength,
    // Whole pixels: the canvas is measured in them.
    height: Math.round(between(from.height, to.height, share)),
    imageWidth: between(from.imageWidth, to.imageWidth, share),
    imageX: between(from.imageX, to.imageX, share),
    imageY: between(from.imageY, to.imageY, share),
    ...(from.inset !== undefined && to.inset !== undefined
      ? { inset: between(from.inset, to.inset, share) }
      : {}),
    ...meshBetween(from, to, share),
    radiusPercent: between(from.radiusPercent, to.radiusPercent, share),
    recenterInsetColor: to.recenterInsetColor ?? from.recenterInsetColor,
    ...(rotation ? { rotation } : {}),
    sourceCrop: {
      ...to.sourceCrop,
      height: between(from.sourceCrop.height, to.sourceCrop.height, share),
      width: between(from.sourceCrop.width, to.sourceCrop.width, share),
      x: between(from.sourceCrop.x, to.sourceCrop.x, share),
      y: between(from.sourceCrop.y, to.sourceCrop.y, share),
    },
    width: Math.round(between(from.width, to.width, share)),
  };
};

/** The camera drawn into the picture between `from` and `to`: its frame,
 * its crop and its corners part of the way. */
export const cameraOverlayBetween = (
  from: CameraOverlaySettings,
  to: CameraOverlaySettings,
  share: number,
): CameraOverlaySettings => ({
  ...to,
  cameraWidth: between(from.cameraWidth, to.cameraWidth, share),
  cameraX: between(from.cameraX, to.cameraX, share),
  cameraY: between(from.cameraY, to.cameraY, share),
  frameHeight: between(from.frameHeight, to.frameHeight, share),
  frameWidth: between(from.frameWidth, to.frameWidth, share),
  frameX: between(from.frameX, to.frameX, share),
  frameY: between(from.frameY, to.frameY, share),
  radiusPercent: between(from.radiusPercent, to.radiusPercent, share),
});
