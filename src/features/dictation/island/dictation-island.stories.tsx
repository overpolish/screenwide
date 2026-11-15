// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { ISLAND } from "../../../primitives/island/island-geometry";
import { featureStage } from "../../../storybook/feature-story-stage";
import { PREVIEW_NOTCH } from "../../../storybook/preview-notch";
import { StoryNotch } from "../../../storybook/story-notch";

import { DictationIsland } from "./dictation-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

// Dictation's island at the top of the screen, under the notch from the
// toolbar when it is on.

const meta = {
  argTypes: {
    isShown: { table: { defaultValue: { summary: "true" } } },
    level: { control: { max: 1, min: 0, step: 0.05, type: "range" } },
  },
  args: {
    isShown: true,
    level: 0.55,
    onAction: fn(),
    state: {
      atPauses: false,
      phase: "listening",
      settled: "",
      taught: [],
      unsettled: "",
    },
  },
  component: DictationIsland,
  decorators: [
    (Story, { args, globals }) =>
      Story({
        args: {
          ...args,
          notch: globals.notch !== "off" ? PREVIEW_NOTCH : null,
        },
      }),
    featureStage({ height: 280, width: 720 }),
  ],
  parameters: {
    controls: { include: ["isShown", "level"] },
    layout: "fullscreen",
    notch: true,
  },
  render: (args) => (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        style={{ top: args.notch ? 0 : ISLAND.screenGap }}
      >
        <DictationIsland {...args} />
      </div>
      {args.notch ? <StoryNotch notch={args.notch} /> : null}
    </div>
  ),
  title: "Features/Dictation/Island/Phases",
} satisfies Meta<typeof DictationIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Listening: Story = {};

export const ListeningAtPauses: Story = {
  args: {
    state: {
      atPauses: true,
      phase: "listening",
      settled: "",
      taught: [],
      unsettled: "",
    },
  },
};

export const Hearing: Story = {
  args: {
    state: {
      atPauses: false,
      phase: "listening",
      settled: "Hi Sarah, can you send me the quarterly numbers?\n\n",
      taught: [],
      unsettled: "Sent with Screenwide",
    },
  },
};

export const AddingAWord: Story = {
  args: {
    state: {
      atPauses: false,
      phase: "listening",
      settled: "I recorded the demo with Screenwide.",
      taught: [{ outcome: "added", word: "Screenwide" }],
      unsettled: "",
    },
  },
};

export const Typing: Story = {
  args: { level: 0, state: { phase: "finishing" } },
};

export const Taught: Story = {
  args: {
    level: 0,
    state: {
      phase: "taught",
      taught: [
        { outcome: "added", word: "Screenwide" },
        { outcome: "notListed", word: "Overpolish" },
      ],
    },
  },
};

export const NoModel: Story = {
  args: { level: 0, state: { phase: "problem", problem: "noModel" } },
};

export const PasswordField: Story = {
  args: { level: 0, state: { phase: "problem", problem: "passwordField" } },
};
