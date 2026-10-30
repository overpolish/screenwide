// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { DEFAULT_KEYBOARD_EFFECTS } from "../../export/recording-export-settings";
import { EditorIslandStage } from "../editor-island-stage";
import {
  editorIslandStoryMeta,
  type EditorIslandStoryProps,
} from "../editor-island-story-meta";

import { keyboardIsland, type KeyboardIslandProps } from "./keyboard-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function KeyboardIsland({
  isShown,
  ...props
}: EditorIslandStoryProps & KeyboardIslandProps) {
  return (
    <EditorIslandStage content={keyboardIsland(props)} isShown={isShown} />
  );
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    canRestoreShortcuts: false,
    isShown: true,
    onChange: fn(),
    onDeleteChosen: fn(),
    onRestoreShortcuts: fn(),
    settings: DEFAULT_KEYBOARD_EFFECTS,
  },
  component: KeyboardIsland,
  decorators: [featureStage({ height: 320, width: 520 })],
  // Every choice changes the story's args, as it would the recording.
  render: function Render(args) {
    const [, updateArgs] = useArgs<KeyboardIslandProps>();
    return (
      <KeyboardIsland
        {...args}
        onChange={(patch) => {
          args.onChange(patch);
          updateArgs({ settings: { ...args.settings, ...patch } });
        }}
        onDeleteChosen={() => {
          args.onDeleteChosen?.();
          updateArgs({ canRestoreShortcuts: true, chosen: 0 });
        }}
        onRestoreShortcuts={() => {
          args.onRestoreShortcuts();
          updateArgs({ canRestoreShortcuts: false });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Keyboard",
} satisfies Meta<typeof KeyboardIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Recording: Story = {};

// Shortcuts turned off: nothing else to set, so the section folds away.
export const Hidden: Story = {
  args: { settings: { ...DEFAULT_KEYBOARD_EFFECTS, bake: false } },
};

// Some shortcuts deleted, with the way to bring them back.
export const DeletedShortcuts: Story = { args: { canRestoreShortcuts: true } };

// Two shortcuts chosen on the strip, with deleting them at the row's end.
export const ShortcutsChosen: Story = { args: { chosen: 2 } };
