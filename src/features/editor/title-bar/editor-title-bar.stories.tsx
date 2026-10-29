// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { fn } from "storybook/test";

import { featureStage } from "../../../storybook/feature-story-stage";
import { StoryGrid, StoryGridRow } from "../../../storybook/story-grid";
import { recording, screenshot } from "../panel/editor-story-fixtures";

import { EditorTitleBar } from "./editor-title-bar";

import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  argTypes: {
    isSaving: { table: { defaultValue: { summary: "false" } } },
  },
  args: {
    artifact: recording,
    canExport: true,
    isSaving: false,
    onClose: fn(),
    onCopy: fn(),
    onDelete: fn(),
    onExport: fn(),
    onMinimize: fn(),
    onRename: fn(),
    onToggleMaximize: fn(),
  },
  component: EditorTitleBar,
  decorators: [featureStage({ width: 900 })],
  parameters: { controls: { include: ["canExport", "isSaving"] } },
  title: "Features/Editor/Title Bar",
} satisfies Meta<typeof EditorTitleBar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** A narrow editor window's width, so each bar in the grid spans one. */
const BAR_WIDTH = 800;

export const Kinds: Story = {
  parameters: { controls: { disable: true } },
  render: (args) => (
    <StoryGrid>
      <StoryGridRow label="recording">
        <div style={{ width: BAR_WIDTH }}>
          <EditorTitleBar {...args} artifact={recording} />
        </div>
      </StoryGridRow>
      <StoryGridRow label="screenshot">
        <div style={{ width: BAR_WIDTH }}>
          <EditorTitleBar {...args} artifact={screenshot} />
        </div>
      </StoryGridRow>
    </StoryGrid>
  ),
};
