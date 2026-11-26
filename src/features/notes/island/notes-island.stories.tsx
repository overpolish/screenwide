// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ISLAND } from "../../../primitives/island/island-geometry";
import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";
import { StoryNotch } from "../../../storybook/story-notch";

import { NotesIsland } from "./notes-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

// The island while notes of a call are taken, at the top of the screen,
// under the notch from the toolbar when it is on. The clock stands still,
// twelve and a half minutes in.

const STARTED_MS = 1_000_000;
const NOW = STARTED_MS + 750_000;

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
  },
  args: {
    isShown: true,
    microphone: {
      isMuted: false,
      level: 0.45,
      onMutedChange: fn(),
    },
    now: NOW,
    onAction: fn(),
    state: { loading: false, phase: "taking", startedMs: STARTED_MS },
  },
  component: NotesIsland,
  decorators: [
    (Story, { args, globals }) =>
      Story({
        args: {
          ...args,
          notch: globals.notch !== "off" ? PREVIEW_NOTCH : null,
        },
      }),
    featureStage({ height: 240, width: 720 }),
  ],
  parameters: {
    controls: { include: ["isShown"] },
    layout: "fullscreen",
    notch: true,
  },
  render: (args) => (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        style={{ top: args.notch ? 0 : ISLAND.screenGap }}
      >
        <NotesIsland {...args} />
      </div>
      {args.notch ? <StoryNotch notch={args.notch} /> : null}
    </div>
  ),
  title: "Features/Notes/Island/Phases",
} satisfies Meta<typeof NotesIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Taking: Story = {};

export const MicrophoneMuted: Story = {
  args: {
    microphone: {
      isMuted: true,
      level: 0,
      onMutedChange: fn(),
    },
  },
};

export const LoadingModel: Story = {
  args: { state: { loading: true, phase: "taking", startedMs: STARTED_MS } },
};

export const Finishing: Story = { args: { state: { phase: "finishing" } } };

export const Saved: Story = {
  args: {
    state: {
      file: "/Users/me/Movies/Screenwide/Notes 2026-05-04 at 10.30.00/notes.md",
      phase: "saved",
    },
  },
};

export const NoModel: Story = {
  args: { state: { phase: "problem", problem: "noModel" } },
};

export const NoMicrophone: Story = {
  args: { state: { phase: "problem", problem: "noMicrophone" } },
};

export const NoMicrophoneConnected: Story = {
  args: { state: { phase: "problem", problem: "noMicrophoneConnected" } },
};

export const NotStarted: Story = {
  args: { state: { phase: "problem", problem: "notStarted" } },
};

export const Failed: Story = {
  args: { state: { phase: "problem", problem: "failed" } },
};
