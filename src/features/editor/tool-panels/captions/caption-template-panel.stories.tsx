// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";

import {
  STORY_CAPTION_STYLES,
  STORY_CAPTION_TEMPLATES,
} from "./caption-template-fixtures";
import { CaptionTemplatePanel } from "./caption-template-panel";
import { CAPTION_TEMPLATE_PANEL_SIZE } from "./caption-template-panel-size";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    content: {
      kind: "caption-templates",
      names: STORY_CAPTION_TEMPLATES.names,
      selected: "editorial",
      styles: STORY_CAPTION_STYLES,
    },
    focusContents: false,
    id: "caption-templates",
  },
  component: CaptionTemplatePanel,
  decorators: [
    (Story, context) => (
      // As tall as the panel grows before it scrolls, which every template
      // here reaches.
      <FeatureStoryStage
        height={CAPTION_TEMPLATE_PANEL_SIZE.height}
        viewMode={context.viewMode}
        width={CAPTION_TEMPLATE_PANEL_SIZE.width}
      >
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Caption Templates",
} satisfies Meta<typeof CaptionTemplatePanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The templates opened from the Style tab's Template button, scrolled to
 * the chosen one. The previews are drawn by the compositor, which a story
 * has none of, so each tile shows its name alone. */
export const Chosen: Story = {};
