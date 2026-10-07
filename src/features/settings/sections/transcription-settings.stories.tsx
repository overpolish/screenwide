// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { use, useState } from "react";

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { SettingsApiContext } from "../settings-api-context";
import {
  transcriptionPreviewApi,
  type TranscriptionPreviewSeed,
} from "../settings-transcription-preview";

import { TranscriptionSettingsPanel } from "./transcription-settings";
import { useTranscription } from "./use-transcription";

import type { Meta, StoryObj } from "@storybook/react-vite";

const ignoreError = () => undefined;

function Pane() {
  return (
    <TranscriptionSettingsPanel controls={useTranscription(ignoreError)} />
  );
}

/** The pane on its own, starting from `seed`, with a download that fills
 * over a few seconds and nothing reaching the network. */
function TranscriptionPreview({ seed }: { seed: TranscriptionPreviewSeed }) {
  const api = use(SettingsApiContext);
  const [preview] = useState(() => ({
    ...api,
    ...transcriptionPreviewApi(seed),
  }));
  return (
    <SettingsApiContext value={preview}>
      <Pane />
    </SettingsApiContext>
  );
}

const meta = {
  args: { seed: { progress: null, status: "available" } },
  component: TranscriptionPreview,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={520}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { controls: { disable: true }, layout: "fullscreen" },
  title: "Features/Settings/Transcription",
} satisfies Meta<typeof TranscriptionPreview>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Nothing downloaded yet: what a first visit shows. */
export const NoModel: Story = {};

export const Downloading: Story = {
  args: { seed: { progress: 0.42, status: "downloading" } },
};

export const Downloaded: Story = {
  args: { seed: { progress: null, status: "downloaded" } },
};
