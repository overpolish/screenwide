// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { expect, userEvent, waitFor, within } from "storybook/test";

import { t } from "../../../i18n/i18n";

import { annotateIslandMeta } from "./annotate-island-story-meta";

import type { AnnotateIsland } from "./annotate-island";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...annotateIslandMeta,
  args: { ...annotateIslandMeta.args, initialSection: "colors" },
  title: "Features/Annotate/Island/Colours",
} satisfies Meta<typeof AnnotateIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Undo and clear are done with the colours: each press folds them away to
// the pen, and they open again from the colour.
export const ClosedByTheRow: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement);
    const color = await canvas.findByRole("button", {
      name: t("annotation-color"),
    });
    for (const name of [t("annotate-undo-label"), t("annotate-clear-label")]) {
      if (color.getAttribute("aria-expanded") !== "true")
        await userEvent.click(color);
      await waitFor(() =>
        expect(
          canvas.getByRole("radiogroup", { name: t("annotation-color") }),
        ).toBeVisible(),
      );
      await userEvent.click(canvas.getByRole("button", { name }));
      await expect(color).toHaveAttribute("aria-expanded", "false");
      await waitFor(() =>
        expect(
          canvas.queryByRole("radiogroup", { name: t("annotation-color") }),
        ).toBeNull(),
      );
    }
  },
};

// `count` colours round the colour wheel, in two lightnesses.
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
