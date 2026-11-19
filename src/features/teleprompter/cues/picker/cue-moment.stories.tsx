// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { useEffect, useState } from "react";
import { fn } from "storybook/test";

import edit from "../../../../site/public/recordings/edit.jpg";
import { featureStage } from "../../../../storybook/feature-story-stage";
import { recordedTime } from "../../../editor/timeline/timeline-scale";

import { CueMoment } from "./cue-moment";
import { panelStory } from "./panel-story";

import type { Meta, StoryObj } from "@storybook/react-vite";

const DURATION = 94;

const meta = {
  args: {
    beats: [
      { end: 12, start: 4 },
      { color: "#F97316", end: 31, start: 26 },
      { end: 58, start: 40 },
      { end: 90, start: 71 },
    ],
    drawn: null,
    hasFailed: false,
    isMaking: false,
    map: recordedTime(DURATION),
    onBack: fn(),
    onConfirm: fn(),
    onPlayedChange: fn(),
    played: 30,
    strip: null,
    title: "Editor walkthrough",
  },
  component: CueMoment,
  decorators: [panelStory, featureStage({ width: 400 })],
  parameters: { controls: { include: ["hasFailed", "isMaking", "title"] } },
  // Scrubbing moves the story's own moment, and the picture stays drawn
  // wherever it rests, as the app draws it there. The picture is read into
  // a bitmap, as the app sends it.
  render: function Render(args) {
    const [played, setPlayed] = useState(args.played);
    const [picture, setPicture] = useState<ImageBitmap | null>(null);
    useEffect(() => {
      const image = new Image();
      image.src = edit;
      void image
        .decode()
        .then(() => createImageBitmap(image))
        .then(setPicture);
    }, []);
    return (
      <CueMoment
        {...args}
        drawn={picture && { at: played, picture }}
        onPlayedChange={(next) => {
          setPlayed(next);
          args.onPlayedChange(next);
        }}
        played={played}
      />
    );
  },
  title: "Features/Teleprompter/Cues/Moment",
} satisfies Meta<typeof CueMoment>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Making: Story = {
  args: { isMaking: true },
};

export const Failed: Story = {
  args: { hasFailed: true },
};
