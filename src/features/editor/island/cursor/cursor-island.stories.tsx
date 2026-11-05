// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { DEFAULT_CURSOR_EFFECTS } from "../../export/recording-export-settings";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { cursorIsland, type CursorIslandProps } from "./cursor-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function CursorIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & CursorIslandProps) {
  return <EditorIslandStage content={cursorIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    isCursorShown: true,
    isInScene: false,
    isShown: true,
    onChange: fn(),
    onShowChange: fn(),
    sceneCursor: null,
    settings: DEFAULT_CURSOR_EFFECTS,
  },
  component: CursorIsland,
  decorators: [featureStage({ height: 300, width: 520 })],
  // Every choice changes the story's args, as it would the recording.
  render: function Render(args) {
    const [, updateArgs] = useArgs<CursorIslandProps>();
    return (
      <CursorIsland
        {...args}
        onChange={(settings) => {
          args.onChange(settings);
          updateArgs({ settings });
        }}
        // Inside a scene the eye sets the scene's own choice, which goes back
        // to none once it matches the recording's; outside, the recording's.
        onShowChange={(isCursorShown) => {
          args.onShowChange(isCursorShown);
          updateArgs(
            args.isInScene
              ? {
                  isCursorShown,
                  sceneCursor:
                    isCursorShown === args.settings.bake ? null : isCursorShown,
                }
              : {
                  isCursorShown,
                  settings: { ...args.settings, bake: isCursorShown },
                },
          );
        }}
      />
    );
  },
  title: "Features/Editor/Island/Cursor",
} satisfies Meta<typeof CursorIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

export const Hidden: Story = {
  args: {
    isCursorShown: false,
    settings: { ...DEFAULT_CURSOR_EFFECTS, bake: false },
  },
};

export const InScene: Story = {
  args: { isInScene: true },
};

export const HiddenInScene: Story = {
  args: { isCursorShown: false, isInScene: true, sceneCursor: false },
};
