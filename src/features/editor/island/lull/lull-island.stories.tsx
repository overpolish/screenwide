// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";

import { lullIsland, type LullIslandProps } from "./lull-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function LullIsland(props: LullIslandProps) {
  return <EditorIslandStage content={lullIsland(props)} />;
}

const meta = {
  args: {
    id: "lull",
    onCut: fn(),
    onPlayChange: fn(),
    onTrimToBreath: fn(),
    play: { kind: "squeeze" },
  },
  component: LullIsland,
  decorators: [featureStage({ height: 320, width: 480 })],
  parameters: {
    controls: { disable: true },
    layout: "fullscreen",
  },
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
