// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { BUILT_IN_BACKGROUND_PRESETS } from "../../../components/shared/background-picker/background-presets";
import { FeatureStoryStage } from "../../../storybook/feature-story-stage";
import { toolPanelWidth } from "../../popup-panel/layout";
import { DEFAULT_CURSOR_EFFECTS } from "../recording-export-settings";

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

/** The screen track of a recording, at the size and position the preview
 * would be showing it. */
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
        height: 2338,
        kind: "primary",
        label: "Screen",
        sourceHeight: 2338,
        sourceWidth: 3600,
        width: 3600,
        x: 0,
        y: 0,
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
