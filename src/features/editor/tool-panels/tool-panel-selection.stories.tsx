// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
} from "../recording-export-settings";

import { ToolPanel } from "./tool-panel";
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

/** The camera track of a recording, carried as a picture of its own: baking
 * is off, so it is placed and padded the way any other layer is. */
export const SelectionCamera: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: {
        canBake: true,
        dropShadow: true,
        height: 720,
        inset: 0,
        insetMaximum: 720,
        isBaked: false,
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
        canBake: true,
        dropShadow: true,
        height: 506,
        inset: 0,
        insetMaximum: 0,
        isBaked: true,
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

/** An audio track picked out with the Select tool: nothing of it is placed,
 * so the panel offers only how loud it is played back. */
export const SelectionAudio: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isLocked: false,
      selection: { decibels: 6, kind: "audio", label: "Microphone" },
    });
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
