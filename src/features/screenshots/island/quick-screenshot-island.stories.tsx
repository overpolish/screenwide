// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ISLAND } from "../../../primitives/island/island-geometry";
import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";
import { StoryNotch } from "../../../storybook/story-notch";
import { SystemKeysNote } from "../../island/system-keys-note";

import { QuickScreenshotIsland } from "./quick-screenshot-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    destination: "editor",
    isShown: true,
    onClose: fn(),
  },
  component: QuickScreenshotIsland,
  decorators: [
    (Story, { args, globals }) =>
      Story({
        args: {
          ...args,
          notch: globals.notch === "off" ? null : PREVIEW_NOTCH,
        },
      }),
    featureStage({ height: 160, width: 720 }),
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
        <QuickScreenshotIsland {...args} />
      </div>
      {args.notch ? <StoryNotch notch={args.notch} /> : null}
    </div>
  ),
  title: "Features/Screenshots/Island/Quick Screenshot",
} satisfies Meta<typeof QuickScreenshotIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ToEditor: Story = {};

export const ToClipboard: Story = {
  args: { destination: "clipboard" },
};

export const OpenedBySystemKeys: Story = {
  args: { note: <SystemKeysNote keys="Super+Shift+KeyS" /> },
};
