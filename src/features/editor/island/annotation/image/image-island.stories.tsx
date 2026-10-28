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

import { imageIsland, type ImageIslandProps } from "./image-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function ImageIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & ImageIslandProps) {
  return <EditorIslandStage content={imageIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    id: "image",
    isMoving: false,
    isShown: true,
    onDelete: fn(),
    onFlip: fn(),
    onReplace: fn(),
    onStyleChange: fn(),
    style: {
      hasShadow: true,
      isAnimated: true,
      loops: true,
      radius: 25,
      rotation: { x: 0, y: 0, z: 0 },
      sways: false,
    },
  },
  component: ImageIsland,
  decorators: [featureStage({ height: 420, width: 560 })],
  // Every choice changes the story's args, as it would the image.
  render: function Render(args) {
    const [, updateArgs] = useArgs<ImageIslandProps>();
    return (
      <ImageIsland
        {...args}
        onStyleChange={(style) => {
          args.onStyleChange(style);
          updateArgs({ style: { ...args.style, ...style } });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Annotation/Image",
} satisfies Meta<typeof ImageIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// A moving picture also chooses whether it loops.
export const MovingPicture: Story = { args: { isMoving: true } };

// A screenshot is one instant: nothing pops in, sways or plays.
export const Screenshot: Story = { args: { canAnimate: false } };
