// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";

import { SoundKindChooser } from "./sound-kind-chooser";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** The timeline under a picture in a 760 by 480 editor. */
const area = { height: 140, left: 64, top: 324, width: 680 };

const meta = {
  argTypes: { onChoose: { control: false } },
  args: {
    adding: { area, count: 1, hasSongRoom: true, isCarried: false, over: null },
    onChoose: fn(),
  },
  component: SoundKindChooser,
  decorators: [
    (Story) => (
      <div className="dark relative size-full text-fg">
        <Story />
      </div>
    ),
    featureStage({ height: 480, width: 760 }),
  ],
  title: "Features/Editor/Island/Sound Kind",
} satisfies Meta<typeof SoundKindChooser>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const DraggedOverEffect: Story = {
  args: {
    adding: {
      area,
      count: 1,
      hasSongRoom: true,
      isCarried: true,
      over: "effect",
    },
  },
};

export const NoSongRoom: Story = {
  args: {
    adding: {
      area,
      count: 1,
      hasSongRoom: false,
      isCarried: false,
      over: null,
    },
  },
};
