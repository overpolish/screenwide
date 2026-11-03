// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ISLAND } from "../../../primitives/island/island-geometry";
import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";
import { StoryNotch } from "../../../storybook/story-notch";

import { ScrollingCaptureIsland } from "./scrolling-capture-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    isShown: true,
    onCancel: fn(),
    state: { escapeClaimed: true, phase: "capturing" },
  },
  component: ScrollingCaptureIsland,
  decorators: [
    (Story, { args, globals }) =>
      Story({
        args: {
          ...args,
          notch: globals.notch === "off" ? null : PREVIEW_NOTCH,
        },
      }),
    featureStage({ height: 240, width: 720 }),
  ],
  parameters: {
    controls: { include: ["isShown"] },
    layout: "fullscreen",
    notch: true,
  },
  render: (args) => (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        style={{ top: args.notch ? 0 : ISLAND.screenGap }}
      >
        <ScrollingCaptureIsland {...args} />
      </div>
      {args.notch ? <StoryNotch notch={args.notch} /> : null}
    </div>
  ),
  title: "Features/Screenshots/Island/Scrolling Capture",
} satisfies Meta<typeof ScrollingCaptureIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Working: Story = {
  args: { state: { escapeClaimed: true, phase: "working" } },
};

export const Capturing: Story = {};

export const Stitching: Story = {
  args: { state: { escapeClaimed: true, phase: "stitching" } },
};

// Escape could not be claimed: the island names no key, and Cancel is the
// one way out.
export const NotCancellableByEscape: Story = {
  args: { state: { escapeClaimed: false, phase: "capturing" } },
};

export const Finishing: Story = {
  args: { state: { escapeClaimed: true, phase: "finishing" } },
};
