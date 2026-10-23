// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { featureStage } from "../../feature-story-stage";

import { EditorWindow } from "./editor-window";

import type { Meta, StoryObj } from "@storybook/react-vite";

// One working editor, to try at a top level: pick a tool in the left column
// and its options open beside it; pick a pen and point at the screen; select
// an annotation and drag its bar or ends on the timeline; hover the strip,
// scrub from the step names, drag its end in, select a beat and press
// Delete; switch to Steps and double-click a card. Space plays; Escape steps
// back.

const meta = {
  argTypes: {
    isVoiced: { control: "boolean" },
    kind: {
      control: "inline-radio",
      options: ["recording", "screenshot", "audio"],
      table: { defaultValue: { summary: "recording" } },
    },
  },
  component: EditorWindow,
  decorators: [featureStage({ window: "editor-recording" })],
  parameters: {
    controls: { include: ["isVoiced", "kind"] },
    layout: "fullscreen",
  },
  // Changing the kind or the voice starts the editor afresh.
  render: (args) => (
    <EditorWindow
      key={`${String(args.kind)}-${String(args.isVoiced)}`}
      {...args}
    />
  ),
  title: "Explorations/Editor",
} satisfies Meta<typeof EditorWindow>;

export default meta;
type Story = StoryObj<typeof meta>;

export const NoVoice: Story = { args: { isVoiced: false } };

export const WithVoice: Story = { args: { isVoiced: true } };

export const AnnotationSelected: Story = {
  args: {
    isVoiced: false,
    start: { playhead: 44, selection: { id: 3, kind: "mark" } },
  },
};

export const Steps: Story = {
  args: { isVoiced: false, start: { view: "board" } },
};

export const Screenshot: Story = { args: { kind: "screenshot" } };

export const SoundOnly: Story = {
  args: { kind: "audio", start: { playhead: 14 } },
};
