// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { islandMeta } from "./island-story-meta";

import type { IslandStage } from "./island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

// The stage shows and hides the island on its own here, so whether it is
// shown is left to it rather than offered in Controls.
const meta = {
  ...islandMeta,
  args: { ...islandMeta.args, appears: true, isShown: undefined },
  parameters: {
    ...islandMeta.parameters,
    controls: {
      include: islandMeta.parameters.controls.include.filter(
        (name) => name !== "isShown",
      ),
    },
  },
  title: "Features/Recording Island/Appearing",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Appear: Story = {};
