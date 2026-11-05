// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../storybook/feature-story-stage";

import { ReferenceIsland, type ReferenceIslandProps } from "./reference-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    isCopied: false,
    isShown: true,
    onClose: fn(),
    onCopy: fn(),
    onHidden: fn(),
    onOpacityChange: fn(),
    onOpenInEditor: fn(),
    onResetSize: fn(),
    onToOrigin: fn(),
    opacity: 100,
    returnsToOrigin: true,
  },
  component: ReferenceIsland,
  decorators: [featureStage({ height: 120, width: 360 })],
  parameters: { controls: { include: ["isShown"] } },
  // The opacity pills move the story's opacity, as they would the reference's.
  render: function Render(args) {
    const [, updateArgs] = useArgs<ReferenceIslandProps>();
    return (
      <div className="pt-inset" dir="ltr">
        <ReferenceIsland
          {...args}
          onOpacityChange={(opacity) => {
            args.onOpacityChange(opacity);
            updateArgs({ opacity });
          }}
        />
      </div>
    );
  },
  title: "Features/Reference/Island",
} satisfies Meta<typeof ReferenceIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Copied: Story = { args: { isCopied: true } };

export const Dropped: Story = { args: { returnsToOrigin: false } };
