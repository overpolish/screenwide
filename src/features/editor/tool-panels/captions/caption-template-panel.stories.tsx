// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";

import { STORY_CAPTION_STYLES } from "./caption-template-fixtures";
import { CaptionTemplatePanel } from "./caption-template-panel";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    content: {
      kind: "caption-templates",
      selected: "editorial",
      styles: STORY_CAPTION_STYLES,
    },
    focusContents: false,
    id: "caption-templates",
  },
  component: CaptionTemplatePanel,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={380}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Caption Templates",
} satisfies Meta<typeof CaptionTemplatePanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The templates opened from the Style tab's Template button. The previews
 * are drawn by the compositor, which a story has none of, so each tile
 * shows its name alone. */
export const Chosen: Story = {};
