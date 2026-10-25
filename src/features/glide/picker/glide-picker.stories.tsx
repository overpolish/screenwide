// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../../storybook/feature-story-stage";

import {
  DISPLAYS,
  ELSEWHERE,
  MANY_SPACES,
  SPACES,
} from "./glide-picker-preview";
import { GlidePickerStage } from "./glide-picker-stage";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    hasIcon: {
      control: "boolean",
      table: { defaultValue: { summary: "true" } },
    },
  },
  args: { hasIcon: true, kind: "spaces", screens: SPACES },
  component: GlidePickerStage,
  decorators: [featureStage({ height: 280, width: 640 })],
  parameters: { controls: { include: ["hasIcon"] }, layout: "fullscreen" },
  title: "Features/Glide Island/Picker",
} satisfies Meta<typeof GlidePickerStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Spaces: Story = { args: { pick: "spaces" } };

export const Displays: Story = {
  args: { kind: "displays", pick: "displays", screens: DISPLAYS },
};

export const ManySpaces: Story = {
  args: { screens: MANY_SPACES, state: ELSEWHERE },
};
