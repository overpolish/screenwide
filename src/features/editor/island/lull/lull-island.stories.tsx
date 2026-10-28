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

import { lullIsland, type LullIslandProps } from "./lull-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function LullIsland({
  isShown,
  ...props
}: LullIslandProps & EditorIslandStoryProps) {
  return <EditorIslandStage content={lullIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    id: "lull",
    isShown: true,
    onCut: fn(),
    onPlayChange: fn(),
    onTrimToBreath: fn(),
    play: { kind: "squeeze" },
  },
  component: LullIsland,
  decorators: [featureStage({ height: 320, width: 480 })],
  // Choosing how the lull plays changes the story's args, so the glyph and
  // the tick follow, and a retimed lull loses its third choice once another
  // is chosen.
  render: function Render(args) {
    const [, updateArgs] = useArgs<LullIslandProps>();
    return (
      <LullIsland
        {...args}
        onPlayChange={(play) => {
          args.onPlayChange(play);
          updateArgs({ play: { kind: play } });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Lull",
} satisfies Meta<typeof LullIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Squeezed: Story = {};

export const Kept: Story = { args: { play: { kind: "keep" } } };

export const RetimedFaster: Story = {
  args: { play: { kind: "retimed", speed: 2.5 } },
};

export const RetimedSlower: Story = {
  args: { play: { kind: "retimed", speed: 0.5 } },
};
