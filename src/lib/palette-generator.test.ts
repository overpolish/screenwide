// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  generatePalette,
  generatePaletteFromLocked,
  hexToOklch,
  PaletteMode,
} from "./palette-generator";

const modes: PaletteMode[] = ["bright", "chaotic", "dull", "shades"];
const counts = [1, 2, 3, 4, 5];

/** The band each mode promises, plus room for 8-bit rounding. Chroma only has
 * an upper bound: a colour outside sRGB loses chroma to get back in. */
const expected = {
  bright: { chroma: 0.22, lightness: [0.62, 0.8] },
  chaotic: { chroma: 0.25, lightness: [0.35, 0.85] },
  dull: { chroma: 0.08, lightness: [0.45, 0.7] },
  shades: { chroma: 0.16, lightness: [0.2, 0.92] },
} satisfies Record<PaletteMode, { chroma: number; lightness: number[] }>;

const TOLERANCE = 0.02;
const GREY_CHROMA = 0.04;

const labOf = (hex: string) => {
  const { c, h, l } = hexToOklch(hex);
  const angle = (h * Math.PI) / 180;
  return { a: c * Math.cos(angle), b: c * Math.sin(angle), l };
};

const minimumDistance = (colors: string[]) => {
  const labs = colors.map((color) => labOf(color));
  let nearest = Number.POSITIVE_INFINITY;
  for (let one = 0; one < labs.length; one += 1)
    for (let other = one + 1; other < labs.length; other += 1)
      nearest = Math.min(
        nearest,
        Math.hypot(
          labs[one].l - labs[other].l,
          labs[one].a - labs[other].a,
          labs[one].b - labs[other].b,
        ),
      );
  return nearest;
};

describe("palette generator", () => {
  it("returns the asked-for count as uppercase hex", () => {
    for (const mode of modes)
      for (const count of counts) {
        const palette = generatePalette(mode, count);
        expect(palette).toHaveLength(count);
        for (const color of palette) expect(color).toMatch(/^#[0-9A-F]{6}$/);
      }
  });

  it("stays inside each mode's band", () => {
    for (const mode of modes) {
      const [minimum, maximum] = expected[mode].lightness;
      for (let run = 0; run < 30; run += 1)
        for (const color of generatePalette(mode, 5)) {
          const { c, l } = hexToOklch(color);
          expect(l).toBeGreaterThanOrEqual(minimum - TOLERANCE);
          expect(l).toBeLessThanOrEqual(maximum + TOLERANCE);
          expect(c).toBeLessThanOrEqual(expected[mode].chroma + TOLERANCE);
        }
    }
  });

  it("keeps locked colours where they are", () => {
    const colors = ["#112233", "#445566", "#778899", "#AABBCC", "#DDEEFF"];
    const locked = [false, true, false, true, false];
    for (const mode of modes) {
      const result = generatePaletteFromLocked({ colors, locked, mode });
      expect(result).toHaveLength(colors.length);
      expect(result[1]).toBe(colors[1]);
      expect(result[3]).toBe(colors[3]);
      for (const index of [0, 2, 4]) {
        expect(result[index]).not.toBe(colors[index]);
        expect(result[index]).toMatch(/^#[0-9A-F]{6}$/);
      }
    }
  });

  it("leaves a fully locked palette alone", () => {
    const colors = ["#112233", "#445566"];
    expect(
      generatePaletteFromLocked({
        colors,
        locked: [true, true],
        mode: "bright",
      }),
    ).toEqual(colors);
  });

  /** Farthest-point selection should never hand back two colours that sit on
   * top of each other. 0.08 in OKLab is roughly the point where two swatches
   * stop reading as separate; measured worst case over 300 runs is near 0.10. */
  it("keeps bright palettes apart", () => {
    for (let run = 0; run < 50; run += 1)
      expect(minimumDistance(generatePalette("bright", 5))).toBeGreaterThan(
        0.08,
      );
  });

  it("gives shades a single hue", () => {
    for (let run = 0; run < 30; run += 1) {
      const hues = generatePalette("shades", 5)
        .map((color) => hexToOklch(color))
        .filter(({ c }) => c > GREY_CHROMA)
        .map(({ h }) => h);
      for (const hue of hues) {
        const difference = Math.abs(hue - hues[0]) % 360;
        expect(Math.min(difference, 360 - difference)).toBeLessThan(5);
      }
    }
  });

  it("takes the shades hue from a locked colour", () => {
    const locked = ["#2E7D32"];
    const result = generatePaletteFromLocked({
      colors: [...locked, "#000000", "#000000", "#000000", "#000000"],
      locked: [true, false, false, false, false],
      mode: "shades",
    });
    const target = hexToOklch(locked[0]).h;
    for (const { c, h } of result.map((color) => hexToOklch(color))) {
      if (c <= GREY_CHROMA) continue;
      const difference = Math.abs(h - target) % 360;
      expect(Math.min(difference, 360 - difference)).toBeLessThan(5);
    }
  });
});
