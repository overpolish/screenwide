// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BUILT_IN_BACKGROUND_PRESETS } from "../../../components/shared/background-picker/background-presets";
import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import {
  DEFAULT_CURSOR_EFFECTS,
  DEFAULT_KEYBOARD_EFFECTS,
} from "../recording-export-settings";

import { ToolPanel } from "./tool-panel";
import {
  DEFAULT_TOOL_PANEL_SNAPSHOT,
  ToolPanelSnapshot,
  useToolPanelStore,
} from "./tool-panel-store";

import type { Meta, StoryObj } from "@storybook/react-vite";

/** The panel window reads what the editor published, so a story seeds the
 * mirror the same way a live editor fills it. */
const seed = (snapshot: Partial<ToolPanelSnapshot>) => {
  useToolPanelStore.setState({
    snapshots: { recording: { ...DEFAULT_TOOL_PANEL_SNAPSHOT, ...snapshot } },
  });
};

const meta = {
  args: { tool: "cursor", workspace: "recording" },
  component: ToolPanel,
  decorators: [
    (Story, context) => (
      <FeatureStoryStage viewMode={context.viewMode} width={toolPanelWidth}>
        <Story />
      </FeatureStoryStage>
    ),
  ],
  parameters: { layout: "fullscreen" },
  title: "Features/Editor Tool Panel",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Cursor: Story = {
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isSaving: false,
      selection: null,
    });
  },
};

/** A recording captured without cursor movement: the tool has nothing to
 * offer, and says so rather than showing controls that do nothing. */
export const WithoutCursorData: Story = {
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: false,
      isSaving: false,
      selection: null,
    });
  },
};

/** The Keyboard tool's panel: how every captured shortcut is drawn, with the
 * two ways back from what the timeline and the canvas have done to them. */
export const Keyboard: Story = {
  args: { tool: "keyboard", workspace: "recording" },
  beforeEach: () => {
    seed({
      canRestoreShortcuts: true,
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      hasKeyboardData: true,
      isSaving: false,
      keyboardEffects: DEFAULT_KEYBOARD_EFFECTS,
      keyboardMaximum: 240,
      selection: null,
    });
  },
};

/** A recording captured without keyboard shortcuts: the tool has nothing to
 * offer, and says so rather than showing controls that do nothing. */
export const WithoutKeyboardData: Story = {
  args: { tool: "keyboard", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasKeyboardData: false,
      isSaving: false,
      selection: null,
    });
  },
};

/** The screen track of a recording, at the size and position the preview
 * would be showing it, padded out past its own picture. */
export const Selection: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isSaving: false,
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
      isSaving: false,
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
      isSaving: false,
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

/** The tool is in hand with nothing under it: the panel says so rather than
 * offering fields that would place nothing. */
export const SelectionEmpty: Story = {
  args: { tool: "selection", workspace: "recording" },
  beforeEach: () => {
    seed({
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isSaving: false,
      selection: null,
    });
  },
};

/** The finished picture's own size, with the capture's size under it: what a
 * reset puts the canvas back to. */
export const Frame: Story = {
  args: { tool: "frame", workspace: "recording" },
  beforeEach: () => {
    seed({
      background: BUILT_IN_BACKGROUND_PRESETS[0].background,
      backgroundPresets: [
        {
          background: { color: "#0B3D2E", kind: "solid" },
          id: "saved-forest",
          name: "Forest",
        },
      ],
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: {
        height: 2338,
        sourceHeight: 2338,
        sourceWidth: 3600,
        width: 3600,
      },
      hasCursorData: true,
      isSaving: false,
      selection: null,
    });
  },
};

/** The Crop tool's panel: how much of the capture is kept, in source pixels,
 * with the whole capture behind it for a reset to return to. */
export const Crop: Story = {
  args: { tool: "crop", workspace: "recording" },
  beforeEach: () => {
    seed({
      crop: {
        height: 1800,
        sourceHeight: 2338,
        sourceWidth: 3600,
        width: 2880,
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      hasCursorData: true,
      isSaving: false,
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
      isSaving: false,
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
