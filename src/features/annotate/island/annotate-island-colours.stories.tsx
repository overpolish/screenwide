// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { annotateIslandMeta } from "./annotate-island-story-meta";

import type { AnnotateIsland } from "./annotate-island";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...annotateIslandMeta,
  args: { ...annotateIslandMeta.args, initialSection: "colors" },
  title: "Features/Annotate Island/Colours",
} satisfies Meta<typeof AnnotateIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** `count` colours of your own around the colour wheel, in two lightnesses,
 * as `#rrggbb`. */
const savedAround = (count: number) =>
  Array.from({ length: count }, (_, index) => {
    const hue = (index * 360) / count;
    const lightness = index % 2 === 0 ? 0.55 : 0.4;
    const channel = (shift: number) => {
      const turn = (shift + hue / 30) % 12;
      const amount =
        lightness -
        0.6 *
          Math.min(lightness, 1 - lightness) *
          Math.max(-1, Math.min(turn - 3, 9 - turn, 1));
      return Math.round(amount * 255)
        .toString(16)
        .padStart(2, "0");
    };
    return `#${channel(0)}${channel(8)}${channel(4)}`;
  });

export const MultipleRows: Story = {
  args: { savedColors: savedAround(46) },
};
