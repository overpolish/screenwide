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

import {
  imagePickIsland,
  type ImagePickIslandProps,
} from "./image-pick-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function ImagePickIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & ImagePickIslandProps) {
  return (
    <EditorIslandStage content={imagePickIsland(props)} isShown={isShown} />
  );
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    isAnimated: true,
    isShown: true,
    onAnimatedChange: fn(),
    onPlace: fn(),
  },
  component: ImagePickIsland,
  decorators: [featureStage({ height: 480, width: 560 })],
  // The animation changes the story's args, as it would the next image.
  render: function Render(args) {
    const [, updateArgs] = useArgs<ImagePickIslandProps>();
    return (
      <ImagePickIsland
        {...args}
        onAnimatedChange={(isAnimated) => {
          args.onAnimatedChange(isAnimated);
          updateArgs({ isAnimated });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Annotation/Image Pick",
} satisfies Meta<typeof ImagePickIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// A screenshot is one instant: nothing pops in.
export const Screenshot: Story = { args: { canAnimate: false } };
