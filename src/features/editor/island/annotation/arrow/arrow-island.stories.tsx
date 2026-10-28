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

import { arrowIsland, type ArrowIslandProps } from "./arrow-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function ArrowIsland({
  isShown,
  ...props
}: ArrowIslandProps & EditorIslandStoryProps) {
  return <EditorIslandStage content={arrowIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    id: "arrow",
    isShown: true,
    onDelete: fn(),
    onRemoveColor: fn(),
    onReverse: fn(),
    onSaveColor: fn(),
    onStyleChange: fn(),
    savedColors: [],
    style: { color: "#FFCC00", head: "end", isAnimated: true, width: 12 },
  },
  component: ArrowIsland,
  decorators: [featureStage({ height: 360, width: 520 })],
  // Every choice changes the story's args, as it would the arrow.
  render: function Render(args) {
    const [, updateArgs] = useArgs<ArrowIslandProps>();
    return (
      <ArrowIsland
        {...args}
        onRemoveColor={(color) => {
          args.onRemoveColor(color);
          updateArgs({
            savedColors: args.savedColors.filter((kept) => kept !== color),
          });
        }}
        onSaveColor={(color) => {
          args.onSaveColor(color);
          if (!args.savedColors.includes(color))
            updateArgs({ savedColors: [...args.savedColors, color] });
        }}
        onStyleChange={(style) => {
          args.onStyleChange(style);
          updateArgs({ style: { ...args.style, ...style } });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Annotation/Arrow",
} satisfies Meta<typeof ArrowIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// A screenshot is one instant, so there is nothing to draw in over.
export const Screenshot: Story = { args: { canAnimate: false } };
