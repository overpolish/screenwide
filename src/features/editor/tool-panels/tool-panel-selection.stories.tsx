// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
} from "../export/recording-export-settings";

import { ToolPanel } from "./tool-panel";
import {
  ToolPanelCameraEffects,
  ToolPanelCameraObjects,
} from "./tool-panel-store";
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

const BAKED_CAMERA = {
  dropShadow: false,
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
} as const;

const MODEL_BYTES = 14_975_696;

const cameraEffects = (
  effects: Partial<ToolPanelCameraEffects>,
): ToolPanelCameraEffects => ({
  background: "kept",
  colour: false,
  hasImage: false,
  model: { progress: 0, sizeBytes: MODEL_BYTES, status: "downloaded" },
  objects: null,
  progress: { background: 0, colour: 0 },
  working: { background: false, colour: false },
  ...effects,
});

const cameraStory = (effects: Partial<ToolPanelCameraEffects>): Story => ({
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      isLocked: false,
      selection: { ...BAKED_CAMERA, cameraEffects: cameraEffects(effects) },
    });
  },
});

/** A camera before the background removal model is here: a download in the
 * switch's place. */
export const SelectionCameraDownload = cameraStory({
  model: { progress: 0, sizeBytes: MODEL_BYTES, status: "available" },
});

/** The person being cut out of the room, with its bar. */
export const SelectionCameraRemovingBackground = cameraStory({
  background: "transparent",
  progress: { background: 0.4, colour: 0 },
  working: { background: true, colour: false },
});

/** The person in front of a picture, their colour being measured to
 * correct it towards the picture. */
export const SelectionCameraPicture = cameraStory({
  background: "image",
  colour: true,
  hasImage: true,
  progress: { background: 0, colour: 0.7 },
  working: { background: false, colour: true },
});

/** The room behind the person blurred. */
export const SelectionCameraBlurred = cameraStory({
  background: "blurred",
});

const OBJECTS_MODEL_BYTES = 190_082_794;

const cameraObjects = (
  objects: Partial<ToolPanelCameraObjects>,
): ToolPanelCameraObjects => ({
  chosen: null,
  isPicking: false,
  model: { progress: 0, sizeBytes: OBJECTS_MODEL_BYTES, status: "downloaded" },
  picks: [],
  progress: 0,
  working: false,
  ...objects,
});

/** A matted camera before the object model is here: a download for it. */
export const SelectionCameraObjectsDownload = cameraStory({
  background: "transparent",
  objects: cameraObjects({
    model: { progress: 0, sizeBytes: OBJECTS_MODEL_BYTES, status: "available" },
  }),
});

/** More objects than the row holds, scrolled to the one chosen. */
export const SelectionCameraObjects = cameraStory({
  background: "transparent",
  objects: cameraObjects({
    chosen: 7,
    isPicking: true,
    picks: Array.from({ length: 8 }, (_, id) => ({
      followed: true,
      following: null,
      id,
      keep: id % 3 !== 0,
      painted: id % 4 === 1,
    })),
  }),
});

/** Picking: a painted kept object chosen and followed, one waiting. */
export const SelectionCameraPickingObjects = cameraStory({
  background: "transparent",
  objects: cameraObjects({
    chosen: 1,
    isPicking: true,
    picks: [
      { followed: true, following: null, id: 0, keep: false, painted: false },
      { followed: false, following: 0.35, id: 1, keep: true, painted: true },
      { followed: false, following: null, id: 2, keep: false, painted: false },
    ],
  }),
});

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
