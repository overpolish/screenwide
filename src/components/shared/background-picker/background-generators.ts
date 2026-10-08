// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

/**
 * One way a mesh background is painted.
 *
 * The classic generator is a composition: a palette laid out as blobs, which
 * the picker can rearrange. The rest are procedural, and take nothing but
 * their colours and a seed, so a tile of one of those is the palette under a
 * picture the shader draws.
 */
export type BackgroundGenerator = {
  /** How many colours this generator reads. A palette longer or shorter than
   * this is padded from, or cut down to, the defaults below. */
  colorCount: number;
  defaultColors: string[];
  id: string;
  /** The generator's name in the app's language. */
  name: () => string;
};

/** The generator every mesh falls back to: the composition one. */
export const DEFAULT_GENERATOR_ID = "mesh";

/** The generators on offer, in the order their tiles are shown. */
export const BACKGROUND_GENERATORS: BackgroundGenerator[] = [
  {
    colorCount: 4,
    defaultColors: ["#1B1F3B", "#4F46E5", "#9333EA", "#38BDF8"],
    id: "mesh",
    name: () => t("background-generator-mesh"),
  },
  {
    colorCount: 4,
    defaultColors: ["#05040A", "#46266E", "#C053A6", "#FFD0A8"],
    id: "silk",
    name: () => t("background-generator-silk"),
  },
  {
    colorCount: 3,
    defaultColors: ["#8FE3FF", "#FF9BE3", "#C9B8FF"],
    id: "aurora",
    name: () => t("background-generator-aurora"),
  },
  {
    colorCount: 4,
    defaultColors: ["#0A0E2A", "#1E4F9C", "#3FC7E8", "#E8FBFF"],
    id: "fluid",
    name: () => t("background-generator-fluid"),
  },
  {
    colorCount: 3,
    defaultColors: ["#8C00FF", "#FF0080", "#00FFFF"],
    id: "gentle",
    name: () => t("background-generator-gentle"),
  },
  {
    colorCount: 4,
    defaultColors: ["#0A2A5E", "#1E7FA8", "#6FD0C8", "#B8A0E8"],
    id: "currents",
    name: () => t("background-generator-currents"),
  },
  {
    colorCount: 4,
    defaultColors: ["#FF5FA2", "#FFD166", "#4CC9F0", "#43E197"],
    id: "paint",
    name: () => t("background-generator-paint"),
  },
  {
    colorCount: 4,
    defaultColors: ["#2A1B4A", "#7A3FA0", "#F26D9C", "#FFD166"],
    id: "ribbons",
    name: () => t("background-generator-ribbons"),
  },
  {
    colorCount: 4,
    defaultColors: ["#3B2A1F", "#6B4A2F", "#A9744F", "#D8B08C"],
    id: "strata",
    name: () => t("background-generator-strata"),
  },
];

/**
 * The generator a mesh names, or the composition one.
 *
 * A background saved by an older build carries no generator at all, and a
 * build that no longer ships a generator still has to draw what was saved
 * under it, so an unknown name is the classic mesh rather than an error.
 */
export const backgroundGenerator = (id: string): BackgroundGenerator =>
  BACKGROUND_GENERATORS.find((generator) => generator.id === id) ??
  BACKGROUND_GENERATORS[0];
