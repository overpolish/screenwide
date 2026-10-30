// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { featureStage } from "../../../../storybook/feature-story-stage";

import { CutBlade } from "./cut-blade";

import type { Meta, StoryObj } from "@storybook/react-vite";

const WIDTH = 640;

/** The blade on a row of the strip's height, following the pointer across
 * it from where Controls put it. */
function LiveBlade({ x: initial }: { x: number }) {
  const [x, setX] = useState(initial);
  return (
    <div
      className="relative mt-3 h-11"
      dir="ltr"
      onPointerMove={(event) => {
        const box = event.currentTarget.getBoundingClientRect();
        setX((event.clientX - box.left) / box.width);
      }}
      style={{ width: WIDTH }}
    >
      <CutBlade x={x} />
    </div>
  );
}

const meta = {
  argTypes: {
    x: { control: { max: 1, min: 0, step: 0.005, type: "range" } },
  },
  args: { x: 0.3 },
  component: CutBlade,
  decorators: [featureStage({ width: WIDTH })],
  render: (args) => <LiveBlade key={args.x} {...args} />,
  title: "Features/Editor/Timeline/Cut Blade",
} satisfies Meta<typeof CutBlade>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
