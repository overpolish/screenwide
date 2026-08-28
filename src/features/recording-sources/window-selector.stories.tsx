// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { WindowDetails } from "./types";
import { WindowSelector } from "./window-selector";

const windows: WindowDetails[] = [
  [101, "Safari", "Screenwide — UI rework"],
  [102, "Finder", "Applications"],
  [103, "Final Cut Pro", "Screen Recording"],
  [104, "Terminal", "screenwide"],
  [105, "Notes", "UI ideas"],
  [106, "Music", "Now Playing"],
  [107, "Messages", "Messages"],
  [108, "System Settings", "Displays"],
].map(([id, appName, title]) => ({
  appIconPath: null,
  appName: String(appName),
  id: Number(id),
  pid: Number(id) + 1_000,
  position: { x: 100, y: 100 },
  size: { height: 720, width: 1280 },
  thumbnailPath: null,
  title: String(title),
}));

const meta = {
  args: {
    error: null,
    isLoading: false,
    onSelect: () => undefined,
    selectedWindow: windows[0],
    windows,
  },
  component: WindowSelector,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage height={250} viewMode={context.viewMode} width={500}>
        <div className="window-surface p-section h-full w-full overflow-hidden">
          <Story />
        </div>
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Window Selection",
} satisfies Meta<typeof WindowSelector>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Loading: Story = {
  args: {
    isLoading: true,
  },
};

export const Empty: Story = {
  args: {
    selectedWindow: null,
    windows: [],
  },
};

export const Error: Story = {
  args: {
    error: "Windows could not be loaded.",
    selectedWindow: null,
    windows: [],
  },
};
