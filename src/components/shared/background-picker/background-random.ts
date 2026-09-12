// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { generatePalette, PaletteMode } from "../../../lib/palette-generator";

import { Background, BackgroundMeshPoint } from "./background";
import {
  backgroundGenerator,
  DEFAULT_GENERATOR_ID,
} from "./background-generators";

type Mesh = Extract<Background, { kind: "mesh" }>;

const paletteModes: PaletteMode[] = ["bright", "chaotic", "dull", "shades"];
const MAXIMUM_MESH_COLORS = 5;

const random = (minimum: number, maximum: number) =>
  minimum + Math.random() * (maximum - minimum);

/** The seed every generator is given, in the range the renderer reads. */
const randomSeed = () => Math.floor(random(1, 65_535));

/** The warp that keeps a composition's blend from banding. A generator that
 * does not read it still carries one, so a mesh is one shape whichever
 * painter is drawing it. */
const randomWarp = () => random(5, 14);

const randomPalette = (colorCount: number) =>
  generatePalette(
    paletteModes[Math.floor(Math.random() * paletteModes.length)],
    colorCount,
  );

/**
 * A fresh mesh: a palette, a blob for every colour past the first, and the
 * warp that keeps the blend from banding.
 *
 * A colour count keeps the palette the size it already was, so a randomize
 * with colours locked does not change how many there are to lock.
 */
export const randomMeshBackground = (colorCount?: number): Mesh => {
  const pointCount = colorCount
    ? Math.max(3, Math.min(MAXIMUM_MESH_COLORS - 1, colorCount - 1))
    : 3 + Math.floor(Math.random() * 2);
  const points: BackgroundMeshPoint[] = Array.from(
    { length: pointCount },
    () => ({
      radiusX: random(38, 105),
      radiusY: random(28, 92),
      rotation: random(-180, 180),
      x: random(-18, 118),
      y: random(-18, 118),
    }),
  );
  const colors = randomPalette(pointCount + 1);
  return {
    colors,
    generator: DEFAULT_GENERATOR_ID,
    kind: "mesh",
    lockedColors: colors.map(() => false),
    points,
    seed: randomSeed(),
    warpPercent: randomWarp(),
  };
};

/**
 * A generator's own palette, freshly seeded.
 *
 * A built-in tile is a generator rather than a picture: pressing it hands the
 * canvas that painter under its own colours, with a seed made on the spot, so
 * the same tile never hands back the same picture twice. The composition
 * generator also gets a fresh arrangement, since its seed alone would leave
 * the blobs where the last one put them, and takes a colour count so a
 * palette already chosen keeps a blob for every colour.
 */
export const randomMeshForGenerator = (
  id: string,
  colorCount?: number,
): Mesh => {
  const generator = backgroundGenerator(id);
  const colors = [...generator.defaultColors];
  const lockedColors = colors.map(() => false);
  if (generator.id === DEFAULT_GENERATOR_ID)
    return {
      ...randomMeshBackground(colorCount ?? colors.length),
      colors,
      generator: generator.id,
      lockedColors,
    };
  return {
    colors,
    generator: generator.id,
    kind: "mesh",
    lockedColors,
    seed: randomSeed(),
    warpPercent: randomWarp(),
  };
};

