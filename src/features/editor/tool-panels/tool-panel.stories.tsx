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
import { seedToolPanel } from "./tool-panel-story-seed";

import type { Meta, StoryObj } from "@storybook/react-vite";

const seed = seedToolPanel;

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
  title: "Features/Editor/Tool Panel",
} satisfies Meta<typeof ToolPanel>;

export default meta;
type Story = StoryObj<typeof meta>;

/** The mark panel over a chosen counter: a disc with a number in it has no
 * head to choose and no ends to swap, so the panel offers its size, whether
 * it arrives over its clip, and its colour. */
export const Counter: Story = {
  args: { tool: "mark", workspace: "recording" },
  beforeEach: () => {
    seed({
      annotation: {
        // A tail turned a quarter past east, so the Angle row shows an aim
        // rather than its own default.
        angle: Math.PI / 4,
        animated: true,
        id: "counter-1",
        kind: "counter",
        style: { color: "#ffcc00", head: "none", width: 56 },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isSaving: false,
      selection: null,
    });
  },
};

/** The Arrow panel: the mark the preview has in hand, and what it is drawn
 * in. It follows the chosen arrow rather than a tool, so it is the one panel
 * that comes up over another. */
export const Arrow: Story = {
  args: { tool: "mark", workspace: "recording" },
  beforeEach: () => {
    seed({
      annotation: {
        animated: true,
        id: "arrow-1",
        kind: "arrow",
        style: { color: "#ff383c", head: "end", width: 8 },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isSaving: false,
      selection: null,
    });
  },
};

/** The same panel wearing a colour of its own: colours chosen from the system
 * panel are kept after the palette, and the Custom tile is the chosen one
 * only while the mark matches no tile at all. */
export const ArrowCustomColour: Story = {
  args: { tool: "mark", workspace: "recording" },
  beforeEach: () => {
    seed({
      annotation: {
        animated: true,
        id: "arrow-1",
        kind: "arrow",
        style: { color: "#2ec4b6", head: "both", width: 16 },
      },
      annotationColors: ["#2ec4b6", "#8b5e34"],
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isSaving: false,
      selection: null,
    });
  },
};

/** Animate switched off: the mark stands on screen for the whole of its clip
 * instead of drawing itself in and out at its ends. */
export const ArrowWithoutAnimation: Story = {
  args: { tool: "mark", workspace: "recording" },
  beforeEach: () => {
    seed({
      annotation: {
        animated: false,
        id: "arrow-1",
        kind: "arrow",
        style: { color: "#ff383c", head: "end", width: 8 },
      },
      cursorEffects: DEFAULT_CURSOR_EFFECTS,
      frame: null,
      isSaving: false,
      selection: null,
    });
  },
};

/** The same panel in the screenshot editor. A still has no clip for a mark to
 * arrive over, so the Animate row is not offered at all and the panel is the
 * three it has always been. */
export const ArrowInAScreenshot: Story = {
  args: { tool: "mark", workspace: "screenshot" },
  beforeEach: () => {
    seed(
      {
        annotation: {
          animated: true,
          id: "arrow-1",
          kind: "arrow",
          style: { color: "#ff383c", head: "end", width: 8 },
        },
        cursorEffects: DEFAULT_CURSOR_EFFECTS,
        frame: null,
        isSaving: false,
        selection: null,
      },
      "screenshot",
    );
  },
};

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
        radius: 8,
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
