// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Meta, StoryObj } from "@storybook/react-vite";

import { displayThumbnail } from "../../storybook/display-thumbnail";
import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { MonitorSelector } from "./monitor-selector";
import { MonitorDetails } from "./types";

const monitors: MonitorDetails[] = [
  {
    id: 1,
    isBuiltin: false,
    isPrimary: false,
    layoutPosition: { x: 0, y: 0 },
    layoutSize: { height: 1080, width: 1920 },
    name: "24G2W1G4",
    physicalPosition: { x: 0, y: 0 },
    physicalSize: { height: 1080, width: 1920 },
    position: { x: 0, y: 0 },
    scaleFactor: 1,
    size: { height: 1080, width: 1920 },
  },
  {
    id: 2,
    isBuiltin: true,
    isPrimary: true,
    layoutPosition: { x: 1920, y: 49 },
    layoutSize: { height: 982, width: 1512 },
    name: "Built-in Retina Display",
    physicalPosition: { x: 1920, y: 49 },
    physicalSize: { height: 1964, width: 3024 },
    position: { x: 320, y: 1080 },
    scaleFactor: 2,
    size: { height: 982, width: 1512 },
  },
];

/** Stands in for the captured still of each display. */
const thumbnails: Record<number, string> = {
  1: displayThumbnail("#2f7d63", "#123f36", 16 / 9),
  2: displayThumbnail("#3b6fd4", "#8f5bd6"),
};

const meta = {
  args: {
    focusContents: false,
    monitors,
    onCommit: () => undefined,
    onSelect: () => undefined,
    selectedMonitor: monitors[1],
    thumbnails,
  },
  component: MonitorSelector,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage height={250} viewMode={context.viewMode} width={500}>
        <div className="window-surface p-section flex h-full w-full items-center justify-center">
          <Story />
        </div>
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Monitor Selection",
} satisfies Meta<typeof MonitorSelector>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Before the stills have been captured, each display keeps its plain fill. */
export const WithoutThumbnails: Story = {
  args: { thumbnails: {} },
};
