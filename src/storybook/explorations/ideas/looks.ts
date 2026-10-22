// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A project's look: one family and one accent, chosen once, from which every
// annotation takes its style. Ink is not a colour anyone picks: it is black
// on a light picture and white on a dark one.

export type LookId = "editorial" | "playful" | "product";

export type Look = {
  /** The one colour that stands out. */
  accent: string;
  /** What else the swatches offer: highlighter, a second accent. */
  extra: string;
  id: LookId;
  ink: string;
  label: string;
  /** Who it looks like, in a few words. */
  like: string;
};

export const LOOKS: Look[] = [
  {
    accent: "#2f6bff",
    extra: "#ffffff",
    id: "product",
    ink: "#1c1c1e",
    label: "Product",
    like: "Launch videos: ink, hairlines, focus",
  },
  {
    accent: "#d93025",
    extra: "#ffe14d",
    id: "editorial",
    ink: "#1c1c1e",
    label: "Editorial",
    like: "Explainers: pen and highlighter",
  },
  {
    accent: "#ff6b3d",
    extra: "#7c5cff",
    id: "playful",
    ink: "#1c1c1e",
    label: "Playful",
    like: "Tutorials: bold, rounded, bouncy",
  },
];

export const lookById = (id: LookId) =>
  LOOKS.find((look) => look.id === id) ?? LOOKS[0];

export type Size = "large" | "medium" | "small";

export const SIZE_SCALE: Record<Size, number> = {
  large: 1.35,
  medium: 1,
  small: 0.75,
};

export type Pen = "arrow" | "counter" | "focus";

/** What a pen's colour choice means: ink follows the picture. */
export type Swatch = "accent" | "extra" | "ink";

export const swatchColor = (look: Look, swatch: Swatch) =>
  swatch === "ink" ? look.ink : swatch === "accent" ? look.accent : look.extra;

/** A loop drawn round a point by hand: a little over one turn, wandering. */
export const handLoop = ({
  centre,
  rx,
  ry,
  seed,
}: {
  centre: { x: number; y: number };
  rx: number;
  ry: number;
  seed: number;
}) => {
  const points: string[] = [];
  for (let index = 0; index <= 56; index += 1) {
    const u = index / 56;
    const angle = -2.4 + seed * 0.3 + u * 1.12 * 2 * Math.PI;
    const wander = 1 + 0.05 * Math.sin(2 * angle + seed) + 0.08 * u;
    const x = centre.x + Math.cos(angle) * rx * wander;
    const y = centre.y + Math.sin(angle) * ry * wander;
    points.push(`${index === 0 ? "M" : "L"}${x.toFixed(1)} ${y.toFixed(1)}`);
  }
  return points.join(" ");
};

/** How a mark draws on: quick to start, gentle to land. */
export const draw = { duration: 0.45, ease: [0.2, 0.7, 0.2, 1] } as const;

/** How a mark pops in: a spring with a little bounce. */
export const pop = { bounce: 0.35, duration: 0.4, type: "spring" } as const;
