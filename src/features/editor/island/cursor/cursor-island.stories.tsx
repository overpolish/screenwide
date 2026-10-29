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
    isShown: true,
    onChange: fn(),
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
      />
    );
  },
  title: "Features/Editor/Island/Cursor",
} satisfies Meta<typeof CursorIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};
