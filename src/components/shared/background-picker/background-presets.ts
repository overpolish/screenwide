// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  Background,
  BackgroundMeshPoint,
  BackgroundPreset,
} from "./background";
import {
  BACKGROUND_GENERATORS,
  BackgroundGenerator,
  DEFAULT_GENERATOR_ID,
} from "./background-generators";

/**
 * The composition tile's blobs, for the tile alone.
 *
 * A composition carries one more colour than it has blobs: the first fills
 * the canvas and each blob takes the next, so three blobs go with a palette
 * of four. The three arrangements below are the ones the classic mesh tile is
 * drawn from, corner-weighted, diagonal, and banded. What a press hands the
 * canvas is not one of these: it is a fresh arrangement under the same
 * colours. The other generators need none of this: they draw from their
 * colours and their seed alone.
 */
const cornerPoints: BackgroundMeshPoint[] = [
  { radiusX: 78, radiusY: 62, rotation: 18, x: 12, y: 14 },
  { radiusX: 64, radiusY: 78, rotation: -34, x: 88, y: 22 },
  { radiusX: 86, radiusY: 58, rotation: 62, x: 46, y: 96 },
];

const diagonalPoints: BackgroundMeshPoint[] = [
  { radiusX: 92, radiusY: 54, rotation: -28, x: -6, y: 8 },
  { radiusX: 70, radiusY: 70, rotation: 40, x: 54, y: 48 },
  { radiusX: 88, radiusY: 60, rotation: -12, x: 108, y: 92 },
];

const bandedPoints: BackgroundMeshPoint[] = [
  { radiusX: 104, radiusY: 44, rotation: 6, x: 50, y: -4 },
  { radiusX: 96, radiusY: 48, rotation: -8, x: 30, y: 52 },
  { radiusX: 104, radiusY: 46, rotation: 4, x: 70, y: 104 },
];

const arrangements = [cornerPoints, diagonalPoints, bandedPoints];

/** A tile's id as a number, so the tile it draws is the same one every time
 * the picker opens. */
const idSeed = (id: string) => {
  let seed = 7;
  for (let index = 0; index < id.length; index += 1)
    seed = (seed * 31 + id.charCodeAt(index)) % 65_521;
  return seed;
};

/**
 * The one picture a generator is shown as.
 *
 * The tile has to stand still: a swatch that redrew itself on every render
 * would be a grid nobody could aim at. The seed, and the composition
 * generator's arrangement with it, both come from the tile's id, so the tile
 * is fixed while what a press hands the canvas is not.
 */
const representative = (
  generator: BackgroundGenerator,
  id: string,
): Background => {
  const seed = idSeed(id);
  const colors = [...generator.defaultColors];
  return {
    colors,
    generator: generator.id,
    kind: "mesh",
    lockedColors: colors.map(() => false),
    ...(generator.id === DEFAULT_GENERATOR_ID
      ? { points: arrangements[seed % arrangements.length] }
      : {}),
    seed,
    warpPercent: 8 + (seed % 6),
  };
};

const solid = (color: string): Background => ({ color, kind: "solid" });

/** One tile per generator: the painter under the colours it ships with. */
const meshPresets: BackgroundPreset[] = BACKGROUND_GENERATORS.map(
  (generator) => {
    const id = `mesh-${generator.id}`;
    return {
      background: representative(generator, id),
      id,
      name: generator.name,
    };
  },
);

const solidPresets: BackgroundPreset[] = [
  // Neutrals, then the saturated tones of the system palette: the colours a
  // backdrop is usually chosen from, with none so pale it reads as unset.
  { background: solid("#FFFFFF"), id: "solid-white", name: "White" },
  { background: solid("#D4D4D8"), id: "solid-light-grey", name: "Light grey" },
  { background: solid("#52525B"), id: "solid-grey", name: "Grey" },
  { background: solid("#171717"), id: "solid-graphite", name: "Graphite" },
  { background: solid("#000000"), id: "solid-black", name: "Black" },
  { background: solid("#FF3B30"), id: "solid-red", name: "Red" },
  { background: solid("#FF9500"), id: "solid-orange", name: "Orange" },
  { background: solid("#FFCC00"), id: "solid-yellow", name: "Yellow" },
  { background: solid("#34C759"), id: "solid-green", name: "Green" },
  { background: solid("#00C7BE"), id: "solid-teal", name: "Teal" },
  { background: solid("#007AFF"), id: "solid-blue", name: "Blue" },
  { background: solid("#5856D6"), id: "solid-indigo", name: "Indigo" },
  { background: solid("#AF52DE"), id: "solid-purple", name: "Purple" },
  { background: solid("#A2845E"), id: "solid-brown", name: "Brown" },
  { background: solid("#1E293B"), id: "solid-navy", name: "Navy" },
  { background: solid("#14532D"), id: "solid-forest", name: "Forest" },
];

/** One picture the system already ships as a desktop background, as the
 * native side describes it. */
export type SystemWallpaper = {
  name: string;
  path: string;
  thumbnailPath?: string | null;
};

/**
 * The backgrounds every install starts with.
 *
 * The generators first, since a gradient is what a canvas is usually given,
 * then the flat tones a screenshot is trimmed to for a document or a slide.
 * The system's own pictures are not in here: they are only known once the
 * native side has read the folder they live in, so a caller without them
 * simply has no wallpaper tiles.
 */
export const BUILT_IN_BACKGROUND_PRESETS: BackgroundPreset[] = [
  ...meshPresets,
  ...solidPresets,
];

/**
 * The built-ins with the system's own desktop pictures folded in.
 *
 * Wallpapers sit between the generators and the flat tones, since they are
 * the same kind of choice as a gradient but a heavier one. A picture the system
 * has a thumbnail for carries it: the picture itself is tens of megabytes,
 * and a swatch is a few dozen pixels across.
 */
export const backgroundPresetsWithWallpapers = (
  wallpapers: SystemWallpaper[],
): BackgroundPreset[] => [
  ...meshPresets,
  ...wallpapers.map(({ name, path, thumbnailPath }) => ({
    background: { kind: "image" as const, path },
    id: `system-wallpaper:${name}`,
    name,
    ...(thumbnailPath ? { thumbnailPath } : {}),
  })),
  ...solidPresets,
];
