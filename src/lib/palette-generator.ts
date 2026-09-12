// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { randomInRange, Range } from "./math";

export type PaletteMode = "bright" | "chaotic" | "dull" | "shades";

/** A colour in OKLab, the space every distance in this file is measured in. */
type Oklab = { a: number; b: number; l: number };
/** The same colour in polar form: lightness, chroma, hue in degrees. */
type Oklch = { c: number; h: number; l: number };

/** A chosen colour, kept in both forms so scoring never reconverts. */
type Chosen = { chroma: number; hue: number; lab: Oklab };

/**
 * A mode is a band, not a curve. Picking inside a band keeps every palette in
 * the same family while leaving the generator free to land anywhere in it, so
 * two runs of the same mode rarely look alike.
 */
type Band = {
  chroma: Range;
  lightness: Range;
  /** Shades take one hue for the whole palette; variety comes from lightness. */
  sameHue: boolean;
};

const bands = {
  bright: {
    chroma: { max: 0.22, min: 0.14 },
    lightness: { max: 0.8, min: 0.62 },
    sameHue: false,
  },
  chaotic: {
    chroma: { max: 0.25, min: 0.08 },
    lightness: { max: 0.85, min: 0.35 },
    sameHue: false,
  },
  dull: {
    chroma: { max: 0.08, min: 0.03 },
    lightness: { max: 0.7, min: 0.45 },
    sameHue: false,
  },
  shades: {
    chroma: { max: 0.16, min: 0.06 },
    lightness: { max: 0.92, min: 0.2 },
    sameHue: true,
  },
} satisfies Record<PaletteMode, Band>;

/** Candidates drawn per slot. Enough to spread well, cheap enough to be free. */
const CANDIDATES = 24;
/** How much a shared hue family costs a candidate, on top of OKLab distance. */
const HUE_WEIGHT = 0.25;
/** Below this chroma a colour reads as grey, so its hue means nothing. */
const GREY_CHROMA = 0.04;
/** Binary search steps used to walk chroma back into sRGB. */
const GAMUT_STEPS = 12;

const clamp = (value: number, minimum: number, maximum: number) =>
  Math.min(maximum, Math.max(minimum, value));

const toLinear = (value: number) =>
  value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;

const toGamma = (value: number) =>
  value <= 0.0031308 ? value * 12.92 : 1.055 * value ** (1 / 2.4) - 0.055;

/** Björn Ottosson's OKLab: linear sRGB through the LMS cone matrix, cube root,
 * then the second matrix into perceptual axes. */
const linearToOklab = (red: number, green: number, blue: number): Oklab => {
  const long = Math.cbrt(
    0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue,
  );
  const medium = Math.cbrt(
    0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue,
  );
  const short = Math.cbrt(
    0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue,
  );
  return {
    a: 1.9779984951 * long - 2.428592205 * medium + 0.4505937099 * short,
    b: 0.0259040371 * long + 0.7827717662 * medium - 0.808675766 * short,
    l: 0.2104542553 * long + 0.793617785 * medium - 0.0040720468 * short,
  };
};

