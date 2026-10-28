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

import { redactionIsland, type RedactionIslandProps } from "./redaction-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function RedactionIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & RedactionIslandProps) {
  return (
    <EditorIslandStage content={redactionIsland(props)} isShown={isShown} />
  );
}

const STYLE: RedactionIslandProps["style"] = {
  color: "#000000",
  isAnimated: false,
  radius: 0,
  redaction: "pixelate",
  strength: 3,
  width: 6,
};

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    id: "redaction",
    isShown: true,
    onDelete: fn(),
    onRemoveColor: fn(),
    onSaveColor: fn(),
    onShuffle: fn(),
    onStyleChange: fn(),
    savedColors: [],
    style: STYLE,
  },
  component: RedactionIsland,
  decorators: [featureStage({ height: 380, width: 560 })],
  // Every choice changes the story's args, as it would the redaction.
  render: function Render(args) {
    const [, updateArgs] = useArgs<RedactionIslandProps>();
    return (
      <RedactionIsland
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
  title: "Features/Editor/Island/Annotation/Redaction",
} satisfies Meta<typeof RedactionIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Pixelated: Story = {};

// Blurred, it offers its strength, and is only a look.
export const Blurred: Story = {
  args: { style: { ...STYLE, redaction: "blur" } },
};

// Filled, it offers its colour.
export const Filled: Story = {
  args: { style: { ...STYLE, redaction: "color" } },
};

// Erased, there is nothing more to choose.
export const Erased: Story = {
  args: { style: { ...STYLE, redaction: "erase" } },
};

// A screenshot is one instant, so there is nothing to ramp in over.
export const Screenshot: Story = { args: { canAnimate: false } };
