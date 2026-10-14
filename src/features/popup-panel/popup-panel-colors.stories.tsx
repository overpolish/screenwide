// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../storybook/feature-story-stage";

import { PopupPanelColors } from "./popup-panel-colors";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    content: { kind: "colors", savedColors: ["#7fffd4"], value: "#0088ff" },
    id: "caption-color:accent",
  },
  component: PopupPanelColors,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={340}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Popup Panel/Colours",
} satisfies Meta<typeof PopupPanelColors>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The colour grid a caption's colour well opens, with one colour of your
 * own after the palette. */
export const Chosen: Story = {};
