// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ISLAND } from "../../../primitives/island/island-geometry";
import poster from "../../../site/public/recordings/edit.jpg";
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
    destination: "screenshot",
    isShown: true,
    onClose: fn(),
    onEdit: fn(),
    onHoverChange: fn(),
    onSave: fn(),
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
    featureStage({ height: 320, width: 720 }),
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

export const Screenshot: Story = {};

export const Reference: Story = {
  args: { destination: "reference" },
};

export const OpenedBySystemKeys: Story = {
  args: { note: <SystemKeysNote keys="Super+Shift+KeyS" layout="section" /> },
};

export const Copied: Story = {
  args: { copied: { height: 180, isSaved: false, src: poster, width: 320 } },
};

export const Saved: Story = {
  args: { copied: { height: 180, isSaved: true, src: poster, width: 320 } },
};
