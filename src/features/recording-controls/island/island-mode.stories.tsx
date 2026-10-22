// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  PREVIEW_MANY_DISPLAYS,
  PREVIEW_REGIONS,
  PREVIEW_WINDOWS,
} from "./island-preview";
import { islandMeta } from "./island-story-meta";

import type { IslandStage } from "./island-stage";
import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  ...islandMeta,
  title: "Features/Recording Controls/Island/Mode",
} satisfies Meta<typeof IslandStage>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Screen: Story = { args: { initialMode: "screen" } };

export const ScreenFiveDisplays: Story = {
  args: { displays: PREVIEW_MANY_DISPLAYS, initialMode: "screen" },
};

export const Window: Story = { args: { initialMode: "window" } };

export const WindowChosen: Story = {
  args: { chosenWindow: PREVIEW_WINDOWS[0], initialMode: "window" },
};

export const Region: Story = { args: { initialMode: "region" } };

export const RegionChosen: Story = {
  args: { chosenRegion: PREVIEW_REGIONS[0], initialMode: "region" },
};

export const RegionAcrossDisplays: Story = {
  args: { chosenRegion: PREVIEW_REGIONS[1], initialMode: "region" },
};

export const NoScreen: Story = { args: { initialMode: "none" } };

export const NoScreenInputsOff: Story = {
  args: {
    camera: { isOn: false },
    initialMode: "none",
    microphone: { isOn: false },
    systemAudio: { isOn: false },
  },
};
