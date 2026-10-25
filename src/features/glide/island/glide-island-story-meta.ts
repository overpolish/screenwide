// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../../storybook/feature-story-stage";

import { GlideIslandStage } from "./glide-island-stage";

import type { Meta } from "@storybook/react-vite";

// What every Glide island story shares: the stage round where the gesture
// began, and whether the glided app's icon resolves. Each story sets the
// state it shows, the gesture it plays or the destinations it lays side by
// side.

type StageSize = { height: number; width: number };

export const glideIslandMeta = {
  argTypes: {
    hasIcon: {
      control: "boolean",
      table: { defaultValue: { summary: "true" } },
    },
  },
  args: { hasIcon: true },
  component: GlideIslandStage,
  decorators: [
    featureStage(
      ({ parameters }) =>
        (parameters.stage as StageSize | undefined) ?? {
          height: 240,
          width: 360,
        },
    ),
  ],
  parameters: { controls: { include: ["hasIcon"] }, layout: "fullscreen" },
} satisfies Meta<typeof GlideIslandStage>;
