// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../../storybook/feature-story-stage";

import { MomentNotePanel } from "./moment-note-panel";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  args: {
    content: {
      artifactId: 1,
      color: "#0088ff",
      durationMs: 4_200,
      kind: "note",
      moment: 1,
      name: "Notable",
      waveform: Array.from(
        { length: 160 },
        (_, point) => Math.abs(Math.sin(point / 7)) * 0.8,
      ),
    },
  },
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

/** A voice note opened from its pin, ready to play. Outside the app there is
 * no recording to fetch its sound from, so Play does nothing here. */
export const Default: Story = {};
