// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useArgs, useEffect } from "storybook/preview-api";
import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";

import {
  momentIsland,
  type MomentIslandProps,
  type MomentNote,
} from "./moment-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

const note: MomentNote = {
  isPlaying: false,
  peaks: Array.from(
    { length: 160 },
    (_, point) => Math.abs(Math.sin(point / 7)) * 0.8,
  ),
  progress: 0,
  transcript: {
    status: "ready",
    text: "Cut the intro here, the demo really starts once the settings window opens.",
  },
};

function MomentIsland(props: MomentIslandProps) {
  return <EditorIslandStage content={momentIsland(props)} />;
}

const meta = {
  // A kind's colour is any the user picks in Settings, not one of a set.
  argTypes: { color: { control: "color" } },
  args: {
    color: "#0088ff",
    id: "moment",
    name: "Notable",
    note,
    onCut: fn(),
    onDownloadModel: fn(),
    onTogglePlay: fn(),
  },
  component: MomentIsland,
  decorators: [featureStage({ height: 360, width: 480 })],
  parameters: {
    controls: { include: ["color", "name"] },
    layout: "fullscreen",
  },
  // Play and pause turn the button over, so the story can be tried; outside
  // the app there is no note to hear.
  render: function Render(args) {
    const [, updateArgs] = useArgs<MomentIslandProps>();
    return (
      <MomentIsland
        {...args}
        onTogglePlay={() => {
          args.onTogglePlay();
          if (args.note) {
            updateArgs({
              note: { ...args.note, isPlaying: !args.note.isPlaying },
            });
          }
        }}
      />
    );
  },
  title: "Features/Editor/Island/Moment",
} satisfies Meta<typeof MomentIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const WithNote: Story = {};

export const Playing: Story = {
  args: { note: { ...note, isPlaying: true, progress: 0.4 } },
};

// Long enough that what was said scrolls.
export const LongTranscript: Story = {
  args: {
    note: {
      ...note,
      transcript: {
        status: "ready",
        text: "Cut the intro here, the demo really starts once the settings window opens. Then keep the part where the export finishes, people always ask how long it takes, and the answer is right there on screen. After that it drags a little while I look for the folder, so that bit can go, but keep the end where the file opens.",
      },
    },
  },
};

export const WithoutNote: Story = { args: { note: null } };

export const Transcribing: Story = {
  args: { note: { ...note, transcript: { status: "transcribing" } } },
};

export const Waiting: Story = {
  args: { note: { ...note, transcript: { status: "queued" } } },
};

// Opened while the note is transcribed: the short line hangs beneath, then
// the words arrive and the part grows to the island's width.
export const TranscriptArrives: Story = {
  args: { note: { ...note, transcript: { status: "transcribing" } } },
  render: function Render(args) {
    const [, updateArgs] = useArgs<MomentIslandProps>();
    useEffect(() => {
      const arrive = window.setTimeout(() => {
        updateArgs({ note });
      }, 2000);
      return () => {
        window.clearTimeout(arrive);
      };
    }, [updateArgs]);
    return <MomentIsland {...args} />;
  },
};

export const NothingSaid: Story = {
  args: { note: { ...note, transcript: { status: "ready", text: "" } } },
};

export const Failed: Story = {
  args: { note: { ...note, transcript: { status: "failed" } } },
};

export const NoModel: Story = {
  args: {
    note: { ...note, transcript: { download: null, status: "noModel" } },
  },
};

export const Downloading: Story = {
  args: {
    note: { ...note, transcript: { download: 0.35, status: "noModel" } },
  },
};
