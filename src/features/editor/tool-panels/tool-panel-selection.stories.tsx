// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
} from "../export/recording-export-settings";

import { ToolPanel } from "./tool-panel";
import { ToolPanelMicrophone } from "./tool-panel-store";
import { seedToolPanel } from "./tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

const seed = seedToolPanel;

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
  title: "Features/Editor/Tool Panel/Selection",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The screen track of a recording, at the size and position the preview
 * would be showing it, padded out past its own picture. */
export const Selection: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: {
        dropShadow: true,
        height: 2458,
        inset: 60,
        insetMaximum: 2338,
        kind: "primary",
        label: "Screen",
        radius: 8,
        sourceHeight: 2338,
        sourceWidth: 3600,
        width: 3720,
        x: -60,
        y: -60,
      },
    });
  },
};

/** The camera track of a recording saved as a file of its own: it is placed
 * and padded the way any other layer is. */
export const SelectionCamera: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: {
        dropShadow: true,
        height: 720,
        inset: 0,
        insetMaximum: 720,
        kind: "camera",
        label: "Camera",
        radius: 8,
        sourceHeight: 720,
        sourceWidth: 1280,
        width: 1280,
        x: 2200,
        y: 1500,
      },
    });
  },
};

/** The same camera drawn into the screen's picture: the overlay is what is
 * placed, so the size is the window it is drawn in and there is no pad. */
export const SelectionBakedCamera: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: {
        dropShadow: true,
        height: 506,
        inset: 0,
        insetMaximum: 0,
        kind: "camera",
        label: "Camera",
        radius: 8,
        sourceHeight: 720,
        sourceWidth: 1280,
        width: 900,
        x: 2592,
        y: 73,
      },
    });
  },
};

/** A system audio track picked out with the Select tool: nothing of it is
 * placed, and it carries no speech, so the panel offers only how loud it is
 * played back. */
export const SelectionAudio: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: { decibels: 6, kind: "audio", label: "System audio" },
    });
  },
};

const microphone = (
  {
    autoVolume,
    noise,
    voice,
  }: Pick<ToolPanelMicrophone, "autoVolume" | "noise" | "voice">,
  silences: Omit<ToolPanelMicrophone["silences"], "progress">,
  progress = { autoVolume: 0, noise: 0, silences: 0, voice: 0 },
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
      { autoVolume: 0.6, noise: 0.45, silences: 0.8, voice: 0.2 },
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

/** The tool is in hand with nothing under it: the panel says so rather than
 * offering fields that would place nothing. */
export const SelectionEmpty: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: null,
    });
  },
};

/** A keyboard shortcut picked out with the Select tool: drawn rather than
 * placed, so it is sized and centred in percent and offered neither a shadow,
 * nor corners, nor a pad. */
export const SelectionShortcut: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasKeyboardData: true,
      isLocked: false,
      keyboardEffects: DEFAULT_KEYBOARD_EFFECTS,
      keyboardMaximum: 240,
      selection: {
        kind: "shortcut",
        label: "Shortcut",
        maximumSizePercent: 240,
        minimumSizePercent: 50,
        positionXPercent: 50,
        positionYPercent: 91.2,
        sizePercent: 100,
      },
    });
  },
};
