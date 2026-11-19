// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { fn } from "storybook/test";

import edit from "../../../../site/public/recordings/edit.jpg";
import sound from "../../../../site/public/recordings/sound.jpg";
import { featureStage } from "../../../../storybook/feature-story-stage";

import { CueSources } from "./cue-sources";
import { panelStory } from "./panel-story";

import type { ProjectSummary } from "../../../project-browser/types";
import type { Meta, StoryObj } from "@storybook/react-vite";

const project = (
  title: string,
  kind: ProjectSummary["kind"],
  durationMs: number | null,
): ProjectSummary => ({
  available: true,
  createdMs: null,
  durationMs,
  expiresMs: null,
  file: `/Movies/Screenwide/${title}/${title}.screenwide`,
  kind,
  modifiedMs: null,
  opening: null,
  replay: false,
  sizeBytes: null,
  title,
  tracks: null,
});

const PROJECTS = [
  project("Editor walkthrough", "screen", 94_000),
  project("Timeline close-up", "screenshot", null),
  project("Moments demo", "screen", 41_000),
  project("Intro to camera", "camera", 18_000),
];

const meta = {
  args: {
    hasFailed: false,
    isMaking: false,
    onChoose: fn(),
    projects: PROJECTS,
    stills: {
      [PROJECTS[0].file]: edit,
      [PROJECTS[2].file]: sound,
    },
  },
  component: CueSources,
  decorators: [panelStory, featureStage({ height: 440, width: 400 })],
  parameters: { controls: { include: ["hasFailed", "isMaking"] } },
  title: "Features/Teleprompter/Cues/Sources",
} satisfies Meta<typeof CueSources>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Empty: Story = {
  args: { projects: [] },
};

export const Failed: Story = {
  args: { hasFailed: true },
};
