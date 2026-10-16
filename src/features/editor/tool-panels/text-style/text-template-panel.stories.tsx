// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";

import {
  STORY_TEXT_STYLES,
  STORY_TEXT_TEMPLATES,
} from "./text-template-fixtures";
import { TextTemplatePanel } from "./text-template-panel";
import { TEXT_TEMPLATE_PANEL_SIZE } from "./text-template-panel-size";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    content: {
      kind: "text-templates",
      names: STORY_TEXT_TEMPLATES.names,
      selected: "editorial",
      styles: STORY_TEXT_STYLES,
    },
    focusContents: false,
    id: "text-templates",
  },
  component: TextTemplatePanel,
  decorators: [
    (Story, context) => (
      // As tall as the panel grows before it scrolls, which every template
      // here reaches.
      <FeatureStoryStage
        height={TEXT_TEMPLATE_PANEL_SIZE.height}
        viewMode={context.viewMode}
        width={TEXT_TEMPLATE_PANEL_SIZE.width}
      >
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Text Templates",
} satisfies Meta<typeof TextTemplatePanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The templates opened from the Style tab's Template button, scrolled to
 * the chosen one. The previews are drawn by the compositor, which a story
 * has none of, so each tile shows its name alone. */
export const Chosen: Story = {};
