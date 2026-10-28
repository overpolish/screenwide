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

import { highlightIsland, type HighlightIslandProps } from "./highlight-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function HighlightIsland({
  isShown,
  ...props
}: HighlightIslandProps & EditorIslandStoryProps) {
  return (
    <EditorIslandStage content={highlightIsland(props)} isShown={isShown} />
  );
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    id: "highlight",
    isShown: true,
    onDelete: fn(),
    onRemoveColor: fn(),
    onSaveColor: fn(),
    onStyleChange: fn(),
    savedColors: [],
    style: {
      color: "#FFCC00",
      isAnimated: true,
      isHandDrawn: false,
      isManual: false,
    },
  },
  component: HighlightIsland,
  decorators: [featureStage({ height: 360, width: 520 })],
  // Every choice changes the story's args, as it would the highlight.
  render: function Render(args) {
    const [, updateArgs] = useArgs<HighlightIslandProps>();
    return (
      <HighlightIsland
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
  title: "Features/Editor/Island/Annotation/Highlight",
} satisfies Meta<typeof HighlightIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// Drawn by hand, its glyph shows the waved band.
export const HandDrawn: Story = {
  args: {
    style: {
      color: "#FFCC00",
      isAnimated: true,
      isHandDrawn: true,
      isManual: false,
    },
  },
};

// A screenshot is one instant, so there is nothing to draw in over.
export const Screenshot: Story = { args: { canAnimate: false } };
