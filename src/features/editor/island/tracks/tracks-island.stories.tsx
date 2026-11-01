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

import { tracksIsland, type TracksIslandProps } from "./tracks-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function TracksIsland({
  isShown,
  ...props
}: TracksIslandProps & EditorIslandStoryProps) {
  return <EditorIslandStage content={tracksIsland(props)} isShown={isShown} />;
}

const meta = {
  ...editorIslandStoryMeta,
  args: {
    isShown: true,
    onEnabledChange: fn(),
    tracks: [
      { id: "primary", isEnabled: true, label: "Screen" },
      { id: "camera", isEnabled: true, label: "Camera" },
      { id: "audio:1", isEnabled: true, label: "Microphone" },
      { id: "audio:2", isEnabled: false, label: "System audio" },
    ],
  },
  component: TracksIsland,
  decorators: [featureStage({ height: 280, width: 480 })],
  // A switch turned lands in the args, so the last track left in shows
  // that it cannot be switched off.
  render: function Render(args) {
    const [, updateArgs] = useArgs<TracksIslandProps>();
    return (
      <TracksIsland
        {...args}
        onEnabledChange={(trackId, isEnabled) => {
          args.onEnabledChange(trackId, isEnabled);
          updateArgs({
            tracks: args.tracks.map((track) =>
              track.id === trackId ? { ...track, isEnabled } : track,
            ),
          });
        }}
      />
    );
  },
  title: "Features/Editor/Island/Tracks",
} satisfies Meta<typeof TracksIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const LastTrack: Story = {
  args: {
    tracks: [
      { id: "primary", isEnabled: true, label: "Screen" },
      { id: "audio:1", isEnabled: false, label: "Microphone" },
    ],
  },
};
