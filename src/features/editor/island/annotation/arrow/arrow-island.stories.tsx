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
  PIN_FOLLOWING,
  PIN_OUT_OF_VIEW,
  PIN_TRACKING,
  PIN_UNSURE,
  PIN_WITH_CORRECTIONS,
  storyPinAfter,
} from "../annotation-pin-story-fixtures";

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
    onPinAction: fn(),
    onRemoveColor: fn(),
    onReverse: fn(),
    onSaveColor: fn(),
    onSceneReset: fn(),
    onStyleChange: fn(),
    pin: null,
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
        onPinAction={(action) => {
          args.onPinAction?.(action);
          updateArgs({ pin: storyPinAfter(args.pin, action) });
        }}
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
        onSceneReset={() => {
          args.onSceneReset?.();
          updateArgs({ placedInScene: false });
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

// A screenshot is one instant, so there is nothing to draw in over, and no
// content moving under it to pin to.
export const Screenshot: Story = {
  args: { canAnimate: false, pin: undefined },
};

export const PinnedFollowing: Story = { args: { pin: PIN_FOLLOWING } };

// Where the content is out of view at the playhead, it can be said back.
export const PinnedOutOfView: Story = { args: { pin: PIN_OUT_OF_VIEW } };

export const PinnedWithCorrections: Story = {
  args: { pin: PIN_WITH_CORRECTIONS },
};

// Where the tracker lost the content at the playhead, the note warns, so the
// place is checked; the answers still apply.
export const PinnedUnsure: Story = { args: { pin: PIN_UNSURE } };

// The path is still being worked out.
export const PinnedTracking: Story = { args: { pin: PIN_TRACKING } };

// Moved while the playhead is in a scene: it has a place of its own there,
// which its reset takes away.
export const PlacedInScene: Story = { args: { placedInScene: true } };
