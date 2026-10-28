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

import { counterIsland, type CounterIslandProps } from "./counter-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function CounterIsland({
  isShown,
  ...props
}: CounterIslandProps & EditorIslandStoryProps) {
  return <EditorIslandStage content={counterIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    id: "counter",
    isShown: true,
    onDelete: fn(),
    onRemoveColor: fn(),
    onSaveColor: fn(),
    onStyleChange: fn(),
    savedColors: [],
    style: { angle: 0, color: "#FFCC00", isAnimated: true, size: 36 },
  },
  component: CounterIsland,
  decorators: [featureStage({ height: 320, width: 520 })],
  // Every choice changes the story's args, as it would the counter.
  render: function Render(args) {
    const [, updateArgs] = useArgs<CounterIslandProps>();
    return (
      <CounterIsland
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
  title: "Features/Editor/Island/Annotation/Counter",
} satisfies Meta<typeof CounterIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// A screenshot is one instant, so there is nothing to pop in over.
export const Screenshot: Story = { args: { canAnimate: false } };
