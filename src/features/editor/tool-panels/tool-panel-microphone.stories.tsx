// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import { DEFAULT_CURSOR_EFFECTS } from "../export/recording-export-settings";

import { ToolPanel } from "./tool-panel";
import { ToolPanelMicrophone } from "./tool-panel-store";
import { seedToolPanel } from "./tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

const seed = seedToolPanel;

/** The Select tool on a microphone track: its speech tools as each stands. */
const meta = {
  args: { tool: "selection", workspace: "recording" },
  component: ToolPanel,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={toolPanelWidth}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor/Tool Panel/Microphone",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

const STUDIO_MODEL_BYTES = 1_005_394_589;

const microphone = (
  {
    autoVolume,
    noise,
    studio = "on",
    studioModel = {
      progress: 0,
      sizeBytes: STUDIO_MODEL_BYTES,
      status: "available",
    },
    voice,
  }: Pick<ToolPanelMicrophone, "autoVolume" | "noise" | "voice"> &
    Partial<Pick<ToolPanelMicrophone, "studio" | "studioModel">>,
  silences: Omit<ToolPanelMicrophone["silences"], "progress">,
  progress = { autoVolume: 0, noise: 0, silences: 0, studio: 0, voice: 0 },
) => {
  seed({
    cursorEffects: DEFAULT_CURSOR_EFFECTS,
    frame: null,
    hasCursorData: true,
    isLocked: false,
    selection: {
      decibels: 0,
      kind: "audio",
      label: "Microphone",
      microphone: {
        autoVolume,
        autoVolumeProgress: progress.autoVolume,
        noise,
        noiseProgress: progress.noise,
        silences: { ...silences, progress: progress.silences },
        studio,
        studioModel,
        studioProgress: progress.studio,
        voice,
        voiceProgress: progress.voice,
      },
    },
  });
};

/** A microphone before any speech tool has run. */
export const SelectionMicrophone: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      { autoVolume: "off", noise: "off", voice: "off" },
      { count: 0, durationMs: 0, status: "idle" },
    );
  },
};

/** The track being cleaned, its voice cleaned up, its volume measured and
 * the pauses being found, each with a bar for how far it has got. */
export const SelectionMicrophoneListening: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      { autoVolume: "cleaning", noise: "cleaning", voice: "cleaning" },
      { count: 0, durationMs: 0, status: "finding" },
      { autoVolume: 0.6, noise: 0.45, silences: 0.8, studio: 0, voice: 0.2 },
    );
  },
};

/** Every tool on and the pauses cut, which Restore all brings back. The
 * summary at its longest, a three-figure count and over an hour cut, still
 * fits the panel on one line. */
export const SelectionMicrophoneCleaned: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      { autoVolume: "on", noise: "on", voice: "on" },
      { count: 128, durationMs: 3_735_000, status: "idle" },
    );
  },
};

/** Speech with hardly a pause: Remove found nothing long enough to cut. */
export const SelectionMicrophoneNoPauses: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      { autoVolume: "off", noise: "off", voice: "off" },
      { count: 0, durationMs: 0, status: "none-found" },
    );
  },
};

/** The Studio sound model coming down, its bar under the row; Reduce noise
 * and Vocal cleanup stay until it is here. */
export const SelectionMicrophoneDownloadingStudioSound: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      {
        autoVolume: "on",
        noise: "on",
        studioModel: {
          progress: 0.35,
          sizeBytes: STUDIO_MODEL_BYTES,
          status: "downloading",
        },
        voice: "on",
      },
      { count: 0, durationMs: 0, status: "idle" },
    );
  },
};

/** Studio sound rebuilding the voice, in place of Reduce noise and Vocal
 * cleanup. */
export const SelectionMicrophoneStudioSound: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    microphone(
      {
        autoVolume: "on",
        noise: "on",
        studio: "cleaning",
        studioModel: {
          progress: 0,
          sizeBytes: STUDIO_MODEL_BYTES,
          status: "downloaded",
        },
        voice: "on",
      },
      { count: 0, durationMs: 0, status: "idle" },
      { autoVolume: 0, noise: 0, silences: 0, studio: 0.4, voice: 0 },
    );
  },
};
