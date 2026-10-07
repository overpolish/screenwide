// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";

import { MomentNotePanel } from "./moment-note-panel";

import type { PopupPanelNoteContent } from "../../../popup-panel/store";
import type { Meta, StoryObj } from "@storybook/react-vite";

const note: PopupPanelNoteContent = {
  artifactId: 1,
  color: "#0088ff",
  durationMs: 4_200,
  kind: "note",
  moment: 1,
  name: "Notable",
  transcript: {
    status: "ready",
    text: "Cut the intro here, the demo really starts once the settings window opens.",
  },
  waveform: Array.from(
    { length: 160 },
    (_, point) => Math.abs(Math.sin(point / 7)) * 0.8,
  ),
};

const meta = {
  args: { content: note },
  component: MomentNotePanel,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={260}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Moment Note",
} satisfies Meta<typeof MomentNotePanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A voice note opened from its pin, with what was said under it. Outside
 * the app there is no recording to fetch its sound from, so Play does
 * nothing here. */
export const Transcribed: Story = {};

/** A note long enough that its transcript scrolls in the panel. */
export const LongTranscript: Story = {
  args: {
    content: {
      ...note,
      durationMs: 41_000,
      transcript: {
        status: "ready",
        text: "Okay so this whole part where the export dialog opens twice is a bug, we should cut from where I click Export to where the progress bar finishes. Then keep the part after, where the file shows up in Finder, because that is the payoff. Also the cursor wobbles a lot at the start, maybe slow that bit down or zoom in on the button instead.",
      },
    },
  },
};

export const Transcribing: Story = {
  args: { content: { ...note, transcript: { status: "transcribing" } } },
};

/** Behind other notes in the queue. */
export const Waiting: Story = {
  args: { content: { ...note, transcript: { status: "queued" } } },
};

/** No model downloaded yet: the note plays, and the panel offers the model
 * it needs. */
export const NoModel: Story = {
  args: {
    content: { ...note, transcript: { model: "base", status: "noModel" } },
  },
};

/** The transcriber could not make anything of it; it is tried again the
 * next time the app starts. */
export const Failed: Story = {
  args: { content: { ...note, transcript: { status: "failed" } } },
};
