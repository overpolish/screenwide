// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** One blob of a mesh gradient, in shares of the canvas. */
export type BackgroundMeshPoint = {
  radiusX: number;
  radiusY: number;
  rotation: number;
  x: number;
  y: number;
};

/**
 * What sits behind the picture, as the picker speaks of it.
 *
 * The three kinds are the three ways a canvas is filled: one colour, a mesh
 * of them, or a picture of your own. Output settings carry the same values
 * spread across flat fields; the editor's adapters translate, so the picker
 * never sees an output canvas.
 */
export type Background =
  | { color: string; kind: "solid" }
  | { kind: "image"; path: string }
  | {
      colors: string[];
      /** Which painter draws this mesh, by the name in the generator table.
       * The classic "mesh" is the only one that reads `points` and the warp;
       * the rest take their colours and the seed and nothing else. */
      generator: string;
      kind: "mesh";
      lockedColors: boolean[];
      seed: number;
      warpPercent: number;
      points?: BackgroundMeshPoint[];
    };

/** A background under a name, offered as a tile in the picker. */
export type BackgroundPreset = {
  background: Background;
  id: string;
  name: string;
  /** A small copy of the picture, where whoever offered it already has one.
   * The system's desktop pictures are tens of megabytes each, so a swatch
   * shows the system's own thumbnail rather than the picture itself. */
  thumbnailPath?: string;
};

const sameColors = (a: string[], b: string[]) =>
  a.length === b.length &&
  a.every((color, index) => color.toUpperCase() === b[index]?.toUpperCase());

/**
 * Whether two backgrounds paint the same thing.
 *
 * A preset tile is selected by value rather than by identity, so a background
 * restored from disk still matches the preset it came from. A mesh preset is
 * a generator under a palette rather than a picture: every picture that
 * generator draws from those colours is that preset, so only the generator
 * and the colours are compared, and the geometry, the seed, the warp and the
 * lock flags are all left out.
 */
export const sameBackground = (a: Background, b: Background): boolean => {
  if (a.kind !== b.kind) return false;
  if (a.kind === "solid" && b.kind === "solid")
    return a.color.toUpperCase() === b.color.toUpperCase();
  if (a.kind === "image" && b.kind === "image") return a.path === b.path;
  if (a.kind === "mesh" && b.kind === "mesh")
    return a.generator === b.generator && sameColors(a.colors, b.colors);
  return false;
};

/** A name for a preset made from a background, since none is asked for: the
 * generator or the colour, or the picture's file name. */
export const presetName = (background: Background): string => {
  if (background.kind === "solid") return background.color.toUpperCase();
  if (background.kind === "mesh") return background.generator;
  const file = background.path.split(/[\\/]/).pop() ?? "Image";
  return file.replace(/\.[^.]+$/, "");
};
