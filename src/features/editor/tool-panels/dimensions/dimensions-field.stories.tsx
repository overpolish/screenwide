// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";

import { DimensionsField } from "./dimensions-field";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isDisabled: {
      control: "boolean",
      table: { defaultValue: { summary: "false" } },
    },
  },
  args: {
    height: 1080,
    isDisabled: false,
    onChange: fn(),
    width: 1920,
  },
  component: DimensionsField,
  decorators: [featureStage({ width: 240 })],
  parameters: {
    controls: { include: ["height", "isDisabled", "width"] },
    layout: "fullscreen",
  },
  // Edits land in the args, as the Frame island's canvas size takes them, so
  // the kept ratio can be tried: change the width and the height follows.
  render: function Render(args) {
    const [, updateArgs] = useArgs();
    return (
      <DimensionsField
        {...args}
        onChange={(change) => {
          args.onChange(change);
          updateArgs(change);
        }}
      />
    );
  },
  title: "Features/Editor/Tool Panels/Dimensions Field",
} satisfies Meta<typeof DimensionsField>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Disabled: Story = {
  args: { isDisabled: true },
  parameters: { controls: { disable: true } },
};
