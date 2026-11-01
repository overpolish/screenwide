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
import { storyPinAfter } from "../annotation-pin-story-fixtures";

import { focusIsland, type FocusIslandProps } from "./focus-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function FocusIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & FocusIslandProps) {
  return <EditorIslandStage content={focusIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canAnimate: true,
    id: "focus",
    isShown: true,
    onDelete: fn(),
    onPinAction: fn(),
    onStyleChange: fn(),
    pin: null,
    style: { dims: false, isAnimated: true, radius: 25 },
  },
  component: FocusIsland,
  decorators: [featureStage({ height: 260, width: 520 })],
  // Every choice changes the story's args, as it would the focus.
  render: function Render(args) {
    const [, updateArgs] = useArgs<FocusIslandProps>();
    return (
      <FocusIsland
        {...args}
        onPinAction={(action) => {
          args.onPinAction?.(action);
          updateArgs({ pin: storyPinAfter(args.pin, action) });
        }}
        onStyleChange={(style) => {
          args.onStyleChange(style);
          updateArgs({ style: { ...args.style, ...style } });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Annotation/Focus",
} satisfies Meta<typeof FocusIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// A screenshot is one instant, so there is nothing to deepen in over.
export const Screenshot: Story = {
  args: { canAnimate: false, pin: undefined },
};
