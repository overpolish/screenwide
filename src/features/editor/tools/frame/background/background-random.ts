// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  generatePalette,
  PaletteMode,
} from "../../../../../lib/palette-generator";

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

/** One of the palette moods, each as likely as the others. */
export const randomPaletteMode = (): PaletteMode =>
  paletteModes[Math.floor(Math.random() * paletteModes.length)];

const randomPalette = (colorCount: number) =>
  generatePalette(randomPaletteMode(), colorCount);

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

/**
 * Ember's palette: a blob's colour for each of its four blobs, in order,
 * then the base the renderer fills the canvas with, which is the last. The
 * third blob is the light, the fourth its faint answer across the canvas,
 * and the first two near-black, so the dark has depth rather than being flat.
 */
export const EMBER_COLORS = [
  "#0D0B0A",
  "#110E0C",
  "#5C2A0E",
  "#22140B",
  "#0A0908",
];

/** Whether a mesh is Ember: the composition under Ember's own colours. A
 * palette of your own is a composition like any other. */
export const isEmberMesh = (mesh: Mesh) =>
  mesh.generator === DEFAULT_GENERATOR_ID &&
  mesh.colors.length === EMBER_COLORS.length &&
  mesh.colors.every(
    (color, index) => color.toUpperCase() === EMBER_COLORS[index],
  );

/**
 * Ember's blobs, freshly placed: one large light low or high on the canvas,
 * a fainter one on the far side and the opposite edge, and two dark blobs
 * inside. The light sits on an edge so it reads as light falling in from
 * outside rather than a patch on the canvas.
 */
export const randomEmberPoints = (): BackgroundMeshPoint[] => {
  const lightX = random(10, 90);
  const isLow = Math.random() < 0.5;
  const edge = (low: boolean) => (low ? random(98, 110) : random(-10, 2));
  return [
    {
      radiusX: random(35, 50),
      radiusY: random(50, 70),
      rotation: random(-20, 20),
      x: random(5, 35),
      y: random(15, 60),
    },
    {
      radiusX: random(35, 55),
      radiusY: random(55, 75),
      rotation: random(-20, 20),
      x: random(35, 65),
      y: random(-5, 40),
    },
    {
      radiusX: random(60, 80),
      radiusY: random(70, 90),
      rotation: random(-15, 15),
      x: lightX,
      y: edge(isLow),
    },
    {
      radiusX: random(35, 50),
      radiusY: random(50, 65),
      rotation: random(-25, 25),
      x: lightX < 50 ? random(88, 110) : random(-10, 12),
      y: edge(!isLow),
    },
  ];
};

/** Ember with its light somewhere new. */
export const randomEmberMesh = (): Mesh => ({
  colors: [...EMBER_COLORS],
  generator: DEFAULT_GENERATOR_ID,
  kind: "mesh",
  lockedColors: EMBER_COLORS.map(() => false),
  points: randomEmberPoints(),
  seed: randomSeed(),
  warpPercent: random(5, 10),
});

/** Roll a mesh's arrangement while keeping its palette and generator intact.
 * Ember keeps its shape and moves its light. */
export const randomizeMeshBackground = (mesh: Mesh): Mesh => {
  if (isEmberMesh(mesh))
    return { ...mesh, points: randomEmberPoints(), seed: randomSeed() };
  const rolled = randomMeshForGenerator(mesh.generator, mesh.colors.length);
  return {
    ...mesh,
    ...(mesh.generator === DEFAULT_GENERATOR_ID
      ? { points: rolled.points }
      : {}),
    seed: rolled.seed,
  };
};
