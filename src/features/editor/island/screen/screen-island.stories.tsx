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

import { screenIsland, type ScreenIslandProps } from "./screen-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function ScreenIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & ScreenIslandProps) {
  return <EditorIslandStage content={screenIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  argTypes: {
    capture: {
      control: "inline-radio",
      options: ["recording", "screenshot"],
      table: { defaultValue: { summary: "recording" } },
    },
  },
  args: {
    capture: "recording",
    hasShadow: true,
    inset: 0,
    isAtDefaultPlacement: false,
    isInScene: false,
    isShown: true,
    onInsetChange: fn(),
    onRadiusChange: fn(),
    onRecenter: fn(),
    onResetPlacement: fn(),
    onShadowChange: fn(),
    radius: 4,
  },
  component: ScreenIsland,
  decorators: [featureStage({ height: 220, width: 520 })],
  // Every choice changes the story's picture, as it would the capture's.
  render: function Render(args) {
    const [, updateArgs] = useArgs<ScreenIslandProps>();
    return (
      <ScreenIsland
        {...args}
        onInsetChange={(inset) => {
          args.onInsetChange(inset);
          updateArgs({ inset });
        }}
        onRadiusChange={(radius) => {
          args.onRadiusChange(radius);
          updateArgs({ radius });
        }}
        onResetPlacement={() => {
          args.onResetPlacement();
          updateArgs({ isAtDefaultPlacement: true });
        }}
        onShadowChange={(hasShadow) => {
          args.onShadowChange(hasShadow);
          updateArgs({ hasShadow });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Screen",
} satisfies Meta<typeof ScreenIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

// Already where a fresh project places it, Reset placement has nothing to
// put back.
export const AtDefaultPlacement: Story = {
  args: { isAtDefaultPlacement: true },
};

// Over a scene, the inset, corners, turn, Recenter and Reset placement are
// the scene's own; the note says so.
export const InScene: Story = { args: { isInScene: true } };
