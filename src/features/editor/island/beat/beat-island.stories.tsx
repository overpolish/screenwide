// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../../storybook/feature-story-stage";
import { EditorIslandStage } from "../editor-island-stage";

import { beatIsland, type BeatIslandProps } from "./beat-island";

import type { Meta, StoryObj } from "@storybook/react-vite";

function BeatIsland(props: BeatIslandProps) {
  return <EditorIslandStage content={beatIsland(props)} />;
}

const meta = {
  args: {
    beat: { kind: "click", label: "New Project" },
    id: "beat",
    onCut: fn(),
  },
  component: BeatIsland,
  decorators: [featureStage({ height: 160, width: 520 })],
  parameters: {
    controls: { disable: true },
    layout: "fullscreen",
  },
  title: "Features/Editor/Island/Beat",
} satisfies Meta<typeof BeatIsland>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Click: Story = {};

export const Type: Story = {
  args: { beat: { kind: "type", label: "Onboarding" } },
};

export const Keys: Story = {
  args: { beat: { keys: ["Command", "S"], kind: "keys" } },
};

export const Switch: Story = {
  args: { beat: { app: "Safari", kind: "switch" } },
};

export const NewScreen: Story = { args: { beat: { kind: "screen" } } };

export const LoudMoment: Story = { args: { beat: { kind: "sound" } } };

// A spoken line runs long, so it is cut short to keep the row a row.
export const Speech: Story = {
  args: {
    beat: {
      kind: "speech",
      line: "Let's set up a project from a template, then give it a name we will find again later.",
    },
  },
};
