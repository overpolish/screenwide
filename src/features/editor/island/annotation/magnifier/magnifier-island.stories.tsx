// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../../editor-island-story-meta";

import { magnifierIsland, type MagnifierIslandProps } from "./magnifier-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function MagnifierIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & MagnifierIslandProps) {
  return (
    <EditorIslandStage content={magnifierIsland(props)} isShown={isShown} />
  );
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    id: "magnifier",
    isShown: true,
    onDelete: fn(),
    onStyleChange: fn(),
    style: { isAnimated: true, radius: 50 },
  },
  component: MagnifierIsland,
  decorators: [featureStage({ height: 260, width: 520 })],
  // Every choice changes the story's args, as it would the magnifier.
  render: function Render(args) {
    const [, updateArgs] = useArgs<MagnifierIslandProps>();
    return (
      <MagnifierIsland
        {...args}
        onStyleChange={(style) => {
          args.onStyleChange(style);
          updateArgs({ style: { ...args.style, ...style } });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Annotation/Magnifier",
} satisfies Meta<typeof MagnifierIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// A screenshot is one instant, so there is nothing to grow in over.
export const Screenshot: Story = { args: { canAnimate: false } };
