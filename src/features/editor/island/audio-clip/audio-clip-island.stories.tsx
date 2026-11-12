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

import {
  type AudioClipChange,
  audioClipIsland,
  type AudioClipIslandProps,
} from "./audio-clip-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function AudioClipIsland({
  isShown,
  ...props
}: AudioClipIslandProps & EditorIslandStoryProps) {
  return (
    <EditorIslandStage content={audioClipIsland(props)} isShown={isShown} />
  );
}

/** Levels that rise and fall like music, ten a second, the same each time. */
const storyLevels = (seconds: number) =>
  Array.from(
    { length: seconds * 10 },
    (_, index) =>
      0.3 +
      0.4 *
        Math.abs(Math.sin(index * 0.31)) *
        Math.abs(Math.sin(index * 0.013 + 0.4)),
  );

const meta = {
  ...editorIslandStoryMeta,
  args: {
    clipMs: 10_000,
    id: "song",
    isLooped: false,
    isShown: true,
    isSong: true,
    levels: storyLevels(600),
    makesWay: true,
    offsetMs: 60_000,
    onChange: fn(),
    preview: {
      isPlaying: false,
      lengthMs: 10_000,
      onTogglePlay: fn(),
      progress: 0,
    },
    soundMs: 600_000,
    title: "Habanera from Carmen (Georges Bizet) - Golden Hour Strings",
    volumeDb: 0,
  },
  component: AudioClipIsland,
  decorators: [featureStage({ height: 360, width: 480 })],
  // A change other than deleting or replacing updates the story's args, so
  // the fields and the loop toggle follow.
  render: function Render(args) {
    const [, updateArgs] = useArgs<AudioClipIslandProps>();
    return (
      <AudioClipIsland
        {...args}
        onChange={(change: AudioClipChange) => {
          args.onChange(change);
          const { delete: _delete, replace: _replace, ...values } = change;
          updateArgs(values);
        }}
      />
    );
  },
  title: "Features/Editor/Island/Audio Clip",
} satisfies Meta<typeof AudioClipIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Song: Story = {};

export const Previewing: Story = {
  args: {
    preview: {
      isPlaying: true,
      lengthMs: 30_000,
      onTogglePlay: fn(),
      progress: 0.4,
    },
  },
};

export const SoundEffect: Story = {
  args: {
    clipMs: 1_200,
    isSong: false,
    levels: storyLevels(2),
    offsetMs: 0,
    soundMs: 1_200,
    title: "Click",
  },
};