const oklabToLinear = ({ a, b, l }: Oklab) => {
  const long = (l + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const medium = (l - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const short = (l - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * long - 3.3077115913 * medium + 0.2309699292 * short,
    -1.2684380046 * long + 2.6097574011 * medium - 0.3413193965 * short,
    -0.0041960863 * long - 0.7034186147 * medium + 1.707614701 * short,
  ];
};

const lchToLab = ({ c, h, l }: Oklch): Oklab => {
  const angle = (h * Math.PI) / 180;
  return { a: c * Math.cos(angle), b: c * Math.sin(angle), l };
};

const labToLch = ({ a, b, l }: Oklab): Oklch => ({
  c: Math.hypot(a, b),
  h: ((Math.atan2(b, a) * 180) / Math.PI + 360) % 360,
  l,
});

export const hexToOklch = (hex: string): Oklch => {
  const digits = hex.replace("#", "").trim();
  const expanded =
    digits.length === 3
      ? digits
          .split("")
          .map((digit) => digit + digit)
          .join("")
      : digits;
  const value = Number.parseInt(expanded.slice(0, 6), 16);
  const safe = Number.isNaN(value) ? 0 : value;
  return labToLch(
    linearToOklab(
      toLinear(((safe >> 16) & 255) / 255),
      toLinear(((safe >> 8) & 255) / 255),
      toLinear((safe & 255) / 255),
    ),
  );
};

const inGamut = (channels: number[]) =>
  channels.every((channel) => channel >= -0.0001 && channel <= 1.0001);

/**
 * OKLCH to sRGB, keeping lightness and hue. A colour outside the display's
 * reach loses chroma until it fits rather than having its channels clipped,
 * which would drag both hue and lightness off with them.
 */
const oklchToHex = (color: Oklch) => {
  let channels = oklabToLinear(lchToLab(color));
  if (!inGamut(channels)) {
    let low = 0;
    let high = color.c;
    for (let step = 0; step < GAMUT_STEPS; step += 1) {
      const middle = (low + high) / 2;
      if (inGamut(oklabToLinear(lchToLab({ ...color, c: middle }))))
        low = middle;
      else high = middle;
    }
    channels = oklabToLinear(lchToLab({ ...color, c: low }));
  }
  return `#${channels
    .map((channel) =>
      Math.round(clamp(toGamma(clamp(channel, 0, 1)), 0, 1) * 255)
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`.toUpperCase();
};

const asChosen = (color: Oklch): Chosen => ({
  chroma: color.c,
  hue: color.h,
  lab: lchToLab(color),
});

const labDistance = (one: Oklab, other: Oklab) =>
  Math.hypot(one.l - other.l, one.a - other.a, one.b - other.b);

/** Hue separation in [0, 1], counted only when both colours have a hue to
 * speak of. Two colours can sit far apart in OKLab and still read as the same
 * family, so the family costs a candidate part of its score. */
const hueSeparation = (one: Chosen, other: Chosen) => {
  if (one.chroma < GREY_CHROMA || other.chroma < GREY_CHROMA) return 0;
  const difference = Math.abs(one.hue - other.hue) % 360;
  return Math.min(difference, 360 - difference) / 180;
};

/** How far a candidate sits from the nearest colour already in the palette. */
const separation = (candidate: Chosen, chosen: Chosen[], hueWeight: number) =>
  chosen.reduce(
    (nearest, other) =>
      Math.min(
        nearest,
        labDistance(candidate.lab, other.lab) +
          hueWeight * hueSeparation(candidate, other),
      ),
    Number.POSITIVE_INFINITY,
  );

const randomCandidate = (band: Band, hue: number) => ({
  c: randomInRange(band.chroma),
  h: band.sameHue ? hue : randomInRange({ max: 360, min: 0 }),
  l: randomInRange(band.lightness),
});

/**
 * Farthest-point selection: every slot draws a handful of candidates from the
 * band and keeps the one sitting furthest from everything already picked,
 * locked colours included. That is what makes a palette read as varied rather
 * than as a walk along a curve.
 */
const pickColors = ({
  band,
  count,
  hue,
  seeds,
}: {
  band: Band;
  count: number;
  hue: number;
  seeds: Chosen[];
}) => {
  const hueWeight = band.sameHue ? 0 : HUE_WEIGHT;
  const chosen = [...seeds];
  const picked: string[] = [];
  for (let index = 0; index < count; index += 1) {
    let best = randomCandidate(band, hue);
    if (chosen.length > 0) {
      let bestScore = separation(asChosen(best), chosen, hueWeight);
      for (let draw = 1; draw < CANDIDATES; draw += 1) {
        const candidate = randomCandidate(band, hue);
        const score = separation(asChosen(candidate), chosen, hueWeight);
        if (score > bestScore) {
          best = candidate;
          bestScore = score;
        }
      }
    }
    chosen.push(asChosen(best));
    picked.push(oklchToHex(best));
  }
  return picked;
};

export function generatePalette(mode: PaletteMode, count: number) {
  return pickColors({
    band: bands[mode],
    count: Math.max(0, count),
    hue: randomInRange({ max: 360, min: 0 }),
    seeds: [],
  });
}

export function generatePaletteFromLocked({
  colors,
  locked,
  mode,
}: {
  colors: string[];
  locked: boolean[];
  mode: PaletteMode;
}) {
  const lockedColors = colors
    .filter((_, index) => locked[index])
    .map((color) => hexToOklch(color));
  const generated = pickColors({
    band: bands[mode],
    count: colors.length - lockedColors.length,
    // Shades keep whatever hue is already on screen, so a locked colour is not
    // left as the odd one out.
    hue: lockedColors[0]?.h ?? randomInRange({ max: 360, min: 0 }),
    seeds: lockedColors.map((color) => asChosen(color)),
  });
  let generatedIndex = 0;
  return colors.map((color, index) => {
    if (locked[index]) return color;
    const next = generated[generatedIndex] ?? color;
    generatedIndex += 1;
    return next;
  });
}
