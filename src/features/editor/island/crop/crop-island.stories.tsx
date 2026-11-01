// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { cropIsland, type CropIslandProps } from "./crop-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function CropIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & CropIslandProps) {
  return <EditorIslandStage content={cropIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  argTypes: {
    aspect: {
      control: "inline-radio",
      options: ["free", "locked", "original", "1:1", "4:3", "16:9", "9:16"],
      table: { defaultValue: { summary: "free" } },
    },
  },
  args: {
    aspect: "free",
    canLock: true,
    isInScene: false,
    isShown: true,
    onAspectChange: fn(),
    onReset: fn(),
  },
  component: CropIsland,
  decorators: [featureStage({ height: 200, width: 520 })],
  // A shape chosen is the crop's from then on, until Reset crop frees it.
  render: function Render(args) {
    const [, updateArgs] = useArgs<CropIslandProps>();
    return (
      <CropIsland
        {...args}
        onAspectChange={(aspect) => {
          args.onAspectChange(aspect);
          updateArgs({ aspect });
        }}
        onReset={() => {
          args.onReset();
          updateArgs({ aspect: "free" });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Crop",
} satisfies Meta<typeof CropIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Over a scene, the crop is the scene's own; the note says so.
export const InScene: Story = { args: { isInScene: true } };

// With the whole capture still in, there is no drawn shape to lock.
export const Uncropped: Story = { args: { canLock: false } };
