// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Background } from "../../components/shared/background-picker/background";
import { DEFAULT_GENERATOR_ID } from "../../components/shared/background-picker/background-generators";

import { randomMeshComposition } from "./screenshot-background";
import { ScreenshotOutputSettings } from "./screenshot-output";

/**
 * The canvas's background, as the picker speaks of it.
 *
 * Output settings carry all three backgrounds at once, in flat fields, so the
 * mesh a canvas was given is still there after a spell on a solid colour. The
 * background type says which of them is being drawn, and that is the only one
 * the picker is shown.
 */
export const backgroundFromOutput = (
  settings: ScreenshotOutputSettings,
): Background => {
  if (settings.backgroundType === "image")
    return { kind: "image", path: settings.backgroundImagePath ?? "" };
  if (settings.backgroundType === "mesh")
    return {
      colors: settings.meshColors,
      generator: settings.meshGenerator || DEFAULT_GENERATOR_ID,
      kind: "mesh",
      lockedColors: settings.meshLockedColors,
      points: settings.meshPoints,
      seed: settings.meshSeed,
      warpPercent: settings.meshWarpPercent,
    };
  return { color: settings.backgroundColor, kind: "solid" };
};

/**
 * A background put back into a canvas.
 *
 * Only the chosen kind's own fields are written, so the other two keep what
 * they had: a canvas swapped to a colour and back finds its mesh unchanged. A
 * composition saved without its geometry is given a fresh one at the size of
 * its palette, since blobless blobs would render as one flat colour. The
 * other generators have no geometry to miss: they draw from their colours and
 * their seed, so none is invented for them and the canvas keeps the blobs it
 * already had for the day it is a composition again.
 */
export const applyBackgroundToOutput = <
  Settings extends ScreenshotOutputSettings,
>(
  settings: Settings,
  background: Background,
): Settings => {
  if (background.kind === "solid")
    return {
      ...settings,
      backgroundColor: background.color,
      backgroundType: "solid",
    };
  if (background.kind === "image")
    return {
      ...settings,
      backgroundImagePath: background.path,
      backgroundType: "image",
    };
  const isComposition = background.generator === DEFAULT_GENERATOR_ID;
  const points =
    background.points && background.points.length > 0
      ? background.points
      : isComposition
        ? randomMeshComposition(background.colors.length).meshPoints
        : settings.meshPoints;
  return {
    ...settings,
    backgroundType: "mesh",
    meshColors: background.colors,
    meshGenerator: background.generator,
    meshLockedColors: background.colors.map(
      (_, index) => background.lockedColors[index] ?? false,
    ),
    meshPoints: points,
    meshSeed: background.seed,
    meshWarpPercent: background.warpPercent,
  };
};
